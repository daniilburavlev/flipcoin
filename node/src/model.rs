use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AppInfo {
    address: String,
}

impl AppInfo {
    pub fn new(address: String) -> Self {
        Self { address }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NodeInfo {
    p2p_port: u16,
    peer_id: String,
}

impl NodeInfo {
    pub fn new(p2p_port: u16, peer_id: String) -> Self {
        Self { p2p_port, peer_id }
    }
}
