use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};

use ipnetwork::{IpNetwork, Ipv4Network};
use nym_wg_gateway_client::GatewayData;
#[cfg(target_os = "ios")]
use nym_wg_go::PeerEndpointUpdate;
use nym_wg_go::{wireguard_go, PeerConfig, PrivateKey, PublicKey};

#[cfg(any(target_os = "ios", target_os = "android"))]
use nym_wg_go::netstack;

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

    /// Mark used for mark-based routing.
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,

    /// Amnezia Configuration
    #[cfg(feature = "amnezia")]
    pub azwg_config: Option<nym_wg_go::amnezia::AmneziaConfig>,
}

impl fmt::Debug for WgInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("WgInterface");
        d.field("listen_port", &self.listen_port)
            .field("private_key", &"(hidden)")
            .field("address", &self.addresses)
            .field("dns", &self.dns)
            .field("mtu", &self.mtu);
        #[cfg(target_os = "linux")]
        d.field("fwmark", &self.fwmark);
        d.finish()
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
    #[cfg(any(target_os = "ios", target_os = "android"))]
    pub fn into_netstack_config(self) -> netstack::Config {
        let allowed_ips = self.allowed_ips();
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
                #[cfg(feature = "amnezia")]
                azwg_config: self.interface.azwg_config,
            },
            peers: vec![PeerConfig {
                public_key: self.peer.public_key,
                preshared_key: None,
                endpoint: self.peer.endpoint,
                // todo: limit to loopback?
                allowed_ips,
            }],
        }
    }

    pub fn into_wireguard_config(self) -> wireguard_go::Config {
        let allowed_ips = self.allowed_ips();
        wireguard_go::Config {
            interface: wireguard_go::InterfaceConfig {
                listen_port: self.interface.listen_port,
                private_key: self.interface.private_key,
                mtu: self.interface.mtu,
                #[cfg(target_os = "linux")]
                fwmark: self.interface.fwmark,
                #[cfg(feature = "amnezia")]
                azwg_config: self.interface.azwg_config,
            },
            peers: vec![PeerConfig {
                public_key: self.peer.public_key,
                preshared_key: None,
                endpoint: self.peer.endpoint,
                allowed_ips,
            }],
        }
    }

    fn allowed_ips(&self) -> Vec<IpNetwork> {
        let mut allowed_ips = vec![];
        if self.interface.addresses.iter().any(|x| x.ip().is_ipv4()) {
            allowed_ips.push("0.0.0.0/0".parse().unwrap());
        }
        if self.interface.addresses.iter().any(|x| x.ip().is_ipv6()) {
            allowed_ips.push("::/0".parse().unwrap());
        }
        allowed_ips
    }
}

impl WgNodeConfig {
    pub fn with_gateway_data(
        gateway_data: GatewayData,
        private_key: &nym_crypto::asymmetric::encryption::PrivateKey,
        dns: Vec<IpAddr>,
        mtu: u16,
    ) -> Self {
        Self {
            interface: WgInterface {
                listen_port: None,
                private_key: PrivateKey::from(private_key.to_bytes()),
                addresses: vec![IpNetwork::V4(Ipv4Network::from(gateway_data.private_ipv4))],
                dns,
                mtu,
                #[cfg(target_os = "linux")]
                fwmark: None,
                #[cfg(feature = "amnezia")]
                azwg_config: None,
            },
            peer: WgPeer {
                public_key: PublicKey::from(*gateway_data.public_key.as_bytes()),
                endpoint: gateway_data.endpoint,
            },
        }
    }
}
