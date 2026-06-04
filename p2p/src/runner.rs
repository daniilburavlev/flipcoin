use block::block::Block;
use common::error::AppError;
use futures::StreamExt;
use libp2p::{
    Multiaddr, Swarm, SwarmBuilder,
    gossipsub::{self, IdentTopic},
    identify,
    identity::{Keypair, secp256k1},
    noise,
    swarm::SwarmEvent,
    tcp, yamux,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tx::tx::Tx;
use voting::Vote;
use wallet::Wallet;

use crate::peer_behaviour::{PeerBehaviour, PeerBehaviourEvent};

const BLOCK_TOPIC: &str = "/block/1.0.0";
const TXS_TOPIC: &str = "/txs/1.0.0";
const VOTE_TOPIC: &str = "/vote/1.0.0";

pub async fn run(
    wallet: Wallet,
    p2p_port: u16,
    outgoing_block_rx: Receiver<Block>,
    outgoing_txs_rx: Receiver<Tx>,
    outgoing_vote_rx: Receiver<Vote>,
    nodes: Vec<String>,
) -> (Receiver<Block>, Receiver<Tx>, Receiver<Vote>) {
    let (incoming_block_tx, incoming_block_rx) = tokio::sync::mpsc::channel(1024);
    let (incoming_txs_tx, incoming_txs_rx) = tokio::sync::mpsc::channel(1024);
    let (incoming_vote_tx, incoming_vote_rx) = tokio::sync::mpsc::channel(1024);
    let runner = Runner::new(
        wallet.keypair(),
        p2p_port,
        incoming_block_tx,
        outgoing_block_rx,
        incoming_txs_tx,
        outgoing_txs_rx,
        incoming_vote_tx,
        outgoing_vote_rx,
    );
    tokio::spawn(runner.run(nodes));
    (incoming_block_rx, incoming_txs_rx, incoming_vote_rx)
}

pub(crate) struct Runner {
    port: u16,
    incoming_block_tx: Sender<Block>,
    outgoing_block_rx: Receiver<Block>,
    incoming_txs_tx: Sender<Tx>,
    outgoing_txs_rx: Receiver<Tx>,
    incoming_vote_tx: Sender<Vote>,
    outgoing_vote_rx: Receiver<Vote>,
    block_topic: IdentTopic,
    txs_topic: IdentTopic,
    vote_topic: IdentTopic,
    swarm: Swarm<PeerBehaviour>,
}

impl Runner {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        keypair: secp256k1::Keypair,
        port: u16,
        incoming_block_tx: Sender<Block>,
        outgoing_block_rx: Receiver<Block>,
        incoming_txs_tx: Sender<Tx>,
        outgoing_txs_rx: Receiver<Tx>,
        incoming_vote_tx: Sender<Vote>,
        outgoing_vote_rx: Receiver<Vote>,
    ) -> Self {
        let swarm = Self::swarm(keypair);
        Self {
            port,
            incoming_block_tx,
            outgoing_block_rx,
            incoming_txs_tx,
            outgoing_txs_rx,
            incoming_vote_tx,
            outgoing_vote_rx,
            block_topic: IdentTopic::new(BLOCK_TOPIC),
            txs_topic: IdentTopic::new(TXS_TOPIC),
            vote_topic: IdentTopic::new(VOTE_TOPIC),
            swarm,
        }
    }

    fn swarm(keypair: secp256k1::Keypair) -> Swarm<PeerBehaviour> {
        let keypair = Keypair::from(keypair);
        SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .unwrap()
            .with_behaviour(|key| Ok(PeerBehaviour::new(key)))
            .unwrap()
            .build()
    }

    pub(crate) async fn run(mut self, nodes: Vec<String>) {
        let address: Multiaddr = build_address(self.port);
        self.swarm.listen_on(address).unwrap();

        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.block_topic)
            .unwrap();
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.txs_topic)
            .unwrap();
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.vote_topic)
            .unwrap();

        for node in nodes {
            let address: Multiaddr = node.parse().unwrap();
            self.swarm
                .dial(address)
                .map_err(|e| {
                    tracing::error!("{}", e);
                    AppError::Network
                })
                .unwrap();
        }

        loop {
            tokio::select! {
                block = self.outgoing_block_rx.recv() => self.send_block(block).await,
                tx = self.outgoing_txs_rx.recv() => self.send_tx(tx).await,
                vote = self.outgoing_vote_rx.recv() => self.send_vote(vote).await,
                event = self.swarm.select_next_some() => self.handle_event(event).await,
            }
        }
    }

    async fn send_block(&mut self, block: Option<Block>) {
        if let Some(block) = block {
            let bytes = serde_json::to_vec(&block).unwrap();
            if let Err(e) = self
                .swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.block_topic.clone(), bytes)
            {
                tracing::error!("block not published: {}", e);
            }
        }
    }

    async fn send_tx(&mut self, tx: Option<Tx>) {
        if let Some(tx) = tx {
            let bytes = serde_json::to_vec(&tx).unwrap();
            if let Err(e) = self
                .swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.block_topic.clone(), bytes)
            {
                tracing::error!("tx not published: {}", e);
            }
        }
    }

    async fn send_vote(&mut self, vote: Option<Vote>) {
        if let Some(vote) = vote {
            let bytes = serde_json::to_vec(&vote).unwrap();
            if let Err(e) = self
                .swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.vote_topic.clone(), bytes)
            {
                tracing::error!("vote not published: {}", e);
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<PeerBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(PeerBehaviourEvent::Identify(e)) => {
                self.handle_identify_event(e).await
            }
            SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(e)) => {
                self.handle_gossipsub_event(e).await
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("listening on: {address}/p2p/{}", self.swarm.local_peer_id());
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                tracing::info!("connected to {peer_id} via {endpoint:?}");
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                tracing::info!("disconnected from {peer_id}: {cause:?}");
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                tracing::warn!("failed to connect to {peer_id:?}: {error}")
            }
            _ => tracing::info!("unhanled event"),
        };
    }

    async fn handle_identify_event(&mut self, event: identify::Event) {
        match event {
            identify::Event::Received { peer_id, info, .. } => {
                tracing::info!(
                    "ID peer {peer_id}:\nagent: {}\nproto_version: {}\nprotocols={:?}\nlisten_address={:?}",
                    info.agent_version,
                    info.protocol_version,
                    info.protocols,
                    info.listen_addrs
                );
                for addr in &info.listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    self.swarm.add_peer_address(peer_id, addr.clone());
                }
            }
            identify::Event::Sent { peer_id, .. } => {
                tracing::info!("sent ID info to {peer_id}");
            }
            identify::Event::Pushed { peer_id, info, .. } => {
                tracing::info!(
                    "peer {peer_id} pushed updeted info: {:?}",
                    info.listen_addrs
                );
            }
            identify::Event::Error { peer_id, error, .. } => {
                tracing::error!("ID error with {peer_id}: {error}");
            }
        }
    }

    async fn handle_gossipsub_event(&mut self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message { message, .. } => {
                if message.topic == self.block_topic.hash() {
                    let block: Block = serde_json::from_slice(&message.data).unwrap();
                    self.incoming_block_tx.send(block).await.unwrap();
                } else if message.topic == self.txs_topic.hash() {
                    let tx: Tx = serde_json::from_slice(&message.data).unwrap();
                    self.incoming_txs_tx.send(tx).await.unwrap();
                } else if message.topic == self.vote_topic.hash() {
                    let vote: Vote = serde_json::from_slice(&message.data).unwrap();
                    self.incoming_vote_tx.send(vote).await.unwrap();
                } else {
                    tracing::warn!("unknown topic: {}", message.topic);
                }
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                tracing::info!("peer {peer_id} subscribed to {topic}");
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                tracing::info!("peer {peer_id} unsubscribed to {topic}");
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                tracing::warn!("{peer_id} does not support gossipsub");
            }
            _ => {}
        }
    }
}

fn build_address(port: u16) -> Multiaddr {
    format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap()
}
