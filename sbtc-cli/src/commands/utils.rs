use bdk::{
    bitcoin::PrivateKey, blockchain::ElectrumBlockchain, database::MemoryDatabase,
    electrum_client::Client, template::P2Wpkh, SyncOptions, Wallet,
};

use serde::Serialize;

pub fn init_blockchain() -> anyhow::Result<ElectrumBlockchain> {
    let client = Client::new("ssl://blockstream.info:993")?;
    let blockchain = ElectrumBlockchain::from(client);
    Ok(blockchain)
}

pub fn setup_wallet(private_key: PrivateKey) -> anyhow::Result<Wallet<MemoryDatabase>> {
    let blockchain = init_blockchain()?;
    let wallet = Wallet::new(
        P2Wpkh(private_key),
        Some(P2Wpkh(private_key)),
        private_key.network,
        MemoryDatabase::default(),
    )?;

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(wallet)
}

#[derive(Serialize)]
pub struct TransactionData {
    pub id: String,
    pub hex: String,
}
