// TODO: ROAST coordination logic
//https://github.com/Trust-Machines/stacks-sbtc/issues/668

use crate::signer::{
    coordinator::{Coordinate, PublicKeys},
    SBTCResult,
};
use bitcoin::{PublicKey, Transaction as BitcoinTransaction};
use wsts::{bip340::SchnorrProof, common::Signature};

/// ROAST coordinator
#[derive(Default)]
pub struct Coordinator {}

impl Coordinate for Coordinator {
    /// Generate the sBTC wallet public key
    fn generate_sbtc_wallet_public_key(&self, _public_keys: &PublicKeys) -> SBTCResult<PublicKey> {
        todo!()
    }
    /// Run the signing round for the transaction
    fn run_signing_round(
        &self,
        _public_keys: &PublicKeys,
        _tx: &BitcoinTransaction,
    ) -> SBTCResult<(Signature, SchnorrProof)> {
        todo!()
    }
}
