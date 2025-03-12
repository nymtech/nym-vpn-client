// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error as StdError, net::IpAddr};

#[cfg(windows)]
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
#[cfg(unix)]
use tun::AsyncDevice;

#[cfg(windows)]
use nym_routing::{Callback, CallbackHandle, EventType};
use nym_task::TaskManager;
use nym_wg_gateway_client::WgGatewayClient;
#[cfg(windows)]
use nym_wg_go::wireguard_go::WintunInterface;
use nym_wg_go::{netstack, wireguard_go};
#[cfg(windows)]
use nym_windows::net::{self as winnet, AddressFamily};

#[cfg(windows)]
use crate::tunnel_state_machine::route_handler::RouteHandler;
#[cfg(target_os = "linux")]
use crate::tunnel_state_machine::route_handler::TUNNEL_FWMARK;
#[cfg(unix)]
use crate::tunnel_state_machine::tunnel::wireguard::fd::DupFd;
use crate::{
    tunnel_state_machine::tunnel::{
        wireguard::{connector::ConnectionData, two_hop_config::TwoHopConfig},
        Error, Result, Tombstone,
    },
    wg_config::WgNodeConfig,
};

pub struct ConnectedTunnel {
    task_manager: TaskManager,
    entry_gateway_client: WgGatewayClient,
    exit_gateway_client: WgGatewayClient,
    connection_data: ConnectionData,
    bandwidth_controller_handle: JoinHandle<()>,
}

impl ConnectedTunnel {
    pub fn new(
        task_manager: TaskManager,
        entry_gateway_client: WgGatewayClient,
        exit_gateway_client: WgGatewayClient,
        connection_data: ConnectionData,
        bandwidth_controller_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            task_manager,
            entry_gateway_client,
            exit_gateway_client,
            connection_data,
            bandwidth_controller_handle,
        }
    }

    pub fn connection_data(&self) -> &ConnectionData {
        &self.connection_data
    }

    pub fn entry_mtu(&self) -> u16 {
        // 1500 - 80 (ipv6+wg header)
        1420
    }

    pub fn exit_mtu(&self) -> u16 {
        // 1420 - 80 (ipv6+wg header)
        1340
    }

    pub async fn run(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        options: TunnelOptions,
    ) -> Result<TunnelHandle> {
        match options {
            TunnelOptions::TunTun(tuntun_options) => {
                self.run_using_tun_tun(
                    #[cfg(windows)]
                    route_handler,
                    tuntun_options,
                )
                .await
            }
            TunnelOptions::Netstack(netstack_options) => self.run_using_netstack(
                #[cfg(windows)]
                route_handler,
                netstack_options,
            ),
        }
    }

    async fn run_using_tun_tun(
        self,
        #[cfg(windows)] route_handler: RouteHandler,
        options: TunTunTunnelOptions,
    ) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            options.dns.clone(),
            self.entry_mtu(),
            #[cfg(target_os = "linux")]
            Some(TUNNEL_FWMARK),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
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
            task_manager: self.task_manager,
            shutdown_token,
            event_handler_task,
            bandwidth_controller_handle: self.bandwidth_controller_handle,
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
    ) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            options.dns.clone(),
            self.entry_mtu(),
            #[cfg(target_os = "linux")]
            Some(TUNNEL_FWMARK),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
            options.dns,
            self.exit_mtu(),
            #[cfg(target_os = "linux")]
            None,
        );

        let two_hop_config = TwoHopConfig::new(wg_entry_config, wg_exit_config);

        let mut entry_tunnel =
            netstack::Tunnel::start(two_hop_config.entry.into_netstack_config())?;

        // Open connection to the exit node via entry node.
        let exit_connection = entry_tunnel.open_connection(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
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
            exit_connection.close();

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
            task_manager: self.task_manager,
            shutdown_token,
            event_handler_task,
            bandwidth_controller_handle: self.bandwidth_controller_handle,
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
    task_manager: TaskManager,
    shutdown_token: CancellationToken,
    event_handler_task: JoinHandle<Tombstone>,
    bandwidth_controller_handle: JoinHandle<()>,
    #[cfg(windows)]
    wintun_entry_interface: Option<WintunInterface>,
    #[cfg(windows)]
    wintun_exit_interface: Option<WintunInterface>,
}

impl TunnelHandle {
    /// Close entry and exit WireGuard tunnels and signal mixnet facilities shutdown.
    pub fn cancel(&mut self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.task_manager.signal_shutdown() {
            tracing::error!("Failed to signal task manager shutdown: {}", e);
        }
    }

    /// Wait for the next mixnet error.
    ///
    /// This method is cancel safe.
    /// Returns `None` if the underlying channel has been closed.
    pub async fn recv_error(&mut self) -> Option<Box<dyn StdError + 'static + Send + Sync>> {
        self.task_manager.wait_for_error().await
    }

    /// Wait until the tunnel finished execution.
    ///
    /// Returns a tombstone containing the no longer used tunnel devices and wireguard tunnels (on Windows).
    pub async fn wait(self) -> Result<Tombstone, JoinError> {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
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
