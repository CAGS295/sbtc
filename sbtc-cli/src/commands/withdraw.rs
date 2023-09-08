use std::{io::stdout, str::FromStr};

use bdk::bitcoin::{psbt::serialize::Serialize, Address as BitcoinAddress, PrivateKey};
use clap::Parser;

use crate::commands::utils::TransactionData;

#[derive(Parser, Debug, Clone)]
pub struct WithdrawalArgs {
    /// WIF of the Bitcoin P2WPKH address that will broadcast and pay for the withdrawal request
    #[clap(short, long)]
    wif: String,

    /// WIF of the Stacks address that owns sBTC to be withdrawn
    #[clap(short, long)]
    drawee_wif: String,

    /// Bitcoin address that will receive BTC
    #[clap(short('b'), long)]
    payee_address: String,

    /// The amount of sats to withdraw
    #[clap(short, long)]
    amount: u64,

    /// The amount of sats to send for the fulfillment fee
    #[clap(short, long)]
    fulfillment_fee: u64,

    /// Bitcoin address of the peg wallet
    #[clap(short, long)]
    peg_wallet: String,
}

pub fn build_withdrawal_tx(withdrawal: &WithdrawalArgs) -> anyhow::Result<()> {
    let broadcaster_bitcoin_private_key = PrivateKey::from_wif(&withdrawal.wif)?;
    let drawee_stacks_private_key = PrivateKey::from_wif(&withdrawal.drawee_wif)?.inner;
    let payee_bitcoin_address = BitcoinAddress::from_str(&withdrawal.payee_address)?;
    let peg_wallet_bitcoin_address = BitcoinAddress::from_str(&withdrawal.peg_wallet)?;

    let tx = sbtc_core::operations::op_return::withdrawal_request::build_withdrawal_tx(
        broadcaster_bitcoin_private_key,
        drawee_stacks_private_key,
        payee_bitcoin_address,
        peg_wallet_bitcoin_address,
        withdrawal.amount,
        withdrawal.fulfillment_fee,
    )?;

    serde_json::to_writer_pretty(
        stdout(),
        &TransactionData {
            id: tx.txid().to_string(),
            hex: hex::encode(tx.serialize()),
        },
    )?;

    Ok(())
}
