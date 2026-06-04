use std::{collections::HashSet, fs, path::Path, sync::Arc};

use chain::Chain;
use common::{AppResult, error::AppError};
use rand::seq::IteratorRandom;
use rpc::client::Client;
use state::State;
use storage::Storage;
use tx::tx::Tx;
use wallet::Wallet;

use crate::{api, config::Config, paths::init_paths};

pub struct Node {
    http_port: u16,
    chain: Chain,
    nodes: HashSet<String>,
}

impl Node {
    pub(crate) async fn init(config_path: &Path, genesis_path: &Path) -> AppResult<()> {
        Config::init()?;
        let config = Config::read(config_path)?;
        let txs = Self::load_genesis_txs(genesis_path)?;
        let storage = Storage::new(&config.storage).await?;
        let state = State::new(storage)?;
        state.add_genesis(txs).await?;
        Ok(())
    }

    fn load_genesis_txs(path: &Path) -> AppResult<Vec<Tx>> {
        let txs = fs::read_to_string(path)?;
        let txs: Vec<Tx> =
            serde_json::from_str(&txs).map_err(|e| AppError::Decoding(e.to_string()))?;
        Ok(txs)
    }

    pub(crate) async fn new(config_path: &Path) -> AppResult<Self> {
        init_paths()?;
        let config = Config::read(config_path)?;
        println!("{}", config);
        let storage = Storage::new(&config.storage).await?;
        let state = Arc::new(State::new(storage)?);
        let wallet = Wallet::read(&config.keystore)?;
        let nodes = Self::get_nodes(config.nodes.clone()).await?;
        let chain = Chain::new(wallet, config.p2p_port, &state, nodes).await;

        let node = Self {
            http_port: config.http_port,
            chain,
            nodes: config.nodes,
        };
        node.sync().await?;

        Ok(node)
    }

    async fn get_nodes(nodes: HashSet<String>) -> AppResult<Vec<String>> {
        let mut addresses = vec![];
        for node in nodes {
            let client = Client::new(node.clone());
            if let Ok(info) = client.get_info().await
                && let Some(address) = info.p2p_address(&node)
            {
                addresses.push(address);
            }
        }
        Ok(addresses)
    }

    async fn sync(&self) -> AppResult<()> {
        let mut rng = rand::rng();
        if let Some(remote) = self.nodes.iter().choose(&mut rng) {
            let client = Client::new(remote.clone());
            let mut height = if let Some(block) = self.chain.get_last_block().await? {
                block.height + 1
            } else {
                0
            };
            loop {
                let Some(block) = client.get_block(height).await? else {
                    break;
                };
                if height == 0 {
                    self.chain.add_genesis(block).await?;
                } else {
                    self.chain.add_block(block).await?;
                }
                height += 1;
                if height > 1000 {
                    break;
                }
            }
            let state = client.get_state().await?;
            self.chain.restore_pool(state).await;
        } else {
            self.chain.restore_state().await?;
        }
        Ok(())
    }

    pub(crate) async fn run(self) {
        api::run(self.http_port, self.chain).await;
    }
}
