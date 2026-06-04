use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use block::block::Block;
use chain::{Chain, model::PeerInfo};
use rpc::ApiResult;
use tokio::net::TcpListener;
use tx::tx::Tx;
use utxo::Utxo;

use crate::state::AppState;

pub async fn run(port: u16, chain: Chain) {
    let state = Arc::new(AppState::new(chain));

    let app = Router::new()
        .route("/api/blocks/{height_or_hash}", get(get_block))
        .route("/api/txs", post(add_tx))
        .route("/api/txs", get(get_txs))
        .route("/api/txs/{tx_id}", get(get_tx))
        .route("/api/info", get(get_info))
        .route("/api/state", get(get_state))
        .route("/api/utxos/{address}", get(get_utxos))
        .with_state(state);
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
pub(crate) async fn get_block(
    Path(height_or_hash): Path<String>,
    state: State<Arc<AppState>>,
) -> ApiResult<Json<Block>> {
    let block = if let Ok(height) = height_or_hash.parse::<u64>() {
        state.0.get_block(height).await?
    } else {
        state.0.get_block_by_hash(height_or_hash).await?
    };
    Ok(Json(block))
}

#[axum::debug_handler]
pub(crate) async fn add_tx(state: State<Arc<AppState>>, Json(tx): Json<Tx>) -> ApiResult<()> {
    state.0.add_tx(tx).await
}

#[axum::debug_handler]
pub(crate) async fn get_txs(state: State<Arc<AppState>>) -> ApiResult<Json<Vec<Tx>>> {
    Ok(Json(state.0.get_txs().await?))
}

#[axum::debug_handler]
pub(crate) async fn get_tx(
    Path(tx_id): Path<String>,
    state: State<Arc<AppState>>,
) -> ApiResult<Json<Tx>> {
    Ok(Json(state.0.get_tx(tx_id).await?))
}

#[axum::debug_handler]
async fn get_info(state: State<Arc<AppState>>) -> Json<PeerInfo> {
    Json(state.get_info())
}

#[axum::debug_handler]
async fn get_state(state: State<Arc<AppState>>) -> Json<pool::AppState> {
    Json(state.get_state().await)
}

#[axum::debug_handler]
async fn get_utxos(Path(address): Path<String>, state: State<Arc<AppState>>) -> Json<Vec<Utxo>> {
    Json(state.get_utxos(address).await)
}
