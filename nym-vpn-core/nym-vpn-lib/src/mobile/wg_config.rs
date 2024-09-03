use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};

use ipnetwork::IpNetwork;
use nym_wg_go::{netstack, wireguard_go, PeerConfig, PrivateKey, PublicKey};

#[derive(Debug)]
pub struct WgNodeConfig {
    /// Interface configuration
    pub interface: WgInterface,

    /// Peer configuration
    pub peer: WgPeer,
}

pub struct WgInterface {
    /// WG client port.
    pub listen_port: Option<u16>,

    /// Private key used by wg client.
    pub private_key: PrivateKey,

    /// Addresses assigned on wg interface.
    pub addresses: Vec<IpNetwork>,

    /// DNS addresses.
    pub dns: Vec<IpAddr>,

    /// Device MTU.
    pub mtu: u16,
}

impl fmt::Debug for WgInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WgInterface")
            .field("listen_port", &self.listen_port)
            .field("private_key", &"(hidden)")
            .field("address", &self.addresses)
            .field("dns", &self.dns)
            .field("mtu", &self.mtu)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct WgPeer {
    /// Gateway public key.
    pub public_key: PublicKey,

    /// Gateway endpoint
    pub endpoint: SocketAddr,
}

impl WgPeer {
    #[cfg(target_os = "ios")]
    pub fn into_peer_endpoint_update(self) -> PeerEndpointUpdate {
        PeerEndpointUpdate {
            public_key: self.public_key,
            endpoint: self.endpoint,
        }
    }
}

impl WgNodeConfig {
    pub fn into_netstack_config(self) -> netstack::Config {
        netstack::Config {
            interface: netstack::InterfaceConfig {
                private_key: self.interface.private_key,
                local_addrs: self
                    .interface
                    .addresses
                    .into_iter()
                    .map(|x| x.ip())
                    .collect(),
                dns_addrs: self.interface.dns,
                mtu: self.interface.mtu,
            },
            peers: vec![PeerConfig {
                // todo: limit to loopback?
                allowed_ips: vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()],
                public_key: self.peer.public_key,
                preshared_key: None,
                endpoint: self.peer.endpoint,
            }],
        }
    }

    pub fn into_wireguard_config(self) -> wireguard_go::Config {
        wireguard_go::Config {
            interface: wireguard_go::InterfaceConfig {
                listen_port: self.interface.listen_port,
                private_key: self.interface.private_key,
                mtu: self.interface.mtu,
            },
            peers: vec![PeerConfig {
                public_key: self.peer.public_key,
                preshared_key: None,
                endpoint: self.peer.endpoint,
                allowed_ips: vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()],
            }],
        }
    }
}
