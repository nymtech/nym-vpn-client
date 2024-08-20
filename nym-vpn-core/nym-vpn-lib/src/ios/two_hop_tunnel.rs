use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{
    default_path_observer::DefaultPathObserver,
    dns64, tun, tunnel_settings,
    two_hop_config::WgTwoHopConfig,
    wg_config::{WgInterface, WgNodeConfig, WgPeer},
    Error, OSDefaultPath, OSDefaultPathObserver, OSTunProvider, Result,
};
use nym_wg_go::{netstack, wireguard_go, PeerEndpointUpdate};

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
                listen_port: None,
                private_key: entry_priv_key,
                addresses: vec!["10.71.122.208/32"
                    .parse()
                    .expect("failed to parse iface addr")],
                dns: crate::DEFAULT_DNS_SERVERS.to_vec(),
                mtu: 1280,
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
                listen_port: None,
                private_key: exit_priv_key,
                addresses: vec!["10.71.122.208/32".parse().expect("exit iface addr")],
                dns: crate::DEFAULT_DNS_SERVERS.to_vec(),
                mtu: 1280,
            },
            peer: WgPeer {
                public_key: exit_pub_key,
                endpoint: "146.70.116.98:12912".parse().expect("exit peer endpoint"),
            },
        };

        tracing::debug!("WG configuration is ready. Proceeding to start the wg clients.");

        // Configure tun interface & DNS
        let tunnel_settings = tunnel_settings::create(
            exit_node_config.interface.addresses.clone(),
            vec!["10.64.0.1".parse().unwrap()], // crate::DEFAULT_DNS_SERVERS.to_vec(),
            super::two_hop_config::MIN_IPV6_MTU,
        );
        os_tun_provider
            .set_tunnel_network_settings(tunnel_settings)
            .await
            .map_err(Error::SetNetworkSettings)?;

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
        let mut entry_wg_config = entry_node_config.into_wireguard_config();

        tracing::info!("Entry wireguard config: \n{:#?}", entry_wg_config);

        // Re-resolve peers with DNS64.
        for peer in entry_wg_config.peers.iter_mut() {
            peer.endpoint = dns64::reresolve_endpoint(peer.endpoint)?;
        }

        // Obtain tunnel file descriptor and interface name.
        let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
        tracing::debug!("Found tunnel fd: {}", tun_fd);

        let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
        tracing::debug!("Tunnel interface name: {}", tun_name);

        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let entry_tunnel = wireguard_go::Tunnel::start(entry_wg_config, tun_fd, |s| {
            tracing::debug!(name = "wg-go", "{}", s);
        })?;

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
    async fn start_wg_tunnel2(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        tun_provider: Arc<dyn OSTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        // Save original peer endpoints so that we can re-resolve them with DNS64 when device switches networks.
        let orig_entry_peer = entry_node_config.peer.clone();
        let orig_exit_peer = exit_node_config.peer.clone();

        let two_hop_config = WgTwoHopConfig::new(entry_node_config, exit_node_config);

        tracing::info!("Two-hop config: {:#?}", two_hop_config);

        // Transform wg config structs into what nym-wg-go expects.
        let mut entry_wg_config = two_hop_config.entry.into_netstack_config();
        let exit_wg_config = two_hop_config.exit.into_wireguard_config();

        // Re-resolve peers with DNS64.
        for peer in entry_wg_config.peers.iter_mut() {
            peer.endpoint = dns64::reresolve_endpoint(peer.endpoint)?;
        }

        // Obtain tunnel file descriptor and interface name.
        let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
        tracing::debug!("Found tunnel fd: {}", tun_fd);

        let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
        tracing::debug!("Tunnel interface name: {}", tun_name);

        // Create netstack wg connected to the entry node.
        let mut entry_tunnel = netstack::Tunnel::start(entry_wg_config, |s| {
            tracing::debug!(name = "wg-netstack", "{}", s);
        })?;

        // Open connection to the exit node via entry node.
        let exit_connection = entry_tunnel.open_connection(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
        )?;

        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let exit_tunnel = wireguard_go::Tunnel::start(exit_wg_config, tun_fd, |s| {
            tracing::debug!(name = "wg-go", "{}", s);
        })?;

        let (default_path_tx, default_path_rx) = mpsc::unbounded_channel();
        let default_path_observer = Arc::new(DefaultPathObserver::new(default_path_tx));

        tun_provider
            .set_default_path_observer(Some(default_path_observer.clone()))
            .map_err(Error::SetDefaultPathObserver)?;

        let two_hop_tunnel = Self {
            entry: Some(entry_tunnel),
            exit: exit_tunnel,
            exit_connection: Some(exit_connection),
            entry_peer: orig_entry_peer,
            exit_peer: orig_exit_peer,
            tun_provider,
            default_path_observer,
            shutdown_token,
            default_path_rx,
        };

        two_hop_tunnel.run().await;

        Ok(())
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    break;
                },
                Some(new_path) = self.default_path_rx.recv() => {
                    self.on_network_path_change(new_path);
                }
                else => {
                    break;
                }
            }
        }
    }

    fn on_network_path_change(&mut self, new_path: OSDefaultPath) {
        tracing::debug!("New default path: {:?}", new_path);

        // Update peers, re-resolving peers with DNS64.
        if let Err(e) = self.update_peers() {
            tracing::error!("Failed to update peers on network change: {}", e);
        }

        // Rebind wireguard-go on tun device.
        self.exit.bump_sockets();
    }

    fn update_peers(&mut self) -> Result<()> {
        let reresolved_endpoint = dns64::reresolve_endpoint(self.entry_peer.endpoint)?;
        let peer_update = PeerEndpointUpdate {
            endpoint: reresolved_endpoint,
            public_key: self.entry_peer.public_key.clone(),
        };

        self.exit.update_peers(&[peer_update])?;
        self.exit.disable_roaming();

        Ok(())
    }
}
