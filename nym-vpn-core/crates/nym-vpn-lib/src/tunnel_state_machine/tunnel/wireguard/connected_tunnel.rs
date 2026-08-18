// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::ops::Deref;
#[cfg(target_os = "ios")]
use std::time::Duration;
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

#[cfg(target_os = "ios")]
use dispatch2::{DispatchQueue, DispatchQueueAttr};
use ipnetwork::IpNetwork;
#[cfg(target_os = "ios")]
use nym_apple_network::PathMonitor;
use nym_crypto::asymmetric::x25519;
#[cfg(windows)]
use nym_routing::{Callback, CallbackHandle, EventType};
#[cfg(not(windows))]
use nym_wg_go::wireguard_go::TunnelFd;
#[cfg(windows)]
use nym_wg_go::wireguard_go::WintunInterface;
use nym_wg_go::{amnezia::AmneziaConfig, netstack, wireguard_go};
#[cfg(windows)]
use nym_windows::net::{self as winnet, AddressFamily};
#[cfg(any(windows, target_os = "ios"))]
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle};
#[cfg(target_os = "ios")]
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;
#[cfg(unix)]
use tun::AsyncDevice;

#[cfg(target_os = "android")]
use crate::dns_filter::DnsFilter;
#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(windows)]
use crate::tunnel_state_machine::route_handler::RouteHandler;
#[cfg(target_os = "android")]
use crate::tunnel_state_machine::tunnel::wireguard::dns_filter_proxy::DnsFilterProxy;
#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::tunnel::wireguard::dns64::Dns64Resolution;
#[cfg(unix)]
use crate::tunnel_state_machine::tunnel::wireguard::fd::DupFd;
use crate::{
    tunnel_state_machine::{
        TunnelConstants,
        tunnel::{
            Error, Result, Tombstone,
            wireguard::{
                ConnectionData,
                two_hop_config::{ENTRY_MTU, EXIT_MTU, TwoHopConfig},
            },
        },
    },
    wg_config::{AllowedIps, WgNodeConfig},
};

/// Delay before acting on default route changes.
#[cfg(target_os = "ios")]
const DEFAULT_PATH_DEBOUNCE: Duration = Duration::from_millis(250);

/// Bridged netstack entry peers listen on loopback. Rebinding that socket on
/// iOS path updates delays the first handshake until after the bridge forwarder
/// used to abort, which never reaches Connected.
#[cfg(any(test, target_os = "ios"))]
pub(crate) fn should_bump_entry_sockets_on_path_change(entry_is_loopback: bool) -> bool {
    !entry_is_loopback
}

pub struct ConnectedTunnel {
    entry_wg_keypair: Arc<x25519::KeyPair>,
    exit_wg_keypair: Arc<x25519::KeyPair>,
    connection_data: ConnectionData,
}

impl ConnectedTunnel {
    pub fn new(
        entry_wg_keypair: Arc<x25519::KeyPair>,
        exit_wg_keypair: Arc<x25519::KeyPair>,
        connection_data: ConnectionData,
    ) -> Self {
        Self {
            entry_wg_keypair,
            exit_wg_keypair,
            connection_data,
        }
    }

    pub fn connection_data(&self) -> &ConnectionData {
        &self.connection_data
    }

    pub fn connection_data_mut(&mut self) -> &mut ConnectionData {
        &mut self.connection_data
    }

    pub fn entry_mtu(&self) -> u16 {
        ENTRY_MTU
    }

    pub fn exit_mtu(&self) -> u16 {
        EXIT_MTU
    }

    pub async fn run(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        options: TunnelOptions,
        tunnel_constants: TunnelConstants,
        entry_amnezia: bool,
    ) -> Result<TunnelHandle> {
        match options {
            #[cfg(not(any(target_os = "ios", target_os = "android")))]
            TunnelOptions::TunTun(tuntun_options) => {
                self.run_using_tun_tun(
                    #[cfg(windows)]
                    route_handler,
                    tuntun_options,
                    tunnel_constants,
                    entry_amnezia,
                )
                .await
            }
            TunnelOptions::Netstack(netstack_options) => self.run_using_netstack(
                #[cfg(windows)]
                route_handler,
                #[cfg(target_os = "android")]
                tun_provider,
                netstack_options,
                tunnel_constants,
                entry_amnezia,
            ),
        }
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    async fn run_using_tun_tun(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        options: TunTunTunnelOptions,
        tunnel_constants: TunnelConstants,
        entry_amnezia: bool,
    ) -> Result<TunnelHandle> {
        let entry_mtu = self.entry_mtu();
        let exit_mtu = self.exit_mtu();
        let (wg_entry_data, wg_exit_data) =
            self.connection_data.into_effective_entry_and_exit_data();
        let mut wg_entry_config = WgNodeConfig::with_wireguard_config(
            wg_entry_data,
            self.entry_wg_keypair,
            AllowedIps::Specific(vec![
                IpNetwork::from(wg_exit_data.endpoint.ip()),
                IpNetwork::from(tunnel_constants.in_tunnel_bandwidth_metadata_endpoint.ip()),
            ]),
            options.dns.clone(),
            entry_mtu,
            #[cfg(target_os = "linux")]
            Some(tunnel_constants.fwmark),
        );
        if entry_amnezia {
            wg_entry_config = wg_entry_config.with_amnezia_config(AmneziaConfig::BASE);
        }

        let wg_exit_config = WgNodeConfig::with_wireguard_config(
            wg_exit_data,
            self.exit_wg_keypair,
            AllowedIps::All,
            options.dns,
            exit_mtu,
            #[cfg(target_os = "linux")]
            None,
        );

        let builder = wireguard_go::TunnelConfig::builder();
        #[cfg(unix)]
        let builder = builder.tun_fd(TunnelFd::Tun(
            options.entry_tun.deref().dup_fd().map_err(Error::DupFd)?,
        ));
        #[cfg(windows)]
        let builder = {
            builder
                .interface_name(options.entry_tun_name)
                .requested_guid(options.entry_tun_guid)
                .wintun_tunnel_type(options.wintun_tunnel_type.clone())
        };
        let entry_tunnel_config = builder.build();

        #[allow(unused_mut)]
        let mut entry_tunnel = wireguard_go::Tunnel::start(
            wg_entry_config.into_wireguard_config(),
            entry_tunnel_config,
        )
        .map_err(Error::Wireguard)?;

        let builder = wireguard_go::TunnelConfig::builder();
        #[cfg(unix)]
        let builder = builder.tun_fd(TunnelFd::Tun(
            options.exit_tun.deref().dup_fd().map_err(Error::DupFd)?,
        ));
        #[cfg(windows)]
        let builder = {
            builder
                .interface_name(options.exit_tun_name)
                .requested_guid(options.exit_tun_guid)
                .wintun_tunnel_type(options.wintun_tunnel_type)
        };
        let exit_tunnel_config = builder.build();

        let exit_tunnel =
            wireguard_go::Tunnel::start(wg_exit_config.into_wireguard_config(), exit_tunnel_config)
                .map_err(Error::Wireguard)?;

        let shutdown_token = CancellationToken::new();
        let child_shutdown_token = shutdown_token.child_token();

        #[cfg(windows)]
        let wintun_entry_interface = entry_tunnel.wintun_interface().clone();
        #[cfg(windows)]
        let wintun_exit_interface = exit_tunnel.wintun_interface().clone();
        let exit_stats_reader = exit_tunnel.stats_reader();

        let event_handler_task = tokio::spawn(async move {
            #[cfg(windows)]
            {
                let (default_route_tx, mut default_route_rx) = mpsc::unbounded_channel();
                let _callback = Self::add_default_route_listener(route_handler, default_route_tx);

                loop {
                    tokio::select! {
                        _ = child_shutdown_token.cancelled() => {
                            tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
                            break
                        }
                        Some((interface_index, address_family)) = default_route_rx.recv() => {
                            tracing::debug!("New default route: {} {}", interface_index, address_family);
                            entry_tunnel.rebind_tunnel_socket(address_family, interface_index);
                        }
                        else => {
                            tracing::error!("Default route listener has been dropped. Exiting event loop.");
                            break;
                        }
                    }
                }
            }

            // On non-windows platforms we have direct ownership over the tunnel adapters,
            // so we can shutdown the tunnel right away and return adapters with a tombstone.
            #[cfg(not(windows))]
            {
                child_shutdown_token.cancelled().await;
                tracing::debug!("Received tunnel shutdown event. Exiting event loop.");

                entry_tunnel.stop();
                exit_tunnel.stop();

                Tombstone::with_tun_devices(vec![options.exit_tun, options.entry_tun])
            }

            // On windows return tunnels as part of tombstone since they own tunnel adapters and should be
            // dropped only after resetting the routing table.
            #[cfg(windows)]
            {
                Tombstone::with_wg_instances(vec![exit_tunnel, entry_tunnel])
            }
        });

        Ok(TunnelHandle {
            shutdown_token,
            event_handler_task,
            exit_stats_reader,
            #[cfg(windows)]
            wintun_entry_interface: Some(wintun_entry_interface),
            #[cfg(windows)]
            wintun_exit_interface: Some(wintun_exit_interface),
        })
    }

    fn run_using_netstack(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        options: NetstackTunnelOptions,
        tunnel_constants: TunnelConstants,
        entry_amnezia: bool,
    ) -> Result<TunnelHandle> {
        let entry_mtu = self.entry_mtu();
        let exit_mtu = self.exit_mtu();
        let (wg_entry_data, wg_exit_data) =
            self.connection_data.into_effective_entry_and_exit_data();
        let mut wg_entry_config = WgNodeConfig::with_wireguard_config(
            wg_entry_data,
            self.entry_wg_keypair,
            AllowedIps::Specific(vec![
                IpNetwork::from(wg_exit_data.endpoint.ip()),
                IpNetwork::from(tunnel_constants.in_tunnel_bandwidth_metadata_endpoint.ip()),
            ]),
            options.dns.clone(),
            entry_mtu,
            #[cfg(target_os = "linux")]
            Some(tunnel_constants.fwmark),
        );

        if entry_amnezia {
            wg_entry_config = wg_entry_config.with_amnezia_config(AmneziaConfig::BASE);
        }

        let wg_exit_config = WgNodeConfig::with_wireguard_config(
            wg_exit_data,
            self.exit_wg_keypair,
            AllowedIps::All,
            options.dns,
            exit_mtu,
            #[cfg(target_os = "linux")]
            None,
        );

        // Save entry peer so that we can re-resolve it and update wg config on network changes.
        #[cfg(target_os = "ios")]
        let entry_peer_update = wg_entry_config.peer.as_peer_endpoint_update();

        let mut two_hop_config = TwoHopConfig::new(wg_entry_config, wg_exit_config);

        // iOS does not perform dns64 resolution by default. Do that manually.
        #[cfg(target_os = "ios")]
        two_hop_config.entry.peer.resolve_in_place()?;

        let mut entry_tunnel =
            netstack::Tunnel::start(two_hop_config.entry.into_netstack_config())?;

        // Configure tunnel sockets to bypass the tunnel interface.
        #[cfg(target_os = "android")]
        {
            match entry_tunnel.get_socket_v4() {
                Ok(fd) => tun_provider.bypass(fd),
                Err(e) => tracing::error!("Failed to obtain bypass socket (ipv4): {}", e),
            }
            match entry_tunnel.get_socket_v6() {
                Ok(fd) => tun_provider.bypass(fd),
                Err(e) => tracing::error!("Failed to obtain bypass socket (ipv6): {}", e),
            }
        }

        // Open connection to the exit node via entry node.
        let exit_in_tunnel_udp_proxy = entry_tunnel.start_in_tunnel_udp_connection_proxy(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
        )?;

        two_hop_config.forwarder.listen_endpoint = exit_in_tunnel_udp_proxy.listen_addr();
        two_hop_config.exit.peer.endpoint = exit_in_tunnel_udp_proxy.listen_addr();

        let entry_magic_bandwidth_tcp_proxy = entry_tunnel.start_in_tunnel_tcp_connection_proxy(
            tunnel_constants.in_tunnel_bandwidth_metadata_endpoint,
        )?;

        let shutdown_token = CancellationToken::new();
        let exit_wg_config = two_hop_config.exit.into_wireguard_config();

        let builder = wireguard_go::TunnelConfig::builder();

        #[cfg(target_os = "android")]
        let (builder, proxy_join_handle, android_exit_tun) = {
            if let Some(dns_filter) = options.dns_filter {
                let proxy = DnsFilterProxy::start(
                    options.exit_tun,
                    dns_filter,
                    shutdown_token.child_token(),
                )
                .map_err(Error::CreateDnsFilterProxy)?;

                let builder = builder.tun_fd(TunnelFd::Proxy(proxy.wg_fd));
                (builder, Some(proxy.join_handle), None)
            } else {
                let tun_fd = options.exit_tun.deref().dup_fd().map_err(Error::DupFd)?;
                let builder = builder.tun_fd(TunnelFd::Tun(tun_fd));

                (builder, None, Some(options.exit_tun))
            }
        };

        #[cfg(windows)]
        let builder = builder
            .interface_name(options.exit_tun_name)
            .requested_guid(options.exit_tun_guid)
            .wintun_tunnel_type(options.wintun_tunnel_type);

        #[cfg(all(unix, not(target_os = "android")))]
        let builder = builder.tun_fd(TunnelFd::Tun(
            options.exit_tun.deref().dup_fd().map_err(Error::DupFd)?,
        ));

        let exit_tunnel_config = builder.build();

        #[allow(unused_mut)]
        let mut exit_tunnel = wireguard_go::Tunnel::start(exit_wg_config, exit_tunnel_config)?;

        if options
            .metadata_proxy_tx
            .send(entry_magic_bandwidth_tcp_proxy.listen_addr())
            .is_err()
        {
            tracing::warn!("Failed to send metadata proxy address")
        }

        #[cfg(windows)]
        let wintun_exit_interface = exit_tunnel.wintun_interface().clone();
        let exit_stats_reader = exit_tunnel.stats_reader();

        let child_shutdown_token = shutdown_token.child_token();
        let event_handler_task = tokio::spawn(async move {
            #[cfg(windows)]
            {
                let (default_route_tx, mut default_route_rx) = mpsc::unbounded_channel();
                let _callback = Self::add_default_route_listener(route_handler, default_route_tx);

                loop {
                    tokio::select! {
                        _ = child_shutdown_token.cancelled() => {
                            tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
                            break
                        }
                        Some((interface_index, address_family)) = default_route_rx.recv() => {
                            tracing::debug!("New default route: {} {}", interface_index, address_family);
                            entry_tunnel.rebind_tunnel_socket(address_family, interface_index);
                        }
                        else => {
                            tracing::error!("Default route listener has been dropped. Exiting event loop.");
                            break;
                        }
                    }
                }
            }

            #[cfg(target_os = "ios")]
            {
                let (default_path_tx, default_path_rx) = mpsc::unbounded_channel();
                let mut default_path_rx = debounced::debounced(
                    UnboundedReceiverStream::new(default_path_rx),
                    DEFAULT_PATH_DEBOUNCE,
                );

                let queue = DispatchQueue::new(
                    "net.nymtech.vpn.wg-path-monitor",
                    DispatchQueueAttr::SERIAL,
                );
                let mut path_monitor = PathMonitor::new();
                path_monitor.set_dispatch_queue(&queue);
                path_monitor.set_update_handler(move |network_path| {
                    if let Err(e) = default_path_tx.send(network_path) {
                        tracing::error!("Failed to send new default path: {}", e);
                    }
                });
                path_monitor.start();

                let mut old_resolved_peer = entry_peer_update.clone();

                loop {
                    tokio::select! {
                        Some(new_path) = default_path_rx.next() => {
                            tracing::debug!("New default path: {}", new_path.description());

                            // Depending on the network device is connected to, we may need to re-resolve the IP addresses.
                            // For instance when device connects to IPv4-only server from IPv6-only network,
                            // it needs to use an IPv4-mapped address, which can be received by re-resolving
                            // the original peer IP.
                            if !entry_peer_update.is_loopback() {
                                match entry_peer_update.clone().resolved() {
                                    Ok(resolved_peer) => {
                                        // check if peer has changed
                                        if resolved_peer != old_resolved_peer {
                                            old_resolved_peer = resolved_peer.clone();
                                            // Update wireguard-go configuration with re-resolved peer endpoints.
                                            if let Err(e) = entry_tunnel.update_peers(&[resolved_peer]) {
                                                tracing::error!("Failed to update peers on network change: {}", e);
                                            }
                                        } else {
                                            tracing::debug!("Skipping peer update: resolved address unchanged: {}", resolved_peer.endpoint);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to re-resolve peer on default path update: {}", e);
                                    }
                                }
                            }

                            // Rebind wireguard-go on tun device.
                            exit_tunnel.bump_sockets();
                            if should_bump_entry_sockets_on_path_change(
                                entry_peer_update.is_loopback(),
                            ) {
                                entry_tunnel.bump_sockets();
                            }
                        }
                        _ = child_shutdown_token.cancelled() => {
                            tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
                            break;
                        }
                    }
                }
            }

            #[cfg(not(any(windows, target_os = "ios")))]
            {
                child_shutdown_token.cancelled().await;
                tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
            }

            // Windows: do not drop exit tunnel as it owns the underlying tunnel device.
            #[cfg(not(windows))]
            exit_tunnel.stop();
            entry_tunnel.stop();
            entry_magic_bandwidth_tcp_proxy.close();
            exit_in_tunnel_udp_proxy.close();

            #[cfg(target_os = "android")]
            {
                if let Some(proxy_join_handle) = proxy_join_handle {
                    match proxy_join_handle.await {
                        Ok(tun_device) => Tombstone::with_tun_device(tun_device),
                        Err(err) => {
                            tracing::error!("DNS filter proxy task panicked: {err}");
                            Tombstone::default()
                        }
                    }
                } else {
                    android_exit_tun
                        .map(Tombstone::with_tun_device)
                        .unwrap_or_default()
                }
            }

            #[cfg(windows)]
            {
                Tombstone::with_wg_instances(vec![exit_tunnel])
            }

            #[cfg(not(any(windows, target_os = "android")))]
            {
                Tombstone::with_tun_device(options.exit_tun)
            }
        });

        Ok(TunnelHandle {
            shutdown_token,
            event_handler_task,
            exit_stats_reader,
            #[cfg(windows)]
            wintun_entry_interface: None,
            #[cfg(windows)]
            wintun_exit_interface: Some(wintun_exit_interface),
        })
    }

    #[cfg(windows)]
    async fn add_default_route_listener(
        mut route_handler: RouteHandler,
        tx: mpsc::UnboundedSender<(u32, AddressFamily)>,
    ) -> Result<CallbackHandle> {
        let default_route_callback: Callback = Box::new(move |event, address_family| {
            let result = match event {
                EventType::Removed => {
                    tracing::debug!(
                        "Default {} interface was removed. Rebind to blackhole.",
                        address_family
                    );
                    Ok(0)
                }
                EventType::Updated(interface_and_gateway)
                | EventType::UpdatedDetails(interface_and_gateway) => {
                    let interface_name =
                        winnet::alias_from_luid(&interface_and_gateway.iface).unwrap_or_default();
                    tracing::debug!(
                        "New default {} route: {}, gateway: {}",
                        interface_name.to_string_lossy(),
                        address_family,
                        interface_and_gateway.gateway,
                    );
                    winnet::index_from_luid(&interface_and_gateway.iface)
                }
            };

            match result {
                Ok(interface_index) => {
                    if let Err(e) = tx.send((interface_index, address_family)) {
                        tracing::error!("Failed to send new default route over the channel: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to convert luid to interface index: {}", e);
                }
            }
        });

        route_handler
            .add_default_route_listener(default_route_callback)
            .await
            .map_err(Error::AddDefaultRouteListener)
    }
}

pub enum TunnelOptions {
    /// Multihop configured using two tun adapters.
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    TunTun(TunTunTunnelOptions),

    /// Multihop using single tun adapter and netstack with local UDP forwarder to wrap tunnels.
    Netstack(NetstackTunnelOptions),
}

/// Multihop configuration using two tun adapters.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
pub struct TunTunTunnelOptions {
    /// Entry tunnel device.
    #[cfg(unix)]
    pub entry_tun: AsyncDevice,

    /// Exit tunnel device.
    #[cfg(unix)]
    pub exit_tun: AsyncDevice,

    /// Entry tunnel device name.
    #[cfg(windows)]
    pub entry_tun_name: String,

    /// Entry tunnel guid.
    #[cfg(windows)]
    pub entry_tun_guid: String,

    /// Exit tunnel device name.
    #[cfg(windows)]
    pub exit_tun_name: String,

    /// Exit tunnel guid.
    #[cfg(windows)]
    pub exit_tun_guid: String,

    /// Wintun tunnel type identifier.
    #[cfg(windows)]
    pub wintun_tunnel_type: String,

    /// In-tunnel DNS addresses
    pub dns: Vec<IpAddr>,
}

/// Multihop configuration based on WireGuard/netstack.
pub struct NetstackTunnelOptions {
    /// Sender that receives an endpoint of metadata proxy for entry interface
    pub metadata_proxy_tx: tokio::sync::oneshot::Sender<SocketAddr>,

    /// Entry tunnel device.
    #[cfg(unix)]
    pub exit_tun: AsyncDevice,

    /// Exit tunnel device name.
    #[cfg(windows)]
    pub exit_tun_name: String,

    /// Exit tunnel guid.
    #[cfg(windows)]
    pub exit_tun_guid: String,

    /// Wintun tunnel type identifier.
    #[cfg(windows)]
    pub wintun_tunnel_type: String,

    /// In-tunnel DNS addresses
    pub dns: Vec<IpAddr>,

    /// DNS filter for ad-blocking (Android only).
    #[cfg(target_os = "android")]
    pub dns_filter: Option<DnsFilter>,
}

pub struct TunnelHandle {
    shutdown_token: CancellationToken,
    event_handler_task: JoinHandle<Tombstone>,
    exit_stats_reader: wireguard_go::TunnelStatsReader,
    #[cfg(windows)]
    wintun_entry_interface: Option<WintunInterface>,
    #[cfg(windows)]
    wintun_exit_interface: Option<WintunInterface>,
}

impl TunnelHandle {
    /// Close entry and exit WireGuard tunnels and signal mixnet facilities shutdown.
    pub fn cancel(&mut self) {
        self.shutdown_token.cancel();
    }

    /// Wait until the tunnel finished execution.
    ///
    /// Returns a tombstone containing the no longer used tunnel devices and wireguard tunnels (on Windows).
    pub async fn wait(self) -> Result<Tombstone, JoinError> {
        self.event_handler_task.await
    }

    /// Query live stats for the exit WireGuard peer via the UAPI GET interface.
    pub fn get_exit_stats(&self) -> nym_wg_go::Result<wireguard_go::TunnelStats> {
        self.exit_stats_reader.get_stats()
    }

    /// Returns entry wintun interface descriptor when available.
    /// Note: netstack based tunnel uses virtual adapter so it will always return `None`.
    #[cfg(windows)]
    pub fn entry_wintun_interface(&self) -> Option<&WintunInterface> {
        self.wintun_entry_interface.as_ref()
    }

    /// Returns exit wintun interface descriptor when available.
    #[cfg(windows)]
    pub fn exit_wintun_interface(&self) -> Option<&WintunInterface> {
        self.wintun_exit_interface.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::should_bump_entry_sockets_on_path_change;

    #[test]
    fn skip_entry_socket_bump_when_bridged_on_loopback() {
        assert!(!should_bump_entry_sockets_on_path_change(true));
        assert!(should_bump_entry_sockets_on_path_change(false));
    }
}
