pub mod config;
pub(crate) mod peer_behaviour;
pub(crate) mod runner;

use libp2p::PeerId;
pub use runner::run;

pub fn address_to_peer_id(address: &[u8]) -> Option<String> {
    let Ok(public) = libp2p::identity::secp256k1::PublicKey::try_from_bytes(address) else {
        return None;
    };
    let public = libp2p::identity::PublicKey::from(public);
    Some(PeerId::from_public_key(&public).to_string())
}
