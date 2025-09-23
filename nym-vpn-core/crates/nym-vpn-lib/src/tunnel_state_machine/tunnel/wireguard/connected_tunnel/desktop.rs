// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, SocketAddr};

use ipnetwork::IpNetwork;
use nym_authenticator_client::AuthClientMixnetListenerHandle;
use nym_crypto::asymmetric::x25519;
#[cfg(windows)]
use nym_routing::{Callback, CallbackHandle, EventType};
#[cfg(windows)]
use nym_wg_go::wireguard_go::WintunInterface;
use nym_wg_go::{netstack, wireguard_go};
#[cfg(windows)]
use nym_windows::net::{self as winnet, AddressFamily};
#[cfg(windows)]
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
#[cfg(unix)]
use tun::AsyncDevice;

#[cfg(windows)]
use crate::tunnel_state_machine::route_handler::RouteHandler;
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

pub struct ConnectedTunnel {
    entry_wg_keypair: x25519::KeyPair,
    exit_wg_keypair: x25519::KeyPair,
    connection_data: ConnectionData,
    bandwidth_controller_handle: JoinHandle<()>,
    auth_client_mixnet_listener_handle: Option<AuthClientMixnetListenerHandle>,
}

impl ConnectedTunnel {
    pub fn new(
        entry_wg_keypair: x25519::KeyPair,
        exit_wg_keypair: x25519::KeyPair,
        connection_data: ConnectionData,
        bandwidth_controller_handle: JoinHandle<()>,
        auth_client_mixnet_listener_handle: Option<AuthClientMixnetListenerHandle>,
    ) -> Self {
        Self {
            entry_wg_keypair,
            exit_wg_keypair,
            connection_data,
            bandwidth_controller_handle,
            auth_client_mixnet_listener_handle,
        }
    }

    pub fn connection_data(&self) -> &ConnectionData {
        &self.connection_data
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
        options: TunnelOptions,
        tunnel_constants: TunnelConstants,
    ) -> Result<TunnelHandle> {
        match options {
            TunnelOptions::TunTun(tuntun_options) => {
                self.run_using_tun_tun(
                    #[cfg(windows)]
                    route_handler,
                    tuntun_options,
                    tunnel_constants,
                )
                .await
            }
            TunnelOptions::Netstack(netstack_options) => self.run_using_netstack(
                #[cfg(windows)]
                route_handler,
                netstack_options,
                tunnel_constants,
            ),
        }
    }

    async fn run_using_tun_tun(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        options: TunTunTunnelOptions,
        tunnel_constants: TunnelConstants,
    ) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
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
            options.entry_tun.get_ref().dup_fd().map_err(Error::DupFd)?,
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
            options.exit_tun.get_ref().dup_fd().map_err(Error::DupFd)?,
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
            bandwidth_controller_handle: self.bandwidth_controller_handle,
            auth_client_mixnet_listener_handle: self.auth_client_mixnet_listener_handle,
            #[cfg(windows)]
            wintun_entry_interface: Some(wintun_entry_interface),
            #[cfg(windows)]
            wintun_exit_interface: Some(wintun_exit_interface),
        })
    }

    fn run_using_netstack(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        options: NetstackTunnelOptions,
        tunnel_constants: TunnelConstants,
    ) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
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

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_wg_keypair.private_key(),
            AllowedIps::All,
            options.dns,
            self.exit_mtu(),
            #[cfg(target_os = "linux")]
            None,
        );

        let mut two_hop_config = TwoHopConfig::new(wg_entry_config, wg_exit_config);

        let mut entry_tunnel =
            netstack::Tunnel::start(two_hop_config.entry.clone().into_netstack_config())?;

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

        let exit_tunnel = wireguard_go::Tunnel::start(
            two_hop_config.exit.into_wireguard_config(),
            #[cfg(unix)]
            options.exit_tun.get_ref().dup_fd().map_err(Error::DupFd)?,
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

            #[cfg(not(windows))]
            {
                child_shutdown_token.cancelled().await;
                tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
            }

            entry_tunnel.stop();
            entry_magic_bandwidth_tcp_proxy.close();
            exit_in_tunnel_udp_proxy.close();

            // Windows: do not drop exit tunnel as it owns the underlying tunnel device.
            #[cfg(not(windows))]
            exit_tunnel.stop();

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
            bandwidth_controller_handle: self.bandwidth_controller_handle,
            auth_client_mixnet_listener_handle: self.auth_client_mixnet_listener_handle,
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
    TunTun(TunTunTunnelOptions),

    /// Multihop using single tun adapter and netstack with local UDP forwarder to wrap tunnels.
    Netstack(NetstackTunnelOptions),
}

/// Multihop configuration using two tun adapters.
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
    bandwidth_controller_handle: JoinHandle<()>,
    auth_client_mixnet_listener_handle: Option<AuthClientMixnetListenerHandle>,
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

    pub fn mixnet_cancel_token(&self) -> Option<CancellationToken> {
        self.auth_client_mixnet_listener_handle
            .as_ref()
            .map(|listener| listener.mixnet_cancel_token())
    }

    /// Wait until the tunnel finished execution.
    ///
    /// Returns a tombstone containing the no longer used tunnel devices and wireguard tunnels (on Windows).
    pub async fn wait(self) -> Result<Tombstone, JoinError> {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
        }
        if let Some(auth_client_handle) = self.auth_client_mixnet_listener_handle {
            auth_client_handle.stop().await;
        }

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
