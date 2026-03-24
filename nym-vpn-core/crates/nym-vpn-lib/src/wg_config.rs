// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_registration_common::WireguardConfiguration;
use nym_sdk::mixnet::x25519;
#[cfg(target_os = "ios")]
use nym_wg_go::PeerEndpointUpdate;
use nym_wg_go::{PeerConfig, amnezia::AmneziaConfig, netstack, wireguard_go};
use nym_wireguard_types::PresharedKey;

#[derive(Debug)]
pub struct WgNodeConfig {
    /// Interface configuration
    pub interface: WgInterface,

    /// Peer configuration
    pub peer: WgPeer,

    /// IPs that are allowed to be routed over the tunnel interface.
    pub allowed_ips: AllowedIps,
}

pub struct WgInterface {
    /// WG client port.
    pub listen_port: Option<u16>,

    /// Keypair, whose private key used by wg client.
    pub keypair: Arc<x25519::KeyPair>,

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
    pub azwg_config: Option<AmneziaConfig>,
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
        d.field("amnezia", &self.azwg_config);
        d.finish()
    }
}

/// IPs that are allowed to be routed over the tunnel interface.
#[derive(Debug, Clone)]
pub enum AllowedIps {
    /// All IPs are allowed.
    All,

    /// Specific IPs are allowed.
    Specific(Vec<IpNetwork>),
}

pub struct WgPeer {
    /// Gateway public key.
    pub public_key: x25519::PublicKey,

    pub preshared_key: Option<PresharedKey>,

    /// Gateway endpoint
    pub endpoint: SocketAddr,
}

impl WgPeer {
    #[cfg(target_os = "ios")]
    pub fn as_peer_endpoint_update(&self) -> PeerEndpointUpdate {
        PeerEndpointUpdate {
            public_key: self.public_key,
            endpoint: self.endpoint,
        }
    }
}

impl fmt::Debug for WgPeer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("WgPeer");
        d.field("public_key", &self.public_key)
            .field("preshared_key", &"(hidden)")
            .field("endpoint", &self.endpoint);
        d.finish()
    }
}

impl WgNodeConfig {
    pub fn into_netstack_config(self) -> netstack::Config {
        let allowed_ips = self.allowed_ips();
        netstack::Config {
            interface: netstack::InterfaceConfig {
                keypair: self.interface.keypair,
                local_addrs: self
                    .interface
                    .addresses
                    .into_iter()
                    .map(|x| x.ip())
                    .collect(),
                dns_addrs: self.interface.dns,
                mtu: self.interface.mtu,
                #[cfg(target_os = "linux")]
                fwmark: self.interface.fwmark,
                azwg_config: self.interface.azwg_config,
            },
            peers: vec![PeerConfig {
                public_key: self.peer.public_key,
                preshared_key: self.peer.preshared_key,
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
                keypair: self.interface.keypair,
                mtu: self.interface.mtu,
                #[cfg(target_os = "linux")]
                fwmark: self.interface.fwmark,
                azwg_config: self.interface.azwg_config,
            },
            peers: vec![PeerConfig {
                public_key: self.peer.public_key,
                preshared_key: self.peer.preshared_key,
                endpoint: self.peer.endpoint,
                allowed_ips,
            }],
        }
    }

    fn allowed_ips(&self) -> Vec<IpNetwork> {
        let mut allowed_ips = vec![];
        match self.allowed_ips {
            AllowedIps::All => {
                if self.interface.addresses.iter().any(|x| x.ip().is_ipv4()) {
                    allowed_ips.push("0.0.0.0/0".parse().unwrap());
                }
                if self.interface.addresses.iter().any(|x| x.ip().is_ipv6()) {
                    allowed_ips.push("::/0".parse().unwrap());
                }
            }
            AllowedIps::Specific(ref ips) => {
                allowed_ips.extend(ips);
            }
        }
        allowed_ips
    }
}

impl WgNodeConfig {
    pub fn with_wireguard_config(
        wireguard_config: WireguardConfiguration,
        keypair: Arc<x25519::KeyPair>,
        allowed_ips: AllowedIps,
        dns: Vec<IpAddr>,
        mtu: u16,
        #[cfg(target_os = "linux")] fwmark: Option<u32>,
    ) -> Self {
        Self {
            interface: WgInterface {
                listen_port: None,
                keypair,
                addresses: vec![
                    IpNetwork::V4(Ipv4Network::from(wireguard_config.private_ipv4)),
                    IpNetwork::V6(Ipv6Network::from(wireguard_config.private_ipv6)),
                ],
                dns,
                mtu,
                #[cfg(target_os = "linux")]
                fwmark,
                azwg_config: Some(AmneziaConfig::OFF),
            },
            peer: WgPeer {
                public_key: wireguard_config.public_key,
                preshared_key: wireguard_config.psk,
                endpoint: wireguard_config.endpoint,
            },
            allowed_ips,
        }
    }

    /// Enable Amnezia wireguard features
    pub fn with_amnezia_config(mut self, azwg_config: AmneziaConfig) -> Self {
        self.interface.azwg_config = Some(azwg_config);
        self
    }
}
