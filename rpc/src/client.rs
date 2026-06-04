use block::block::Block;
use chain::model::PeerInfo;
use common::{AppResult, error::AppError};
use pool::AppState;
use tx::tx::Tx;
use utxo::Utxo;

use crate::model::ErrorResponse;

pub struct Client {
    client: reqwest::Client,
    host: String,
}

impl Client {
    pub fn new(host: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, host }
    }

    pub async fn get_info(&self) -> AppResult<PeerInfo> {
        let info = self
            .client
            .get(format!("{}/api/info", &self.host))
            .send()
            .await
            .map_err(to_err)?
            .error_for_status()
            .map_err(to_err)?
            .json()
            .await
            .map_err(to_err)?;
        Ok(info)
    }

    pub async fn get_block(&self, height: u64) -> AppResult<Option<Block>> {
        let res = self
            .client
            .get(format!("{}/api/blocks/{}", &self.host, height))
            .send()
            .await
            .map_err(to_err)?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let block = res
            .error_for_status()
            .map_err(to_err)?
            .json()
            .await
            .map_err(to_err)?;
        Ok(Some(block))
    }

    pub async fn get_state(&self) -> AppResult<AppState> {
        let state = self
            .client
            .get(format!("{}/api/state", &self.host))
            .send()
            .await
            .map_err(to_err)?
            .error_for_status()
            .map_err(to_err)?
            .json()
            .await
            .map_err(to_err)?;
        Ok(state)
    }

    pub async fn get_utxos(&self, address: &str) -> AppResult<Vec<Utxo>> {
        let utxos = self
            .client
            .get(format!("{}/api/utxos/{}", &self.host, address))
            .send()
            .await
            .map_err(to_err)?
            .error_for_status()
            .map_err(to_err)?
            .json()
            .await
            .map_err(to_err)?;
        Ok(utxos)
    }

    pub async fn add_tx(&self, tx: Tx) -> AppResult<()> {
        let response = self
            .client
            .post(format!("{}/api/txs", &self.host))
            .json(&tx)
            .send()
            .await
            .map_err(to_err)?;
        if response.status().is_client_error() {
            let error: ErrorResponse = response.json().await.map_err(to_err)?;
            return Err(AppError::AddTx(error.error));
        }
        Ok(())
    }
}

fn to_err<T: std::fmt::Display>(err: T) -> AppError {
    tracing::warn!("client error: {}", err);
    AppError::Other
}
