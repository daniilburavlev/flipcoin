use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use common::AppResult;
use num_bigint::BigUint;
use rpc::client::Client;
use tx::tx::Tx;
use wallet::Wallet;

use crate::{
    config::Config,
    keystore::keygen,
    node::Node,
    paths::{config_path, keystore_path},
};

#[derive(Parser)]
#[command(version, about, long_about = "FlipCoin blockchain node")]
struct NodeCli {
    #[arg(
        long,
        value_name = "CONFIG",
        help = "Config file path",
        default_value = default_config_path()
    )]
    config: PathBuf,
    #[command(subcommand)]
    command: Option<NodeCmd>,
}

fn default_config_path() -> String {
    config_path().unwrap().to_string_lossy().to_string()
}

fn default_keystore_path() -> String {
    keystore_path().unwrap().to_string_lossy().to_string()
}

#[derive(Subcommand)]
enum NodeCmd {
    #[clap(about = "init new blockchain")]
    New {
        #[arg(long, value_name = "GENESIS", help = "Path to genesis.json file")]
        genesis: PathBuf,
    },
    #[clap(about = "generate keygen")]
    Keygen {
        #[arg(long, value_name = "KEYSTORE", help = "Path to keystore", default_value = default_keystore_path())]
        keystore: PathBuf,
    },
    #[clap(about = "Send ")]
    Transfer {
        #[arg(long, value_name = "REMOTE")]
        remote: Option<String>,
        #[arg(long, value_name = "KEYSTORE", default_value = default_keystore_path())]
        keystore: PathBuf,
        address: String,
        amount: BigUint,
    },
}

pub(crate) async fn start_cli() -> AppResult<()> {
    let cli = NodeCli::parse();
    if let Some(command) = cli.command {
        match command {
            NodeCmd::New { genesis } => init_node(&cli.config, &genesis).await?,
            NodeCmd::Keygen { keystore } => keygen(&keystore)?,
            NodeCmd::Transfer {
                remote,
                keystore,
                address,
                amount,
            } => transfer(remote, &keystore, address, amount).await?,
        }
    } else {
        run_node(&cli.config).await?;
    }
    Ok(())
}

async fn init_node(config: &Path, genesis_path: &Path) -> AppResult<()> {
    Node::init(config, genesis_path).await?;
    Ok(())
}

async fn run_node(config: &Path) -> AppResult<()> {
    let node = Node::new(config).await?;
    node.run().await;
    Ok(())
}

async fn transfer(
    remote: Option<String>,
    keystore: &Path,
    address: String,
    amount: BigUint,
) -> AppResult<()> {
    let remote = if let Some(remote) = remote {
        remote
    } else {
        let config = Config::read(&config_path()?)?;
        config.get_random_node()?
    };
    let wallet = Wallet::read(keystore)?;
    let client = Client::new(remote);
    let utxos = client.get_utxos(&wallet.address_str()).await?;
    let fee = amount.clone() / BigUint::from(10000u64);
    client
        .add_tx(Tx::transfer(&wallet, utxos, address, amount, fee)?)
        .await?;
    Ok(())
}
