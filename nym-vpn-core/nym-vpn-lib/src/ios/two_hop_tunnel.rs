use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use ipnetwork::IpNetwork;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{
    default_path_observer::DefaultPathObserver, dns64, tun, tunnel_settings, Error, OSDefaultPath,
    OSDefaultPathObserver, OSTunProvider, Result,
};
use nym_wg_go::{netstack, wireguard_go, PeerConfig, PrivateKey, PublicKey};

/// Entry tunnel MTU.
const ENTRY_MTU: u16 = 1420;

/// Exit tunnel MTU.
const EXIT_MTU: u16 = 1340;

/// Minimum IPv6 MTU that the hosts should be ready to accept.
const MIN_IPv6_MTU: u16 = 1280;

/// WG tunnel overhead (IPv6)
const WG_TUNNEL_OVERHEAD: u16 = 80;

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
    entry: Option<netstack::Tunnel>,

    /// Exit node tunnel
    #[allow(unused)]
    exit: wireguard_go::Tunnel,

    /// UDP connection over the entry tunnel, towards exit node.
    #[allow(unused)]
    exit_connection: Option<netstack::TunnelConnection>,

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

    /// An object observing the default path.
    default_path_observer: Arc<dyn OSDefaultPathObserver>,

    /// Default path receiver.
    default_path_rx: mpsc::UnboundedReceiver<OSDefaultPath>,

    /// Cancellation token.
    shutdown_token: CancellationToken,
}

impl TwoHopTunnel {
    /// Fetch wg gateways and start the tunnel.
    pub async fn start(
        os_tun_provider: Arc<dyn OSTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        // Configure tun interface & DNS
        let tunnel_settings = tunnel_settings::create(
            vec!["10.71.122.208".parse().expect("iface addr")],
            vec!["10.64.0.1".parse().unwrap()], // crate::DEFAULT_DNS_SERVERS.to_vec(),
            EXIT_MTU,
        );
        os_tun_provider
            .set_tunnel_network_settings(tunnel_settings)
            .await
            .map_err(Error::SetNetworkSettings)?;

        let Some(entry_priv_key) =
            nym_wg_go::PrivateKey::from_base64("MMLgfpAzr5RsVoK6UQezipadSx/QhjDDBM86MqOTolA=")
        else {
            tracing::error!("Failed to decode entry priv key.");
            return Err(Error::InvalidKey);
        };

        let Some(entry_pub_key) =
            nym_wg_go::PublicKey::from_base64("TNrdH73p6h2EfeXxUiLOCOWHcjmjoslLxZptZpIPQXU=")
        else {
            tracing::error!("Failed to decode entry pub key.");
            return Err(Error::InvalidKey);
        };

        let entry_node_config = WgNodeConfig {
            interface: WgInterface {
                private_key: entry_priv_key,
                address: "10.71.122.208".parse().expect("entry iface addr"),
                dns: crate::DEFAULT_DNS_SERVERS.to_vec(),
            },
            peer: WgPeer {
                public_key: entry_pub_key,
                endpoint: "146.70.116.98:12912".parse().expect("entry peer endpoint"),
            },
        };

        let Some(exit_priv_key) =
            nym_wg_go::PrivateKey::from_base64("MMLgfpAzr5RsVoK6UQezipadSx/QhjDDBM86MqOTolA=")
        else {
            tracing::error!("Failed to decode exit priv key.");
            return Err(Error::InvalidKey);
        };
        let Some(exit_pub_key) =
            nym_wg_go::PublicKey::from_base64("TNrdH73p6h2EfeXxUiLOCOWHcjmjoslLxZptZpIPQXU=")
        else {
            tracing::error!("Failed to decode exit pub key.");
            return Err(Error::InvalidKey);
        };

        let exit_node_config = WgNodeConfig {
            interface: WgInterface {
                private_key: exit_priv_key,
                address: "10.71.122.208".parse().expect("exit iface addr"),
                dns: crate::DEFAULT_DNS_SERVERS.to_vec(),
            },
            peer: WgPeer {
                public_key: exit_pub_key,
                endpoint: "146.70.116.98:12912".parse().expect("exit peer endpoint"),
            },
        };

        tracing::debug!("WG configuration is ready. Proceeding to start the wg clients.");

        Self::start_wg_tunnel(
            entry_node_config,
            exit_node_config,
            os_tun_provider,
            shutdown_token,
        )
        .await
    }

    async fn start_wg_tunnel(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        tun_provider: Arc<dyn OSTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        // Save original peer endpoints so that we can re-resolve them with DNS64 when device switches networks.
        let orig_entry_peer = entry_node_config.peer.clone();
        let orig_exit_peer = exit_node_config.peer.clone();

        // Transform wg config structs into what nym-wg-go expects.
        let mut entry_wg_config = entry_node_config.into_wg_go_config();

        tracing::info!("Entry wireguard config: \n{}", entry_wg_config);

        // Resolve peer IP addresses with DNS64.
        dns64::resolve_peers(&mut entry_wg_config.peers)?;

        // Obtain tunnel file descriptor and interface name.
        let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
        tracing::debug!("Found tunnel fd: {}", tun_fd);

        let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
        tracing::debug!("Tunnel interface name: {}", tun_name);

        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let entry_tunnel = wireguard_go::Tunnel::start(entry_wg_config, tun_fd, |s| {
            tracing::debug!(name = "wg-go", "{}", s);
        })
        .map_err(Error::Tunnel)?;

        let (default_path_tx, default_path_rx) = mpsc::unbounded_channel();
        let default_path_observer = Arc::new(DefaultPathObserver::new(default_path_tx));

        tun_provider
            .set_default_path_observer(Some(default_path_observer.clone()))
            .map_err(Error::SetDefaultPathObserver)?;

        let tunnel = Self {
            entry: None,
            exit: entry_tunnel,
            exit_connection: None,
            entry_peer: orig_entry_peer,
            exit_peer: orig_exit_peer,
            tun_provider,
            default_path_observer,
            default_path_rx,
            shutdown_token,
        };

        tunnel.run().await;

        Ok(())
    }

    /// Start two-hop wg tunnel given entry and exit nodes.
    fn start_wg_tunnel2(
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
        let exit_wg_config = exit_node_config.into_wg_exit_config(udp_forwarder_endpoint);

        tracing::info!("Entry wireguard config: \n{}", entry_wg_config);
        tracing::info!("Exit wireguard config: \n{}", exit_wg_config);
        tracing::info!("UDP forwarder listener: \n{}", udp_forwarder_endpoint);
        tracing::info!("UDP forwarder exit endpoint: \n{}", exit_endpoint);

        // Resolve peer IP addresses with DNS64.
        dns64::resolve_peers(&mut entry_wg_config.peers)?;

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
            entry: Some(entry_tunnel),
            exit: exit_tunnel,
            exit_connection: Some(exit_connection),
            entry_peer: orig_entry_peer,
            exit_peer: orig_exit_peer,
            tun_provider,
            default_path_observer: todo!(),
            shutdown_token: todo!(),
            default_path_rx: todo!(),
        })
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    break;
                },
                new_path = self.default_path_rx.recv() => {
                    // todo: reresolve endpoints
                    // todo: update wg config
                    if let Some(new_path) = new_path {
                        tracing::debug!("New default path: {:?}", new_path);
                        self.exit.bump_sockets();
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct WgTunConfig {
    pub addresses: Vec<IpNetwork>,
    pub dns: Vec<IpAddr>,
    pub mtu: u16,
}

#[derive(Debug)]
struct WgTwoHopConfig {
    entry: WgNodeConfig,
    exit: WgNodeConfig,
    forwarder: WgForwarderConfig,
    tun_config: WgTunConfig,
}

impl WgTwoHopConfig {
    pub fn new(mut entry: WgNodeConfig, mut exit: WgNodeConfig) -> WgTwoHopConfig {
        // Since we collect the exit traffic on tun, the tun mtu must be smaller than entry mtu.
        let tun_mtu = MIN_IPv6_MTU;
        let entry_mtu = tun_mtu + WG_TUNNEL_OVERHEAD;

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
            client_port: EXIT_WG_CLIENT_PORT,
        };

        exit.peer.endpoint = forwarder_config.listen_endpoint;

        let tun_config = WgTunConfig {
            addresses: vec![exit.interface.address],
            dns: exit.interface.dns.clone(),
            mtu: tun_mtu,
        };

        WgTwoHopConfig {
            entry,
            exit,
            forwarder: forwarder_config,
            tun_config,
        }
    }
}
#[derive(Debug)]
struct WgForwarderConfig {
    /// Local endpoint for collecting exit wg traffic.
    pub listen_endpoint: SocketAddr,

    /// Actual exit endpoint.
    pub exit_endpoint: SocketAddr,

    /// Client port from which the connection will be established to the listen endpoint.
    /// Specified as listen_port in wg config.
    pub client_port: u16,
}

#[derive(Debug)]
struct WgNodeConfig {
    /// Interface configuration
    interface: WgInterface,

    /// Peer configuration
    peer: WgPeer,
}

#[derive(Debug)]
struct WgInterface {
    /// Private key used by wg client.
    private_key: PrivateKey,

    /// Address assigned on wg interface.
    address: IpNetwork,

    /// Interface DNS addresses.
    dns: Vec<IpAddr>,
}

#[derive(Debug, Clone)]
struct WgPeer {
    /// Gateway public key.
    public_key: PublicKey,

    /// Gateway endpoint
    endpoint: SocketAddr,
}

impl WgNodeConfig {
    fn new(interface: WgInterface, peer: WgPeer) -> Self {
        Self { interface, peer }
    }

    fn with_gateway_data(
        private_key: &nym_crypto::asymmetric::encryption::PrivateKey,
        gateway_data: crate::wg_gateway_client::GatewayData,
    ) -> Self {
        Self {
            interface: WgInterface {
                address: if gateway_data.private_ip.is_ipv4() {
                    IpNetwork::new(gateway_data.private_ip, 32).expect("private_ip v4/32")
                } else {
                    IpNetwork::new(gateway_data.private_ip, 128).expect("private_ip v6/128")
                },
                private_key: PrivateKey::from(private_key.to_bytes()),
                dns: vec![],
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
                local_addrs: vec![self.interface.address.ip()],
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

    fn into_wg_go_config(self) -> wireguard_go::Config {
        wireguard_go::Config {
            interface: wireguard_go::InterfaceConfig {
                listen_port: None,
                private_key: self.interface.private_key,
                mtu: EXIT_MTU,
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
