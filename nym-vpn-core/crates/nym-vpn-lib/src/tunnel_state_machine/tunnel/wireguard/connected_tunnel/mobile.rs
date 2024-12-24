// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;
#[cfg(target_os = "ios")]
use std::time::Duration;
use std::{error::Error as StdError, net::IpAddr};

#[cfg(target_os = "ios")]
use nym_apple_dispatch::{Queue, QueueAttr};
#[cfg(target_os = "ios")]
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle};
#[cfg(target_os = "ios")]
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

#[cfg(target_os = "ios")]
use nym_apple_network::PathMonitor;
use nym_task::TaskManager;
use nym_wg_gateway_client::WgGatewayClient;
use nym_wg_go::{netstack, wireguard_go};

#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::tunnel::wireguard::dns64::Dns64Resolution;
use crate::{
    tunnel_state_machine::tunnel::{
        wireguard::{
            connector::ConnectionData,
            fd::DupFd,
            two_hop_config::{TwoHopConfig, ENTRY_MTU, EXIT_MTU},
        },
        Error, Result, Tombstone,
    },
    wg_config::WgNodeConfig,
};

/// Delay before acting on default route changes.
#[cfg(target_os = "ios")]
const DEFAULT_PATH_DEBOUNCE: Duration = Duration::from_millis(250);

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
        ENTRY_MTU
    }

    pub fn exit_mtu(&self) -> u16 {
        // minimum mtu guaranteed by ipv6
        EXIT_MTU
    }

    pub async fn run(
        self,
        tun_device: AsyncDevice,
        dns: Vec<IpAddr>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
    ) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            dns.clone(),
            self.entry_mtu(),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
            dns,
            self.exit_mtu(),
        );

        // Save entry peer so that we can re-resolve it and update wg config on network changes.
        #[cfg(target_os = "ios")]
        let orig_entry_peer = wg_entry_config.peer.clone();

        #[allow(unused_mut)]
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
        let exit_connection = entry_tunnel.open_connection(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
        )?;

        #[allow(unused_mut)]
        let mut exit_tunnel = wireguard_go::Tunnel::start(
            two_hop_config.exit.into_wireguard_config(),
            tun_device.get_ref().dup_fd().map_err(Error::DupFd)?,
        )?;

        let shutdown_token = CancellationToken::new();
        let cloned_shutdown_token = shutdown_token.child_token();

        let event_loop_handle = tokio::spawn(async move {
            #[cfg(target_os = "ios")]
            {
                let (default_path_tx, default_path_rx) = mpsc::unbounded_channel();
                let mut default_path_rx = debounced::debounced(
                    UnboundedReceiverStream::new(default_path_rx),
                    DEFAULT_PATH_DEBOUNCE,
                );

                let queue =
                    Queue::new(Some("net.nymtech.vpn.wg-path-monitor"), QueueAttr::serial())
                        .expect("failed to create dispatch queue");
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
                        _ = cloned_shutdown_token.cancelled() => {
                            tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
                            break;
                        }
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
                        else => {
                            tracing::error!("Default path observer has been dropped. Exiting event loop.");
                            break;
                        }
                    }
                }
            }

            #[cfg(target_os = "android")]
            {
                cloned_shutdown_token.cancelled().await;
                tracing::debug!("Received tunnel shutdown event. Exiting event loop.");
            }

            exit_tunnel.stop();
            exit_connection.close();
            entry_tunnel.stop();

            Tombstone::with_tun_device(tun_device)
        });

        Ok(TunnelHandle {
            task_manager: self.task_manager,
            shutdown_token,
            event_loop_handle,
            bandwidth_controller_handle: self.bandwidth_controller_handle,
        })
    }
}

pub struct TunnelHandle {
    task_manager: TaskManager,
    shutdown_token: CancellationToken,
    event_loop_handle: JoinHandle<Tombstone>,
    bandwidth_controller_handle: JoinHandle<()>,
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
    /// Returns an array with a single tunnel device that is no longer in use.
    pub async fn wait(self) -> Result<Tombstone, JoinError> {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
        }

        self.event_loop_handle.await
    }
}
