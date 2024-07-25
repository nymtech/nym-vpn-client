// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

//! WireGuard tunnel creation and management on Android and iOS
//! todo: the location of this module will be changed.

mod dns64;
mod gateway;
pub mod tun;
pub mod tunnel_settings;

use std::net::{IpAddr, SocketAddr};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use crate::bandwidth_controller::BandwidthController;
use crate::error::Result;
use crate::mixnet_connect::SharedMixnetClient;
use crate::wg_gateway_client::{GatewayData, WgGatewayClient};
use crate::NymVpn;
use crate::WireguardVpn;

use nym_authenticator_client::AuthClient;
use nym_gateway_directory::{AuthAddresses, GatewayClient};
use nym_task::TaskManager;
use nym_wg_go::{netstack, wireguard_go, PeerConfig, PrivateKey, PublicKey};

/// Entry tunnel MTU.
const ENTRY_MTU: u16 = 1280;

/// Exit tunnel MTU.
const EXIT_MTU: u16 = 1200;

/// Local port used for accepting exit traffic.
const UDP_FORWARDER_PORT: u16 = 34001;

/// Local port used by exit tunnel when sending traffic to the udp forwarder.
const EXIT_WG_CLIENT_PORT: u16 = 54001;

/// Two-hop WireGuard tunnel.
///
/// ## Abstract
///
/// In principle the two-hop WireGuard is implemented in the following way:
///
/// * The tunnel to the entry node is established using wg/netstack.
/// * The UDP connection to the exit node is established over the entry tunnel.
/// * The exit traffic is captured on tun interface and directed towards local UDP forwarding proxy.
/// * The local UDP forwarding proxy injects all received UDP datagrams into the UDP connection to the exit node.
#[derive(Debug)]
pub struct TwoHopTunnel {
    /// Entry node tunnel
    #[allow(unused)]
    entry: netstack::Tunnel,

    /// Exit node tunnel
    #[allow(unused)]
    exit: wireguard_go::Tunnel,

    /// UDP connection over the entry tunnel, towards exit node.
    #[allow(unused)]
    exit_connection: netstack::TunnelConnection,

    /// Entry peer configuration.
    /// todo: use it later to update peers on network change
    #[allow(unused)]
    entry_peer: WgPeer,

    /// Exit peer configuration.
    /// todo: use it later to update peers on network change
    #[allow(unused)]
    exit_peer: WgPeer,

    /// Interface for interacting with the iOS tunnel provider.
    #[allow(unused)]
    tun_provider: Arc<dyn OSTunProvider>,
}

impl TwoHopTunnel {
    /// Fetch wg gateways and start the tunnel.
    pub async fn start(
        nym_vpn: &mut NymVpn<WireguardVpn>,
        mixnet_client: SharedMixnetClient,
        task_manager: &mut TaskManager,
        gateway_directory_client: GatewayClient,
        auth_addresses: AuthAddresses,
    ) -> Result<Self> {
        let bandwidth_controller =
            BandwidthController::new(mixnet_client.clone(), task_manager.subscribe());
        tokio::spawn(bandwidth_controller.run());

        let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
            (auth_addresses.entry().0, auth_addresses.exit().0)
        else {
            return Err(crate::Error::AuthenticationNotPossible(
                auth_addresses.to_string(),
            ));
        };
        let auth_client = AuthClient::new_from_inner(mixnet_client.inner()).await;
        tracing::info!("Created wg gateway clients");

        let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
            &nym_vpn.generic_config.data_path,
            auth_client.clone(),
            entry_auth_recipient,
        );

        let mut wg_exit_gateway_client = WgGatewayClient::new_exit(
            &nym_vpn.generic_config.data_path,
            auth_client.clone(),
            exit_auth_recipient,
        );

        let entry_gateway_data = gateway::register_client_pubkey(
            &gateway_directory_client,
            &mut wg_entry_gateway_client,
        )
        .await?;
        let exit_gateway_data =
            gateway::register_client_pubkey(&gateway_directory_client, &mut wg_exit_gateway_client)
                .await?;

        // Configure tun interface & DNS
        let tun_provider = nym_vpn.ios_tun_provider.clone();
        let tunnel_settings = tunnel_settings::create(
            vec![exit_gateway_data.private_ip],
            entry_gateway_data.endpoint.ip(),
            crate::DEFAULT_DNS_SERVERS.to_vec(),
            EXIT_MTU,
        );
        nym_vpn
            .ios_tun_provider
            .set_tunnel_network_settings(tunnel_settings)
            .await?;

        let entry_node_config = WgNodeConfig::new(
            wg_entry_gateway_client.keypair().private_key(),
            entry_gateway_data,
        );
        let exit_node_config = WgNodeConfig::new(
            wg_exit_gateway_client.keypair().private_key(),
            exit_gateway_data,
        );

        Self::start_wg_tunnel(entry_node_config, exit_node_config, tun_provider)
    }

    /// Start two-hop wg tunnel given entry and exit nodes.
    fn start_wg_tunnel(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        tun_provider: Arc<dyn OSTunProvider>,
    ) -> Result<Self> {
        // Local endpoint that will forward exit traffic over entry tunnel
        let udp_forwarder_endpoint = SocketAddr::new(
            if exit_node_config.peer.endpoint.is_ipv4() {
                IpAddr::V4(Ipv4Addr::LOCALHOST)
            } else {
                IpAddr::V6(Ipv6Addr::LOCALHOST)
            },
            UDP_FORWARDER_PORT,
        );
        let exit_endpoint = exit_node_config.peer.endpoint;

        // Save original peer endpoints so that we can re-resolve them with DNS64 when device switches networks.
        let orig_entry_peer = entry_node_config.peer.clone();
        let orig_exit_peer = exit_node_config.peer.clone();

        // Transform wg config structs into what nym-wg-go expects.
        let mut entry_wg_config =
            entry_node_config.into_wg_entry_config(crate::DEFAULT_DNS_SERVERS.to_vec());
        let mut exit_wg_config = exit_node_config.into_wg_exit_config(udp_forwarder_endpoint);

        tracing::info!("Entry wireguard config: \n{}", entry_wg_config);
        tracing::info!("Exit wireguard config: \n{}", exit_wg_config);
        tracing::info!("UDP forwarder listener: \n{}", udp_forwarder_endpoint);
        tracing::info!("UDP forwarder exit endpoint: \n{}", exit_endpoint);

        // Resolve peer IP addresses with DNS64.
        dns64::resolve_peers(&mut entry_wg_config.peers)?;
        // todo: we probably don't have to do that for netstack connections!
        dns64::resolve_peers(&mut exit_wg_config.peers)?;

        // Obtain tunnel file descriptor and interface name.
        let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
        tracing::debug!("Found tunnel fd: {}", tun_fd);

        let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
        tracing::debug!("Tunnel interface name: {}", tun_name);

        // Create netstack wg connected to the entry node.
        let mut entry_tunnel = netstack::Tunnel::start(entry_wg_config, |s| {
            tracing::debug!(name = "wg-netstack", "{}", s);
        })
        .map_err(Error::Tunnel)?;

        // Open connection to the exit node via entry node.
        let exit_connection = entry_tunnel
            .open_connection(UDP_FORWARDER_PORT, EXIT_WG_CLIENT_PORT, exit_endpoint)
            .map_err(Error::Tunnel)?;

        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let exit_tunnel = wireguard_go::Tunnel::start(exit_wg_config, tun_fd, |s| {
            tracing::debug!(name = "wg-go", "{}", s);
        })
        .map_err(Error::Tunnel)?;

        Ok(Self {
            entry: entry_tunnel,
            exit: exit_tunnel,
            exit_connection: exit_connection,
            entry_peer: orig_entry_peer,
            exit_peer: orig_exit_peer,
            tun_provider,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to locate tun fd")]
    CannotLocateTunFd,

    #[error("Failed to obtain tun interface name")]
    ObtainTunName,

    #[error("Tunnel failure")]
    Tunnel(nym_wg_go::Error),

    #[error("Failed to resolve {} (error code: {})", addr, code)]
    DnsLookup { code: i32, addr: SocketAddr },

    #[error("Failed to parse addrinfo")]
    ParseAddrInfo(std::io::Error),

    #[error("DNS lookup has seemingly succeeded without any results")]
    EmptyDnsLookupResult,

    #[error("Failed to convert port to C-string")]
    ConvertPortToCstr,

    #[error("Failed to convert ip to C-string")]
    ConvertIpToCstr,
}

struct WgNodeConfig {
    /// Interface configuration
    interface: WgInterface,

    /// Peer configuration
    peer: WgPeer,
}

struct WgInterface {
    /// Private key used by wg client.
    private_key: PrivateKey,

    /// Address assigned on wg interface.
    address: IpAddr,
}

#[derive(Debug, Clone)]
struct WgPeer {
    /// Gateway public key.
    public_key: PublicKey,

    /// Gateway endpoint
    endpoint: SocketAddr,
}

impl WgNodeConfig {
    fn new(
        private_key: &nym_crypto::asymmetric::encryption::PrivateKey,
        gateway_data: GatewayData,
    ) -> Self {
        Self {
            interface: WgInterface {
                address: gateway_data.private_ip,
                private_key: PrivateKey::from(private_key.to_bytes()),
            },
            peer: WgPeer {
                public_key: PublicKey::from(*gateway_data.public_key.as_bytes()),
                endpoint: gateway_data.endpoint,
            },
        }
    }

    /// Returns entry config for the WireGuard.
    fn into_wg_entry_config(self, dns_addrs: Vec<IpAddr>) -> netstack::Config {
        netstack::Config {
            interface: netstack::InterfaceConfig {
                private_key: self.interface.private_key,
                local_addrs: vec![self.interface.address],
                dns_addrs,
                mtu: ENTRY_MTU,
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

    /// Returns exit config for the WireGuard, rewriting the endpoint to point at local UDP forwarder.
    fn into_wg_exit_config(self, udp_forwarder_endpoint: SocketAddr) -> wireguard_go::Config {
        wireguard_go::Config {
            interface: wireguard_go::InterfaceConfig {
                listen_port: Some(EXIT_WG_CLIENT_PORT),
                private_key: self.interface.private_key,
                mtu: EXIT_MTU,
            },
            peers: vec![PeerConfig {
                public_key: self.peer.public_key,
                preshared_key: None,
                endpoint: udp_forwarder_endpoint,
                allowed_ips: vec!["0.0.0.0/0".parse().unwrap(), "::/0".parse().unwrap()],
            }],
        }
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: tunnel_settings::TunnelNetworkSettings,
    ) -> std::result::Result<(), crate::platform::error::FFIError>;
}
