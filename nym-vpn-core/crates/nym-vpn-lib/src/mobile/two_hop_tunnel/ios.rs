use std::sync::Arc;

use nym_wg_go::{netstack, wireguard_go};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    mobile::{
        ios::{
            default_path_observer::{DefaultPathObserver, DefaultPathReceiver, OSDefaultPath},
            dns64::Dns64Resolution,
            tun,
            tun_provider::OSTunProvider,
        },
        tunnel_settings::TunnelSettings,
        two_hop_config::TwoHopConfig,
        wg_config::{WgNodeConfig, WgPeer},
        Error, Result,
    },
    platform::uniffi_set_listener_status,
    uniffi_custom_impls::{StatusEvent, TunStatus},
};

pub struct TwoHopTunnelImp {
    /// Entry node tunnel
    /// Retained inside struct on purpose
    _entry: netstack::Tunnel,

    /// Exit node tunnel
    exit: wireguard_go::Tunnel,

    /// UDP connection over the entry tunnel, towards exit node.
    /// Retained inside struct on purpose
    _exit_connection: Option<netstack::TunnelConnection>,

    /// Entry peer configuration.
    entry_peer: WgPeer,

    /// Interface for interacting with the iOS tunnel provider.
    /// Retained inside struct on purpose
    _tun_provider: Arc<dyn OSTunProvider>,

    /// A conduit for receiving default path updates.
    default_path_receiver: DefaultPathReceiver,

    /// Cancellation token.
    shutdown_token: CancellationToken,
}

impl TwoHopTunnelImp {
    /// Start two-hop wg tunnel given entry and exit nodes.
    pub async fn start(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        tun_provider: Arc<dyn OSTunProvider>,
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
        two_hop_config.entry.peer.resolve_in_place()?;

        // Obtain tunnel file descriptor and interface name.
        let tun_fd = tun::get_tun_fd().ok_or(Error::CannotLocateTunFd)?;
        tracing::debug!("Found tunnel fd: {}", tun_fd);

        let tun_name = tun::get_tun_ifname(tun_fd).ok_or(Error::ObtainTunName)?;
        tracing::debug!("Tunnel interface name: {}", tun_name);

        let tunnel_settings = TunnelSettings {
            interface_addresses: two_hop_config.tun.addresses,
            dns_servers: two_hop_config.tun.dns,
            remote_addresses: vec![two_hop_config.entry.peer.endpoint.ip()],
            mtu: two_hop_config.tun.mtu,
        };

        // Configure tun, dns and routing
        tun_provider
            .set_tunnel_network_settings(tunnel_settings.into_tunnel_network_settings())
            .await
            .map_err(Error::SetNetworkSettings)?;

        // Transform wg config structs into what nym-wg-go expects.
        let entry_wg_config = two_hop_config.entry.into_netstack_config();
        let exit_wg_config = two_hop_config.exit.into_wireguard_config();

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

        uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Up));

        // Setup default path observer.
        let default_path_receiver = Self::add_default_path_observer(tun_provider.clone())?;

        let two_hop_tunnel = Self {
            _entry: entry_tunnel,
            exit: exit_tunnel,
            _exit_connection: Some(exit_connection),
            entry_peer: orig_entry_peer,
            _tun_provider: tun_provider,
            default_path_receiver,
            shutdown_token,
        };

        two_hop_tunnel.run().await;

        Ok(())
    }

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

    fn update_peers(&mut self) -> Result<()> {
        let peer_update = self.entry_peer.resolved()?.into_peer_endpoint_update();

        // Update wireguard-go configuration with re-resolved peer endpoints.
        self.exit.update_peers(&[peer_update])?;

        Ok(())
    }
}
