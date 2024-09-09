use std::sync::Arc;

use nym_wg_go::{netstack, wireguard_go};
use tokio_util::sync::CancellationToken;

use crate::{
    mobile::{
        tunnel_settings::TunnelSettings, two_hop_config::TwoHopConfig, wg_config::WgNodeConfig,
        Error, Result,
    },
    platform::{android::AndroidTunProvider, uniffi_set_listener_status},
    uniffi_custom_impls::{StatusEvent, TunStatus},
};

pub struct TwoHopTunnelImp {
    /// Entry node tunnel
    /// Retained inside struct on purpose
    _entry: netstack::Tunnel,

    /// Exit node tunnel
    /// Retained inside struct on purpose
    _exit: wireguard_go::Tunnel,

    /// UDP connection over the entry tunnel, towards exit node.
    /// Retained inside struct on purpose
    _exit_connection: netstack::TunnelConnection,

    /// Interface for interacting with the Android tunnel provider.
    /// Retained inside struct on purpose
    _tun_provider: Arc<dyn AndroidTunProvider>,

    /// Cancellation token.
    shutdown_token: CancellationToken,
}

impl TwoHopTunnelImp {
    /// Start two-hop wg tunnel given entry and exit nodes.
    pub async fn start(
        entry_node_config: WgNodeConfig,
        exit_node_config: WgNodeConfig,
        tun_provider: Arc<dyn AndroidTunProvider>,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        let two_hop_config = TwoHopConfig::new(entry_node_config, exit_node_config);

        tracing::info!("Two-hop entry: {:#?}", two_hop_config.entry);
        tracing::info!("Two-hop exit: {:#?}", two_hop_config.exit);
        tracing::info!("Two-hop tun: {:#?}", two_hop_config.tun);
        tracing::info!("Two-hop forwarder: {:#?}", two_hop_config.forwarder);

        let tunnel_settings = TunnelSettings {
            interface_addresses: two_hop_config.tun.addresses,
            dns_servers: two_hop_config.tun.dns,
            remote_addresses: vec![two_hop_config.entry.peer.endpoint.ip()],
            mtu: two_hop_config.tun.mtu,
        };

        // Transform wg config structs into what nym-wg-go expects.
        let entry_wg_config = two_hop_config.entry.into_netstack_config();
        let exit_wg_config = two_hop_config.exit.into_wireguard_config();

        // Create netstack wg connected to the entry node.
        let mut entry_tunnel = netstack::Tunnel::start(entry_wg_config)?;

        // Configure tunnel sockets to bypass the tunnel interface.
        match entry_tunnel.get_socket_v4() {
            Ok(fd) => tun_provider.bypass(fd),
            Err(e) => tracing::error!("Failed to obtain bypass socket (ipv4): {}", e),
        }
        match entry_tunnel.get_socket_v6() {
            Ok(fd) => tun_provider.bypass(fd),
            Err(e) => tracing::error!("Failed to obtain bypass socket (ipv6): {}", e),
        }

        // Open connection to the exit node via entry node.
        let exit_connection = entry_tunnel.open_connection(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
        )?;

        let tun_fd = tun_provider
            .configure_tunnel(tunnel_settings.into_tunnel_network_settings())
            .map_err(Error::SetNetworkSettings)?;
        if tun_fd == -1 {
            return Err(Error::CannotLocateTunFd);
        }

        // Create exit tunnel capturing exit traffic on device and sending it to the local udp forwarder.
        let exit_tunnel = wireguard_go::Tunnel::start(exit_wg_config, tun_fd)?;

        uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Up));

        let two_hop_tunnel = Self {
            _entry: entry_tunnel,
            _exit: exit_tunnel,
            _exit_connection: exit_connection,
            _tun_provider: tun_provider,
            shutdown_token,
        };

        two_hop_tunnel.run().await;

        Ok(())
    }

    async fn run(self) {
        self.shutdown_token.cancelled().await;
        tracing::debug!("Received shutdown.");
    }
}
