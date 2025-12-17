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
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(windows)]
use crate::tunnel_state_machine::route_handler::RouteHandler;
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
        let mut wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.effective_entry_gateway_data(),
            self.entry_wg_keypair.private_key(),
            AllowedIps::Specific(vec![
                IpNetwork::from(self.connection_data.exit.endpoint.ip()),
                IpNetwork::from(tunnel_constants.in_tunnel_bandwidth_metadata_endpoint.ip()),
            ]),
            options.dns.clone(),
            self.entry_mtu(),
            #[cfg(target_os = "linux")]
            Some(tunnel_constants.fwmark),
        );
        if entry_amnezia {
            wg_entry_config = wg_entry_config.with_amnezia_config(AmneziaConfig::BASE);
        }

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_wg_keypair.private_key(),
            AllowedIps::All,
            options.dns,
            self.exit_mtu(),
            #[cfg(target_os = "linux")]
            None,
        );

        #[allow(unused_mut)]
        let mut entry_tunnel = wireguard_go::Tunnel::start(
            wg_entry_config.into_wireguard_config(),
            #[cfg(unix)]
            options.entry_tun.deref().dup_fd().map_err(Error::DupFd)?,
            #[cfg(windows)]
            &options.entry_tun_name,
            #[cfg(windows)]
            &options.entry_tun_guid,
            #[cfg(windows)]
            &options.wintun_tunnel_type,
        )
        .map_err(Error::Wireguard)?;

        let exit_tunnel = wireguard_go::Tunnel::start(
            wg_exit_config.into_wireguard_config(),
            #[cfg(unix)]
            options.exit_tun.deref().dup_fd().map_err(Error::DupFd)?,
            #[cfg(windows)]
            &options.exit_tun_name,
            #[cfg(windows)]
            &options.exit_tun_guid,
            #[cfg(windows)]
            &options.wintun_tunnel_type,
        )
        .map_err(Error::Wireguard)?;

        let shutdown_token = CancellationToken::new();
        let child_shutdown_token = shutdown_token.child_token();

        #[cfg(windows)]
        let wintun_entry_interface = entry_tunnel.wintun_interface().clone();
        #[cfg(windows)]
        let wintun_exit_interface = exit_tunnel.wintun_interface().clone();

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
        let mut wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.effective_entry_gateway_data(),
            self.entry_wg_keypair.private_key(),
            AllowedIps::Specific(vec![
                IpNetwork::from(self.connection_data.exit.endpoint.ip()),
                IpNetwork::from(tunnel_constants.in_tunnel_bandwidth_metadata_endpoint.ip()),
            ]),
            options.dns.clone(),
            self.entry_mtu(),
            #[cfg(target_os = "linux")]
            Some(tunnel_constants.fwmark),
        );

        if entry_amnezia {
            wg_entry_config = wg_entry_config.with_amnezia_config(AmneziaConfig::BASE);
        }

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_wg_keypair.private_key(),
            AllowedIps::All,
            options.dns,
            self.exit_mtu(),
            #[cfg(target_os = "linux")]
            None,
        );

        // Save entry peer so that we can re-resolve it and update wg config on network changes.
        #[cfg(target_os = "ios")]
        let orig_entry_peer = wg_entry_config.peer.clone();

        let mut two_hop_config = TwoHopConfig::new(wg_entry_config, wg_exit_config);

        // iOS does not perform dns64 resolution by default. Do that manually.
        #[cfg(target_os = "ios")]
        two_hop_config.entry.peer.resolve_in_place()?;

        let mut entry_tunnel =
            netstack::Tunnel::start(two_hop_config.entry.clone().into_netstack_config())?;

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

        two_hop_config.set_udp_proxy_listen_addr(exit_in_tunnel_udp_proxy.listen_addr());

        let entry_magic_bandwidth_tcp_proxy = entry_tunnel.start_in_tunnel_tcp_connection_proxy(
            tunnel_constants.in_tunnel_bandwidth_metadata_endpoint,
        )?;

        #[allow(unused_mut)]
        let mut exit_tunnel = wireguard_go::Tunnel::start(
            two_hop_config.exit.into_wireguard_config(),
            #[cfg(unix)]
            options.exit_tun.deref().dup_fd().map_err(Error::DupFd)?,
            #[cfg(windows)]
            &options.exit_tun_name,
            #[cfg(windows)]
            &options.exit_tun_guid,
            #[cfg(windows)]
            &options.wintun_tunnel_type,
        )?;

        if options
            .metadata_proxy_tx
            .send(entry_magic_bandwidth_tcp_proxy.listen_addr())
            .is_err()
        {
            tracing::warn!("Failed to send metadata proxy address")
        }

        let shutdown_token = CancellationToken::new();
        let child_shutdown_token = shutdown_token.child_token();

        #[cfg(windows)]
        let wintun_exit_interface = exit_tunnel.wintun_interface().clone();

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

            #[cfg(not(any(windows, target_os = "ios")))]
            {
                child_shutdown_token.cancelled().await;
                tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
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

                loop {
                    tokio::select! {
                        Some(new_path) = default_path_rx.next() => {
                            tracing::debug!("New default path: {}", new_path.description());

                            // Depending on the network device is connected to, we may need to re-resolve the IP addresses.
                            // For instance when device connects to IPv4-only server from IPv6-only network,
                            // it needs to use an IPv4-mapped address, which can be received by re-resolving
                            // the original peer IP.
                            match orig_entry_peer.resolved() {
                                Ok(resolved_peer) => {
                                    let peer_update = resolved_peer.into_peer_endpoint_update();

                                    // Update wireguard-go configuration with re-resolved peer endpoints.
                                    if let Err(e) = entry_tunnel.update_peers(&[peer_update]) {
                                       tracing::error!("Failed to update peers on network change: {}", e);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to re-resolve peer on default path update: {}", e);
                                }
                            }

                            // Rebind wireguard-go on tun device.
                            exit_tunnel.bump_sockets();
                            entry_tunnel.bump_sockets();
                        }
                        _ = child_shutdown_token.cancelled() => {
                            tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
                            break;
                        }
                    }
                }
            }

            // Windows: do not drop exit tunnel as it owns the underlying tunnel device.
            #[cfg(not(windows))]
            exit_tunnel.stop();
            entry_tunnel.stop();
            entry_magic_bandwidth_tcp_proxy.close();
            exit_in_tunnel_udp_proxy.close();

            Tombstone {
                tun_devices: vec![
                    #[cfg(not(windows))]
                    options.exit_tun,
                ],
                #[cfg(windows)]
                wg_instances: vec![exit_tunnel],
            }
        });

        Ok(TunnelHandle {
            shutdown_token,
            event_handler_task,
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
}

pub struct TunnelHandle {
    shutdown_token: CancellationToken,
    event_handler_task: JoinHandle<Tombstone>,
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
