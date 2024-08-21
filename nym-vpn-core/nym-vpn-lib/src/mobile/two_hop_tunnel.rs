use std::sync::Arc;
use log::info;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{
    default_path_observer::DefaultPathObserver,
    dns64, tun, tunnel_settings,
    two_hop_config::TwoHopConfig,
    wg_config::{WgInterface, WgNodeConfig, WgPeer},
    Error, OSDefaultPath, OSDefaultPathObserver, OSTunProvider, Result,
};
use nym_wg_go::{netstack, wireguard_go, PeerEndpointUpdate};
use crate::platform::android::AndroidTunProvider;

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
    entry: netstack::Tunnel,

    /// Exit node tunnel
    #[allow(unused)]
    exit: wireguard_go::Tunnel,

    /// UDP connection over the entry tunnel, towards exit node.
    #[allow(unused)]
    exit_connection: Option<netstack::TunnelConnection>,

    /// Entry peer configuration.
    entry_peer: WgPeer,

    /// Exit peer configuration.
    #[allow(unused)]
    exit_peer: WgPeer,

    /// Interface for interacting with the iOS tunnel provider.
    #[allow(unused)]
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,

    /// Interface for interacting with the Android tunnel provider.
    #[allow(unused)]
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,

    /// An object observing the default path.
    #[allow(unused)]
    default_path_observer: Arc<dyn OSDefaultPathObserver>,

    /// Default path receiver.
    default_path_rx: mpsc::UnboundedReceiver<OSDefaultPath>,

    /// Cancellation token.
    shutdown_token: CancellationToken,
}

impl TwoHopTunnel {
    /// Fetch wg gateways and start the tunnel.
    pub async fn start(
        #[cfg(target_os = "ios")]
        tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")]
        tun_provider: Arc<dyn AndroidTunProvider>,
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
            nym_wg_go::PrivateKey::from_base64("2GJR/sWv5YuutftnnxVr3UI6DjSjAdnWjvMtPkBtKn4=")
        else {
            tracing::error!("Failed to decode exit priv key.");
            return Err(Error::InvalidKey);
        };
        let Some(exit_pub_key) =
            nym_wg_go::PublicKey::from_base64("GE2WP6hmwVggSvGVWLgq2L10T3WM2VspnUptK5F4B0U=")
        else {
            tracing::error!("Failed to decode exit pub key.");
            return Err(Error::InvalidKey);
        };

        let exit_node_config = WgNodeConfig {
            interface: WgInterface {
                listen_port: None,
                private_key: exit_priv_key,
                addresses: vec!["10.64.93.204/32".parse().expect("exit iface addr")],
                dns: crate::DEFAULT_DNS_SERVERS.to_vec(),
                mtu: 1280,
            },
            peer: WgPeer {
                public_key: exit_pub_key,
                endpoint: "91.90.123.2:443".parse().expect("exit peer endpoint"),
            },
        };

        Self::start_inner(
            entry_node_config,
            exit_node_config,
            tun_provider,
            shutdown_token,
        )
        .await
    }

    /// Start two-hop wg tunnel given entry and exit nodes.
    async fn start_inner(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        #[cfg(target_os = "ios")]
        tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")]
        tun_provider: Arc<dyn AndroidTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        // Save original peer endpoints so that we can re-resolve them with DNS64 when device switches networks.
        let orig_entry_peer = entry_node_config.peer.clone();
        let orig_exit_peer = exit_node_config.peer.clone();

        let two_hop_config = TwoHopConfig::new(entry_node_config, exit_node_config);

        tracing::info!("Two-hop entry: {:#?}", two_hop_config.entry);
        tracing::info!("Two-hop exit: {:#?}", two_hop_config.exit);
        tracing::info!("Two-hop tun: {:#?}", two_hop_config.tun);
        tracing::info!("Two-hop forwarder: {:#?}", two_hop_config.forwarder);

        // Transform wg config structs into what nym-wg-go expects.
        let mut entry_wg_config = two_hop_config.entry.into_netstack_config();
        let exit_wg_config = two_hop_config.exit.into_wireguard_config();

        // Re-resolve peers with DNS64.
        for peer in entry_wg_config.peers.iter_mut() {
            peer.endpoint = dns64::reresolve_endpoint(peer.endpoint)?;
        }

        // Obtain tunnel file descriptor and interface name.
        #[cfg(target_os = "ios")]
        let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
        #[cfg(target_os = "ios")]
        tracing::debug!("Found tunnel fd: {}", tun_fd);

        #[cfg(target_os = "ios")]
        let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
        #[cfg(target_os = "ios")]
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

        // Configure tun, dns and routing
        let tunnel_settings = tunnel_settings::create(
            two_hop_config.tun.addresses,
            two_hop_config.tun.dns,
            two_hop_config.tun.mtu,
        );

        #[cfg(target_os = "android")]
        let tun_fd = tun_provider.configure_wg(tunnel_settings).map_err(|_| {
            Error::CannotLocateTunFd
        })?;

        // #[cfg(target_os = "android")]
        // {
        //     info!("Bypassing sockets");
        //     tun_provider.bypass(entry_tunnel.get_socket_v6(tun_fd));
        //     tun_provider.bypass(entry_tunnel.get_socket_v4(tun_fd));
        //
        // }


        #[cfg(target_os = "android")]
        if tun_fd == -1 { return Err(Error::CannotLocateTunFd); }
        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let exit_tunnel = wireguard_go::Tunnel::start(exit_wg_config, tun_fd, |s| {
            tracing::debug!(name = "wg-go", "{}", s);
        })?;

        #[cfg(target_os = "ios")]
        tun_provider
            .set_tunnel_network_settings(tunnel_settings)
            .await
            .map_err(Error::SetNetworkSettings)?;

        // Setup default path observer.
        let (default_path_tx, default_path_rx) = mpsc::unbounded_channel();
        let default_path_observer = Arc::new(DefaultPathObserver::new(default_path_tx));

        #[cfg(target_os = "ios")]
        tun_provider
            .set_default_path_observer(Some(default_path_observer.clone()))
            .map_err(Error::SetDefaultPathObserver)?;

        let two_hop_tunnel = Self {
            entry: entry_tunnel,
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

        // Depending on the network device is connected to, we may need to re-resolve the IP addresses.
        // For instance when device connects to IPv4-only server from IPv6-only network,
        // it needs to use an IPv4-mapped address, which can be received by re-resolving
        // the original peer IP.
        if let Err(e) = self.update_peers() {
            tracing::error!("Failed to update peers on network change: {}", e);
        }

        // Rebind wireguard-go on tun device.
        #[cfg(target_os = "ios")]
        self.exit.bump_sockets();
    }

    fn update_peers(&mut self) -> Result<()> {
        let reresolved_endpoint = dns64::reresolve_endpoint(self.entry_peer.endpoint)?;
        let peer_update = PeerEndpointUpdate {
            endpoint: reresolved_endpoint,
            public_key: self.entry_peer.public_key.clone(),
        };

        // Update wireguard-go configuration with re-resolved peer endpoints.
        self.exit.update_peers(&[peer_update])?;

        // wireguard-go resets the roaming flag when updating peers, this call fixes this.
        #[cfg(target_os = "ios")]
        self.exit.disable_roaming();

        Ok(())
    }
}
