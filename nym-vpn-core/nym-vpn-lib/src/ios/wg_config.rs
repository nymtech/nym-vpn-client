use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use ipnetwork::IpNetwork;
use nym_wg_go::{netstack, wireguard_go, PeerConfig, PresharedKey, PrivateKey, PublicKey};

#[derive(Debug)]
pub struct WgNodeConfig {
    /// Interface configuration
    pub interface: WgInterface,

    /// Peer configuration
    pub peer: WgPeer,
}

pub struct WgInterface {
    /// Private key used by wg client.
    pub private_key: PrivateKey,

    /// Addresses assigned on wg interface.
    pub addresses: Vec<IpNetwork>,

    /// DNS addresses.
    pub dns: Vec<IpAddr>,

    /// Device MTU.
    pub mtu: u16,
}

impl std::fmt::Debug for WgInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("WgInterface")
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

impl WgNodeConfig {
    pub fn with_gateway_data(
        private_key: &nym_crypto::asymmetric::encryption::PrivateKey,
        gateway_data: crate::wg_gateway_client::GatewayData,
    ) -> Self {
        Self {
            interface: WgInterface {
                addresses: vec![if gateway_data.private_ip.is_ipv4() {
                    IpNetwork::new(gateway_data.private_ip, 32).expect("private_ip v4/32")
                } else {
                    IpNetwork::new(gateway_data.private_ip, 128).expect("private_ip v6/128")
                }],
                private_key: PrivateKey::from(private_key.to_bytes()),
                dns: vec![],
                mtu: 0,
            },
            peer: WgPeer {
                public_key: PublicKey::from(*gateway_data.public_key.as_bytes()),
                endpoint: gateway_data.endpoint,
            },
        }
    }

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
                listen_port: None,
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
