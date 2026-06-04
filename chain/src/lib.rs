use std::sync::Arc;

use block::block::Block;
use common::{AppResult, error::AppError};
use pool::AppState;
use state::State;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_cron_scheduler::{Job, JobScheduler};
use tx::tx::Tx;
use utxo::Utxo;
use validator::validator::Validator;
use voting::{Vote, Voting};
use wallet::Wallet;

use crate::model::PeerInfo;

pub mod model;

pub struct Chain {
    wallet: Wallet,
    info: PeerInfo,
    validator: Arc<Validator>,
    state: Arc<State>,
    voting: Arc<Voting>,
    outgoing_block_tx: Sender<Block>,
    outgoing_txs_tx: Sender<Tx>,
    outgoing_vote_tx: Sender<Vote>,
}

impl Chain {
    pub async fn new(
        wallet: Wallet,
        p2p_port: u16,
        state: &Arc<State>,
        nodes: Vec<String>,
    ) -> Self {
        let (outgoing_block_tx, outgoing_block_rx) = tokio::sync::mpsc::channel(100);
        let (outgoing_txs_tx, outgoing_txs_rx) = tokio::sync::mpsc::channel(100);
        let (outgoing_vote_tx, outgoing_vote_rx) = tokio::sync::mpsc::channel(100);
        let (incoming_block_rx, incoming_txs_rx, incoming_vote_rx) = p2p::run(
            wallet.clone(),
            p2p_port,
            outgoing_block_rx,
            outgoing_txs_rx,
            outgoing_vote_rx,
            nodes,
        )
        .await;
        let chain = Self {
            validator: Arc::new(Validator::new(wallet.clone(), state)),
            info: PeerInfo::new(&wallet, p2p_port),
            voting: Arc::new(Voting::default()),
            wallet,
            state: Arc::clone(state),
            outgoing_block_tx,
            outgoing_txs_tx,
            outgoing_vote_tx,
        };
        chain.run_validator().await;
        chain
            .run_state_updater(incoming_block_rx, incoming_txs_rx, incoming_vote_rx)
            .await;
        chain
    }

    async fn run_validator(&self) {
        tracing::info!("validator started");
        let validator = Arc::clone(&self.validator);
        let voting = Arc::clone(&self.voting);
        let block_tx = self.outgoing_block_tx.clone();
        let scheduler = JobScheduler::new().await.unwrap();
        scheduler
            .add(
                Job::new_async("*/12 * * * * *", move |_, _| {
                    tracing::info!("create block");
                    let validator = Arc::clone(&validator);
                    let voting = Arc::clone(&voting);
                    let block_tx = block_tx.clone();
                    Box::pin(async move {
                        match validator.create_block().await {
                            Ok((Some(block), _, _)) => {
                                if let Err(e) = block_tx.send(block).await {
                                    tracing::error!("{}", e);
                                }
                            }
                            Ok((None, height, stake)) => {
                                tracing::info!("other validator selected");
                                voting.start(height, stake).await;
                            }
                            Err(e) => tracing::error!("cannot create block: {}", e),
                        };
                    })
                })
                .unwrap(),
            )
            .await
            .unwrap();

        scheduler.start().await.unwrap();
    }

    async fn run_state_updater(
        &self,
        mut incoming_block_rx: Receiver<Block>,
        mut incoming_txs_rx: Receiver<Tx>,
        mut incoming_vote_rx: Receiver<Vote>,
    ) {
        let wallet = self.wallet.clone();
        let state = Arc::clone(&self.state);
        let validator = Arc::clone(&self.validator);
        let voting = Arc::clone(&self.voting);
        let outgoing_vote_tx = self.outgoing_vote_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    block = incoming_block_rx.recv() => add_new_block(&wallet, block, &validator, &state, &outgoing_vote_tx).await,
                    tx = incoming_txs_rx.recv() => add_tx(&state, tx).await,
                    vote = incoming_vote_rx.recv() => add_vote(&state, &voting, vote).await,
                }
            }
        });
    }

    pub async fn add_genesis(&self, block: Block) -> AppResult<()> {
        self.state.add_genesis(block.txs).await
    }

    pub async fn add_block(&self, block: Block) -> AppResult<()> {
        add_block(&self.wallet, block, &self.validator, &self.state)
            .await
            .ok_or(AppError::Other)?;
        Ok(())
    }

    pub async fn add_tx(&self, tx: Tx) -> AppResult<()> {
        self.state.add_tx(tx.clone()).await?;
        self.outgoing_txs_tx.send(tx).await.unwrap();
        Ok(())
    }

    pub async fn get_txs(&self) -> AppResult<Vec<Tx>> {
        self.state.get_txs().await
    }

    pub async fn get_block(&self, height: u64) -> AppResult<Option<Block>> {
        tracing::debug!("get block");
        self.state.get_block(height).await
    }

    pub async fn get_block_by_hash(&self, hash: String) -> AppResult<Option<Block>> {
        self.state.get_block_by_hash(hash).await
    }

    pub async fn get_tx(&self, tx_id: String) -> AppResult<Option<Tx>> {
        self.state.get_tx(tx_id).await
    }

    pub async fn get_last_block(&self) -> AppResult<Option<Block>> {
        self.state.get_last_block().await
    }

    pub fn get_info(&self) -> PeerInfo {
        self.info.clone()
    }

    pub async fn get_state(&self) -> AppState {
        self.state.get_state().await
    }

    pub async fn restore_pool(&self, state: AppState) {
        self.state.restore_pool(state).await
    }

    pub async fn restore_state(&self) -> AppResult<()> {
        self.state.restore_state().await
    }

    pub async fn get_utxos(&self, address: String) -> Vec<Utxo> {
        self.state.get_utxos(address).await
    }
}

async fn add_new_block(
    wallet: &Wallet,
    block: Option<Block>,
    validator: &Validator,
    state: &State,
    outgoing_vote_tx: &Sender<Vote>,
) {
    if let Some(block) = block
        && let Some(vote) = add_block(wallet, block, validator, state).await
    {
        outgoing_vote_tx.send(vote).await.unwrap();
    }
}

async fn add_block(
    wallet: &Wallet,
    block: Block,
    validator: &Validator,
    state: &State,
) -> Option<Vote> {
    let Ok(slot) = validator.slot_validator(&block).await else {
        tracing::warn!("cannot select validator");
        return None;
    };
    let stake = state.get_stake(wallet.address_str()).await;
    let vote = Vote::init(
        block.height,
        block.hash.clone(),
        slot.validator.clone(),
        stake.amount,
    );
    if let Err(e) = state.add_block(&slot.validator, &block).await {
        tracing::warn!("block not added: {}", e);
        return None;
    }
    match vote.approve(wallet) {
        Ok(vote) => Some(vote),
        Err(e) => {
            tracing::warn!("cannot approve vote: {}", e);
            None
        }
    }
}

async fn add_tx(state: &State, tx: Option<Tx>) {
    if let Some(tx) = tx
        && let Err(e) = state.add_tx(tx).await
    {
        tracing::warn!("tx not added: {}", e);
    }
}

async fn add_vote(state: &State, voting: &Voting, vote: Option<Vote>) {
    if let Some(vote) = vote {
        let stake = state.get_stake(vote.voter.clone()).await;
        if stake.amount != vote.stake {
            return;
        }
        voting.vote(vote).await;
    }
}
