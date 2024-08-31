use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
#[cfg(target_os = "ios")]
use super::ios::{
    default_path_observer::{DefaultPathObserver, DefaultPathReceiver, OSDefaultPath},
    dns64::Dns64Resolution,
    tun,
    tun_provider::OSTunProvider,
};
use super::tunnel_settings::TunnelSettings;

use super::{dns64, two_hop_config::TwoHopConfig, wg_config::{WgNodeConfig, WgPeer}, Error, Result};

use nym_wg_go::{netstack, wireguard_go};
use crate::mobile::dns64::Dns64Resolution;
#[cfg(target_os = "android")]
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

    /// Cancellation token.
    shutdown_token: CancellationToken,

    /// Entry peer configuration.
    entry_peer: WgPeer,

    /// Interface for interacting with the iOS tunnel provider.
    #[cfg(target_os = "android")]
    #[allow(unused)]
    tun_provider: Arc<dyn AndroidTunProvider>,

    /// Interface for interacting with the iOS tunnel provider.
    #[cfg(target_os = "ios")]
    #[allow(unused)]
    tun_provider: Arc<dyn OSTunProvider>,

    #[cfg(target_os = "ios")]
    /// A conduit for receiving default path updates.
    default_path_receiver: DefaultPathReceiver,
}

impl TwoHopTunnel {
    /// Start two-hop wg tunnel given entry and exit nodes.
    pub async fn start(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        // Save entry peer so that we can re-resolve it and update wg config on network changes.
        let orig_entry_peer = entry_node_config.peer.clone();

        let mut two_hop_config = TwoHopConfig::new(entry_node_config, exit_node_config);

        tracing::info!("Two-hop entry: {:#?}", two_hop_config.entry);
        tracing::info!("Two-hop exit: {:#?}", two_hop_config.exit);
        tracing::info!("Two-hop tun: {:#?}", two_hop_config.tun);
        tracing::info!("Two-hop forwarder: {:#?}", two_hop_config.forwarder);


        // iOS does not perform dns64 resolution by default. Do that manually.
        #[cfg(target_os = "ios")]
        two_hop_config.entry.peer.resolve_in_place()?;

        // Obtain tunnel file descriptor and interface name.
        #[cfg(target_os = "ios")]
        let tun_fd = {
            let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
            tracing::debug!("Found tunnel fd: {}", tun_fd);
            tun_fd
        };


        #[cfg(target_os = "ios")]
        let tun_name = {
            let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
            tracing::debug!("Tunnel interface name: {}", tun_name);
            tun_name
        };

        let tunnel_settings = TunnelSettings {
            interface_addresses: two_hop_config.tun.addresses,
            dns_servers: two_hop_config.tun.dns,
            remote_addresses: vec![two_hop_config.entry.peer.endpoint.ip()],
            mtu: two_hop_config.tun.mtu,
        };

        // Configure tun, dns and routing
        #[cfg(target_os = "ios")]
        {
            tun_provider
                .set_tunnel_network_settings(tunnel_settings.into_tunnel_network_settings())
                .await
                .map_err(Error::SetNetworkSettings)?;
        }

        // Transform wg config structs into what nym-wg-go expects.
        let mut entry_wg_config = two_hop_config.entry.into_netstack_config();
        let exit_wg_config = two_hop_config.exit.into_wireguard_config();

        // Re-resolve peers with DNS64.
        for peer in entry_wg_config.peers.iter_mut() {
            peer.endpoint = dns64::reresolve_endpoint(peer.endpoint).map_err(|e| {
                Error::DnsResolution(e)
            })?;
        }

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

        #[cfg(target_os = "android")]
        let tun_fd = {
            tun_provider
                .configure_tunnel(tunnel_settings.into_tunnel_network_settings())
                .map_err(Error::SetNetworkSettings)?
        };

        #[cfg(target_os = "android")]
        if tun_fd == -1 { return Err(Error::CannotLocateTunFd); }


        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let exit_tunnel = wireguard_go::Tunnel::start(exit_wg_config, tun_fd, |s| {
            tracing::debug!(name = "wg-go", "{}", s);
        })?;

        // Setup default path observer.
        #[cfg(target_os = "ios")]
        let default_path_receiver = Self::add_default_path_observer(tun_provider.clone())?;

        let two_hop_tunnel = Self {
            entry: entry_tunnel,
            exit: exit_tunnel,
            exit_connection: Some(exit_connection),
            shutdown_token,
            entry_peer: orig_entry_peer,
            #[cfg(target_os = "android")]
            tun_provider,
            #[cfg(target_os = "ios")]
            tun_provider,
            #[cfg(target_os = "ios")]
            default_path_receiver,
        };

        two_hop_tunnel.run().await;

        Ok(())
    }

    #[cfg(target_os = "android")]
    async fn run(self) {
        self.shutdown_token.cancelled().await;
        tracing::debug!("Received shutdown.");
    }

    #[cfg(target_os = "ios")]
    async fn run(mut self) {
        loop {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::debug!("Received shutdown.");
                    break;
                },
                Some(new_path) = self.default_path_receiver.recv() => {
                    self.on_network_path_change(new_path);
                },
                else => {
                    tracing::debug!("Default path channel is closed. Proceeding to shutdown.");
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "ios")]
    fn add_default_path_observer(
        tun_provider: Arc<dyn OSTunProvider>,
    ) -> Result<DefaultPathReceiver> {
        let (tx, rx) = mpsc::unbounded_channel();
        let observer = Arc::new(DefaultPathObserver::new(tx));

        tun_provider
            .set_default_path_observer(Some(observer.clone()))
            .map_err(Error::SetDefaultPathObserver)?;

        Ok(rx)
    }

    #[cfg(target_os = "ios")]
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
        self.exit.bump_sockets();
    }

    #[cfg(target_os = "ios")]
    fn update_peers(&mut self) -> Result<()> {
        let peer_update = self.entry_peer.resolved()?.into_peer_endpoint_update();

        // Update wireguard-go configuration with re-resolved peer endpoints.
        self.exit.update_peers(&[peer_update])?;

        Ok(())
    }
}
