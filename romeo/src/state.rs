//! State

use std::io::Cursor;

use bdk::bitcoin::{Address as BitcoinAddress, Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::codec::StacksMessageCodec;
use blockstack_lib::types::chainstate::StacksAddress;
use blockstack_lib::vm::types::PrincipalData;
use sbtc_core::operations::op_return;
use stacks_core::codec::Codec;
use tracing::debug;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::task::Task;

/// The whole state of the application
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct State {
    contract: Option<Response<StacksTxId>>,
    deposits: Vec<Deposit>,
    withdrawals: Vec<Withdrawal>,
    block_height: u64,
}

/// A parsed deposit
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Deposit {
    info: DepositInfo,
    mint: Option<Response<StacksTxId>>,
}

/// Relevant information for processing deposits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepositInfo {
    /// ID of the bitcoin deposit transaction
    pub txid: BitcoinTxId,

    /// Amount to deposit
    pub amount: u64,

    /// Recipient of the sBTC
    pub recipient: PrincipalData,

    /// Height of the Bitcoin blockchain where this tx is included
    pub block_height: u64,
}

impl Deposit {
    fn mint(&mut self, _block_height: u64) -> Option<Task> {
        todo!();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Withdrawal {
    info: WithdrawalInfo,
    burn: Option<Response<StacksTxId>>,
    fulfillment: Option<Response<BitcoinTxId>>,
}

/// Relevant information for processing withdrawals
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalInfo {
    txid: BitcoinTxId,
    amount: u64,
    source: StacksAddress,
    recipient: BitcoinAddress,
    block_height: u64,
}

impl Withdrawal {
    fn burn(&mut self, _block_height: u64) -> Option<Task> {
        todo!();
    }

    fn fulfill(&mut self, _block_height: u64) -> Option<Task> {
        todo!();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Response<T> {
    txid: T,
    status: TransactionStatus,
}

impl<T> Response<T>
where
    T: PartialEq + Eq,
{
    fn update_status(&mut self, txid: T, status: TransactionStatus) -> bool {
        if self.txid == txid {
            self.status = status;
            true
        } else {
            false
        }
    }
}

/// Spawn initial tasks given a recovered state
pub fn bootstrap(state: &State) -> Task {
    match state.contract {
        None => Task::CreateAssetContract,
        Some(_) => Task::FetchBitcoinBlock(Some(state.block_height)),
    }
}

/// The beating heart of Romeo.
/// This function updates the system state in response to an I/O event.
/// It returns any new I/O tasks the system need to perform alongside the updated state.
#[tracing::instrument(skip(config, state))]
pub fn update(config: &Config, state: State, event: Event) -> (State, Vec<Task>) {
    debug!("Handling update");

    match event {
        Event::StacksTransactionUpdate(txid, status) => {
            process_stacks_transaction_update(config, state, txid, status)
        }
        Event::BitcoinTransactionUpdate(txid, status) => {
            process_bitcoin_transaction_update(config, state, txid, status)
        }
        Event::BitcoinBlock(block) => process_bitcoin_block(config, state, block),
        Event::AssetContractCreated(txid) => process_asset_contract_created(config, state, txid),
        event => panic!("Cannot handle yet: {:?}", event),
    }
}

fn process_bitcoin_block(config: &Config, mut state: State, block: Block) -> (State, Vec<Task>) {
    let deposits = parse_deposits(config, &block);
    let withdrawals = parse_withdrawals(&block);

    state.deposits.extend_from_slice(&deposits);
    state.withdrawals.extend_from_slice(&withdrawals);

    state.block_height = block
        .bip34_block_height()
        .expect("Failed to get block height");

    let mut tasks = vec![Task::FetchBitcoinBlock(Some(state.block_height + 1))];

    let mint_tasks = state
        .deposits
        .iter_mut()
        .filter_map(|deposit| deposit.mint(state.block_height));

    let withdrawals = &mut state.withdrawals;

    let burn_tasks: Vec<_> = withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.burn(state.block_height))
        .collect();

    let fulfillment_tasks: Vec<_> = withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.fulfill(state.block_height))
        .collect();

    tasks.extend(mint_tasks);
    tasks.extend(burn_tasks);
    tasks.extend(fulfillment_tasks);

    (state, tasks)
}

fn parse_deposits(config: &Config, block: &Block) -> Vec<Deposit> {
    let block_height = block
        .bip34_block_height()
        .expect("Failed to get block height");

    block
        .txdata
        .iter()
        .cloned()
        .filter_map(|tx| {
            let txid = tx.txid();

            op_return::deposit::Deposit::parse(config.private_key.network, tx)
                .ok()
                .map(|parsed_deposit| Deposit {
                    info: DepositInfo {
                        txid,
                        amount: parsed_deposit.amount,
                        recipient: convert_principal_data(parsed_deposit.recipient),
                        block_height,
                    },
                    mint: None,
                })
        })
        .collect()
}

fn convert_principal_data(data: stacks_core::utils::PrincipalData) -> PrincipalData {
    let bytes = data.serialize_to_vec();

    PrincipalData::consensus_deserialize(&mut Cursor::new(bytes)).unwrap()
}

fn parse_withdrawals(_block: &Block) -> Vec<Withdrawal> {
    // TODO: #68
    vec![]
}

fn process_bitcoin_transaction_update(
    _config: &Config,
    state: State,
    _txid: BitcoinTxId,
    _status: TransactionStatus,
) -> (State, Vec<Task>) {
    // TODO: #67 and #68

    (state, vec![])
}

fn process_stacks_transaction_update(
    _config: &Config,
    mut state: State,
    txid: StacksTxId,
    status: TransactionStatus,
) -> (State, Vec<Task>) {
    if status == TransactionStatus::Rejected {
        panic!("Stacks transaction failed");
    }

    let tasks = if let Some(contract) = state.contract.as_ref() {
        if contract.txid == txid && contract.status == TransactionStatus::Broadcasted {
            vec![Task::FetchBitcoinBlock(None)]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let contract_response = state.contract.as_mut().into_iter();

    let deposit_responses = state
        .deposits
        .iter_mut()
        .filter_map(|deposit| deposit.mint.as_mut());

    let withdrawal_responses = state
        .withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.burn.as_mut());

    let statuses_updated: usize = std::iter::empty()
        .chain(contract_response)
        .chain(deposit_responses)
        .chain(withdrawal_responses)
        .map(|response| response.update_status(txid, status.clone()) as usize)
        .sum();

    if statuses_updated != 1 {
        panic!(
            "Unexpected number of statuses updated: {}",
            statuses_updated
        );
    }

    (state, tasks)
}

fn process_asset_contract_created(
    _config: &Config,
    mut state: State,
    txid: StacksTxId,
) -> (State, Vec<Task>) {
    state.contract = Some(Response {
        txid,
        status: TransactionStatus::Broadcasted,
    });

    let task = Task::CheckStacksTransactionStatus(txid);

    (state, vec![task])
}
