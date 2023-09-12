//! Config

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use blockstack_lib::vm::ContractName;
use clap::Parser;
use stacks_core::{
    wallet::{BitcoinCredentials, Credentials, Wallet},
    Network,
};

/// sBTC Alpha Romeo
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Where the config file is located
    #[arg(short, long, value_name = "FILE")]
    pub config_file: PathBuf,
}

/// System configuration. This is typically constructed once and never mutated throughout the systems lifetime.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory to persist the state of the system to
    pub state_directory: PathBuf,

    /// Path to the contract file
    pub contracts: Vec<(ContractName, PathBuf)>,

    /// Credentials used to interact with the Stacks network
    pub stacks_credentials: Credentials,

    /// Credentials used to interact with the Bitcoin network
    pub bitcoin_credentials: BitcoinCredentials,

    /// Address of a bitcoin node
    pub bitcoin_node_url: reqwest::Url,

    /// Address of a stacks node
    pub stacks_node_url: reqwest::Url,

    /// sBTC asset contract name
    pub contract_name: ContractName,
}

impl Config {
    /// Read the config file in the path
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_root = normalize(
            std::env::current_dir().unwrap(),
            path.as_ref().parent().unwrap(),
        );

        let config_file = ConfigFile::from_path(&path)?;
        let state_directory = normalize(config_root.clone(), config_file.state_directory);
        let contracts: Vec<(ContractName, PathBuf)> = config_file
            .contracts
            .iter()
            .map(|path| {
                let contract_name_string = format!(
                    "{}-{}",
                    config_file.contract_namespace,
                    path.file_stem().unwrap().to_str().unwrap()
                );
                let contract_name = ContractName::from(contract_name_string.as_str());

                (contract_name, normalize(&config_root, path))
            })
            .collect();
        let contract_name = contracts.last().unwrap().0.clone();
        let bitcoin_node_url = reqwest::Url::parse(&config_file.bitcoin_node_url)?;
        let stacks_node_url = reqwest::Url::parse(&config_file.stacks_node_url)?;

        let wallet = Wallet::new(config_file.network, &config_file.mnemonic)?;
        let bitcoin_credentials = wallet.bitcoin_credentials(0)?;
        let stacks_credentials = wallet.credentials(0)?;

        Ok(Self {
            state_directory,
            contracts,
            bitcoin_credentials,
            stacks_credentials,
            bitcoin_node_url,
            stacks_node_url,
            contract_name,
        })
    }
}

fn normalize(root_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> PathBuf {
    if path.as_ref().is_relative() {
        root_dir.as_ref().to_owned().join(path)
    } else {
        path.as_ref().into()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigFile {
    /// Directory to persist the state of the system to
    pub state_directory: PathBuf,

    /// Path to the contract file
    pub contracts: Vec<PathBuf>,

    /// Stacks network
    pub network: Network,

    /// Seed mnemonic
    pub mnemonic: String,

    /// Address of a bitcoin node
    pub bitcoin_node_url: String,

    /// Address of a stacks node
    pub stacks_node_url: String,

    /// sBTC asset contract name
    pub contract_namespace: String,
}

impl ConfigFile {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_file = File::open(&path)?;

        Ok(serde_json::from_reader(config_file)?)
    }
}
