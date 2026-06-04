use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use libp2p::{
    gossipsub::{self, Behaviour as Gossipsub, ConfigBuilder, MessageId, ValidationMode},
    identify::{self, Behaviour as Identify},
    identity::Keypair,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
pub(crate) struct PeerBehaviour {
    pub(crate) gossipsub: Gossipsub,
    pub(crate) identify: Identify,
}

impl PeerBehaviour {
    pub(crate) fn new(key: &Keypair) -> Self {
        Self {
            gossipsub: Self::gossipsub(key.clone()),
            identify: Self::identify(key.clone()),
        }
    }

    fn gossipsub(key: Keypair) -> Gossipsub {
        let message_id_fn = |msg: &gossipsub::Message| {
            let mut hasher = DefaultHasher::new();
            msg.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .expect("valid config");

        Gossipsub::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .expect("valid gossipsub")
    }

    fn identify(key: Keypair) -> Identify {
        Identify::new(
            identify::Config::new("/chain-id/1.0.0".to_string(), key.public())
                .with_push_listen_addr_updates(true),
        )
    }
}
