use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use ipnetwork::IpNetwork;
use crate::wg_config::{WgInterface, WgNodeConfig, WgPeer};

/// Minimum IPv6 MTU that the hosts should be ready to accept.
const MIN_IPV6_MTU: u16 = 1280;

/// WG tunnel overhead (IPv6)
const WG_TUNNEL_OVERHEAD: u16 = 80;

/// Local port used for accepting exit traffic.
const UDP_FORWARDER_PORT: u16 = 34001;

/// Local port used by exit tunnel when sending traffic to the udp forwarder.
const EXIT_WG_CLIENT_PORT: u16 = 54001;

/// A struct that holds all configuration needed to setup the tunnels, tun device and forwarder.
#[derive(Debug)]
pub struct TwoHopConfig {
    /// Entry configuration applied to netstack based WireGuard tunnel.
    pub entry: WgNodeConfig,

    /// Exit configuration applied to wireguard-go attached to tun device.
    pub exit: WgNodeConfig,

    /// Configuration for UDP forwader that's used for wrapping tunnel in tunnel.
    pub forwarder: WgForwarderConfig,

    /// Tun device configuration.
    pub tun: TunConfig,
}

impl TwoHopConfig {
    /// Create new two-hop configuration given two individual WireGuard configurations.
    pub fn new(entry: WgNodeConfig, exit: WgNodeConfig) -> Self {
        // Ensure that exit instance of wg attached on tun interface, uses a fixed port number
        // to initiate connection to the udp forwarder, because it ignores traffic from other ports.
        let client_port = exit.interface.listen_port.unwrap_or(EXIT_WG_CLIENT_PORT);

        let forwarder_config = WgForwarderConfig {
            // Local endpoint that will forward exit traffic over entry tunnel
            listen_endpoint: SocketAddr::new(
                if exit.peer.endpoint.is_ipv4() {
                    IpAddr::V4(Ipv4Addr::LOCALHOST)
                } else {
                    IpAddr::V6(Ipv6Addr::LOCALHOST)
                },
                UDP_FORWARDER_PORT,
            ),
            exit_endpoint: exit.peer.endpoint,
            client_port,
        };

        // Since we collect the exit traffic on tun, the tun's mtu must be lesser than entry mtu.
        let exit_mtu = MIN_IPV6_MTU;
        let entry_mtu = exit_mtu + WG_TUNNEL_OVERHEAD;

        let tun_config = TunConfig {
            addresses: exit.interface.addresses.clone(),
            dns: exit.interface.dns.clone(),
            mtu: exit_mtu,
        };

        Self {
            entry: WgNodeConfig {
                interface: WgInterface {
                    mtu: entry_mtu,
                    ..entry.interface
                },
                peer: entry.peer,
            },
            exit: WgNodeConfig {
                interface: WgInterface {
                    listen_port: Some(client_port),
                    mtu: exit_mtu,
                    ..exit.interface
                },
                peer: WgPeer {
                    endpoint: forwarder_config.listen_endpoint,
                    ..exit.peer
                },
            },
            forwarder: forwarder_config,
            tun: tun_config,
        }
    }
}

#[derive(Debug)]
pub struct WgForwarderConfig {
    /// Local endpoint for collecting exit wg traffic.
    pub listen_endpoint: SocketAddr,

    /// Actual exit endpoint.
    pub exit_endpoint: SocketAddr,

    /// Client port from which the connection will be established to the listen endpoint.
    /// Specified as listen_port in wg config.
    pub client_port: u16,
}

#[derive(Debug)]
pub struct TunConfig {
    pub addresses: Vec<IpNetwork>,
    pub dns: Vec<IpAddr>,
    pub mtu: u16,
}
