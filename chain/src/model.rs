use std::net::{IpAddr, ToSocketAddrs};

use p2p::address_to_peer_id;
use serde::{Deserialize, Serialize};
use url::Url;
use wallet::Wallet;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PeerInfo {
    address: String,
    peer_id: String,
    peer_port: u16,
}

impl PeerInfo {
    pub fn new(wallet: &Wallet, peer_port: u16) -> Self {
        Self {
            address: wallet.address_str(),
            peer_id: address_to_peer_id(&wallet.address()).unwrap(),
            peer_port,
        }
    }

    pub fn p2p_address(&self, url: &str) -> Option<String> {
        let ip = resolve_ip(url)?;
        Some(format!(
            "/ip4/{}/tcp/{}/p2p/{}",
            ip, self.peer_port, &self.peer_id
        ))
    }
}

fn resolve_ip(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(ip.to_string());
    }
    let addr = format!("{}:{}", host, 80).to_socket_addrs().ok()?.next()?;
    Some(addr.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recreate_info() {
        let wallet = Wallet::default();
        let port: u16 = 5555;
        let first = PeerInfo::new(&wallet, port);
        let second = PeerInfo::new(&wallet, port);
        assert_eq!(first, second);
    }

    #[test]
    fn p2p_address() {
        let wallet = Wallet::default();
        let port = 8080;
        let info = PeerInfo::new(&wallet, port);
        let address = info.p2p_address("http://192.89.12.1:4321/test").unwrap();
        assert_eq!(
            format!("/ip4/192.89.12.1/tcp/8080/p2p/{}", &info.peer_id),
            address
        );
    }

    #[test]
    fn ip() {
        let ip = resolve_ip("http://192.92.1.0:5421/path").unwrap();
        assert_eq!("192.92.1.0", ip);
    }
}
