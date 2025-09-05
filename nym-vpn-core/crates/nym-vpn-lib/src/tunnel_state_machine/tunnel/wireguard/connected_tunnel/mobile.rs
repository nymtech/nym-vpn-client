// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, SocketAddr};
#[cfg(target_os = "android")]
use std::sync::Arc;
#[cfg(target_os = "ios")]
use std::time::Duration;

#[cfg(target_os = "ios")]
use dispatch2::{DispatchQueue, DispatchQueueAttr};

use ipnetwork::IpNetwork;
use nym_authenticator_client::AuthClientMixnetListenerHandle;
#[cfg(target_os = "ios")]
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle};
#[cfg(target_os = "ios")]
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

#[cfg(target_os = "ios")]
use nym_apple_network::PathMonitor;
use nym_wg_gateway_client::WgGatewayClient;
use nym_wg_go::{netstack, wireguard_go};

#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::tunnel::wireguard::dns64::Dns64Resolution;
use crate::{
    tunnel_state_machine::{
        TunnelConstants,
        tunnel::{
            Error, Result, Tombstone,
            wireguard::{
                connector::ConnectionData,
                fd::DupFd,
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
    entry_gateway_client: WgGatewayClient,
    exit_gateway_client: WgGatewayClient,
    connection_data: ConnectionData,
    bandwidth_controller_handle: JoinHandle<()>,
    auth_client_mixnet_listener_handle: AuthClientMixnetListenerHandle,
}

impl ConnectedTunnel {
    pub fn new(
        entry_gateway_client: WgGatewayClient,
        exit_gateway_client: WgGatewayClient,
        connection_data: ConnectionData,
        bandwidth_controller_handle: JoinHandle<()>,
        auth_client_mixnet_listener_handle: AuthClientMixnetListenerHandle,
    ) -> Self {
        Self {
            entry_gateway_client,
            exit_gateway_client,
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
        // minimum mtu guaranteed by ipv6
        EXIT_MTU
    }

    pub async fn run(
        self,
        tun_device: AsyncDevice,
        dns: Vec<IpAddr>,
        tunnel_constants: TunnelConstants,
        metadata_proxy_tx: tokio::sync::oneshot::Sender<SocketAddr>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
    ) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            AllowedIps::Specific(vec![
                IpNetwork::from(self.connection_data.exit.endpoint.ip()),
                IpNetwork::from(tunnel_constants.in_tunnel_bandwidth_metadata_endpoint.ip()),
            ]),
            dns.clone(),
            self.entry_mtu(),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
            AllowedIps::All,
            dns,
            self.exit_mtu(),
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
        let exit_intunnel_udp_proxy = entry_tunnel.start_intunnel_udp_connection_proxy(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
        )?;

        two_hop_config.set_udp_proxy_listen_addr(exit_intunnel_udp_proxy.listen_addr());

        let entry_magic_bandwidth_tcp_proxy = entry_tunnel.start_intunnel_tcp_connection_proxy(
            tunnel_constants
                .tunnel_constants
                .in_tunnel_bandwidth_metadata_endpoint,
        )?;

        #[allow(unused_mut)]
        let mut exit_tunnel = wireguard_go::Tunnel::start(
            two_hop_config.exit.into_wireguard_config(),
            tun_device.get_ref().dup_fd().map_err(Error::DupFd)?,
        )?;

        let shutdown_token = CancellationToken::new();
        let cloned_shutdown_token = shutdown_token.child_token();

        metadata_proxy_tx
            .send(entry_magic_bandwidth_tcp_proxy.listen_addr())
            .await?;

        let event_loop_handle = tokio::spawn(async move {
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
            entry_magic_bandwidth_tcp_proxy.close();
            exit_intunnel_udp_proxy.close();
            entry_tunnel.stop();

            Tombstone::with_tun_device(tun_device)
        });

        Ok(TunnelHandle {
            shutdown_token,
            event_loop_handle,
            bandwidth_controller_handle: self.bandwidth_controller_handle,
            auth_client_mixnet_listener_handle: self.auth_client_mixnet_listener_handle,
        })
    }
}

pub struct TunnelHandle {
    shutdown_token: CancellationToken,
    event_loop_handle: JoinHandle<Tombstone>,
    bandwidth_controller_handle: JoinHandle<()>,
    auth_client_mixnet_listener_handle: AuthClientMixnetListenerHandle,
}

impl TunnelHandle {
    /// Close entry and exit WireGuard tunnels and signal mixnet facilities shutdown.
    pub fn cancel(&mut self) {
        self.shutdown_token.cancel();
    }

    /// Wait until the tunnel finished execution.
    ///
    /// Returns an array with a single tunnel device that is no longer in use.
    pub async fn wait(self) -> Result<Tombstone, JoinError> {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
        }

        // No need to call cancel on auth_clients_mixnet_listener_handle as its external
        // cancel_token should already be cancelled by the time we reach this point.
        // We just need to wait for the task to finish.
        self.auth_client_mixnet_listener_handle.wait().await;

        self.event_loop_handle.await
    }
}
