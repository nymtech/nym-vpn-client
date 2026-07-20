// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::{Ipv4Addr, Ipv6Addr};
#[cfg(any(target_os = "linux", target_os = "ios", target_os = "android"))]
use std::os::fd::BorrowedFd;
#[cfg(any(target_os = "android", target_os = "ios"))]
use std::os::fd::{AsRawFd, IntoRawFd};
#[cfg(target_os = "android")]
use std::os::fd::{FromRawFd, OwnedFd};
use std::{
    net::{IpAddr, SocketAddr},
    ops::Deref,
    pin::pin,
    time::Duration,
};
#[cfg(unix)]
use std::{os::fd::RawFd, sync::Arc};

use futures::{FutureExt, StreamExt, future::Fuse};
#[cfg(any(target_os = "ios", target_os = "android"))]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(target_os = "linux")]
use nix::sys::socket::{SetSockOpt, sockopt::Mark};
use nym_bandwidth_controller::requests::BandwidthControllerRequestSender;
use nym_gateway_directory::{
    GatewayCacheHandle, GatewayClient, GatewayMinPerformance, NodeIdentity,
};
use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tun::AbstractDevice;
#[cfg(windows)]
use tun::AbstractDeviceExt;
use tun::AsyncDevice;
#[cfg(windows)]
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;

use nym_authenticator_client::AuthClientMixnetListenerHandle;
use nym_common::{ErrorExt, trace_err_chain};
use nym_connection_monitor::{
    ConnectionEvent, ConnectionMonitor, ConnectionStatusEvent, IcmpProbe, IcmpProbeConfig,
    TcpProbe, TcpProbeConfig, TimingConfig,
};
use nym_registration_client::{
    MixnetRegistrationResult, RegistrationClientBuilder, RegistrationClientBuilderConfig,
    RegistrationMode, RegistrationNymNode, RegistrationResult, WireguardRegistrationResult,
};
use nym_vpn_account_controller::{AccountCommandSender, AccountStateReceiver};
use nym_vpn_lib_types::{
    AccountControllerError, BridgeAddress, ConnectionData, ErrorStateReason,
    EstablishConnectionData, GatewayLightInfo, MixnetConnectionData, NymAddress,
    TunnelConnectionData, TunnelType, WireguardConnectionData, WireguardNode,
};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use super::route_handler::{RouteHandler, RoutingConfig};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use super::tun_ipv6;
#[cfg(any(target_os = "ios", target_os = "android"))]
use super::tun_name;
#[cfg(not(any(target_os = "ios", target_os = "android")))]
use super::tunnel::wireguard::connected_tunnel::TunTunTunnelOptions;
#[cfg(windows)]
use super::wintun::{self, WintunAdapterConfig};
use super::{
    Error, NymConfig, Result, TunnelInterface, TunnelMetadata, TunnelSettings,
    tunnel::{
        self, AnyTunnelHandle, SelectedGateways, Tombstone,
        wireguard::{
            connected_tunnel,
            connected_tunnel::{NetstackTunnelOptions, TunnelOptions},
        },
    },
};
#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::OSTunProvider;
use crate::{
    DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE, UserAgent,
    bandwidth_monitor::BandwidthMonitor,
    mixnet::VpnTopologyServiceHandle,
    tunnel_health::{METADATA_PATH_HEALTH_GRACE, MetadataPathHealth, should_defer_probe_teardown},
    tunnel_state_machine::{
        TunnelConstants, WireguardMultihopMode, account, ipv6_availability,
        tunnel::{
            gateway_provider::GatewayProvider,
            mixnet,
            transports::{self, TransportError},
            wireguard::{
                ConnectionData as WgConnectionData, MetadataEvent, MetadataReceiver,
                connected_tunnel::ConnectedTunnel,
            },
        },
    },
};

/// Default MTU for mixnet tun device.
const DEFAULT_TUN_MTU: u16 = if cfg!(any(target_os = "ios", target_os = "android")) {
    1280
} else {
    1500
};

/// User-facing tunnel type identifier.
#[cfg(windows)]
const WINTUN_TUNNEL_TYPE: &str = "Nym";

/// The user-facing name of wintun adapter.
///
/// Note that it refers to tunnel type because rust-tun uses the same name for adapter and
/// tunnel type and there is no way to change that.
#[cfg(windows)]
const MIXNET_WINTUN_NAME: &str = WINTUN_TUNNEL_TYPE;

/// The user-facing name of wintun adapter used as entry tunnel.
#[cfg(windows)]
const WG_ENTRY_WINTUN_NAME: &str = "WireGuard (entry)";

/// The user-facing name of wintun adapter used as exit tunnel.
#[cfg(windows)]
const WG_EXIT_WINTUN_NAME: &str = "WireGuard (exit)";

/// WireGuard entry adapter GUID.
#[cfg(windows)]
const WG_ENTRY_WINTUN_GUID: &str = "{AFE43773-E1F8-4EBB-8536-176AB86AFE9B}";

/// WireGuard exit adapter GUID.
#[cfg(windows)]
const WG_EXIT_WINTUN_GUID: &str = "{AFE43773-E1F8-4EBB-8536-176AB86AFE9C}";

pub type TunnelMonitorEventSender = mpsc::UnboundedSender<TunnelMonitorEvent>;
pub type TunnelMonitorEventReceiver = mpsc::UnboundedReceiver<TunnelMonitorEvent>;

/// Timeout when waiting for reply from the event handler.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

const METADATA_ENDPOINT_REACHABILITY_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for starting the registration client
const REGISTRATION_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug)]
pub enum TunnelMonitorEvent {
    /// Checking account
    AwaitingAccountReadiness,

    /// Refreshing gateways
    RefreshingGateways,

    /// Selecting gateways
    SelectingGateways,

    /// Selected gateways
    SelectedGateways {
        gateways: Box<SelectedGateways>,
        /// Back channel to acknowledge that the event has been processed
        reply_tx: tokio::sync::oneshot::Sender<()>,
    },

    /// Registering with gateways
    RegisteringWithGateways,

    /// Registration with gateways failed
    RegistrationFailed {
        /// The gateway that failed registration
        gateway_id: NodeIdentity,
    },

    /// Finished gateway registration
    RegisteredWithGateways {
        /// Connection data
        connection_data: Box<EstablishConnectionData>,
        /// Back channel to acknowledge that the event has been processed
        reply_tx: tokio::sync::oneshot::Sender<()>,
    },

    /// Tunnel interface is up.
    InterfaceUp {
        /// Tunnel interface
        tunnel_interface: TunnelInterface,
        /// Connection data
        connection_data: Box<EstablishConnectionData>,
        /// Back channel to acknowledge that the event has been processed
        reply_tx: tokio::sync::oneshot::Sender<()>,
    },

    /// Tunnel is up and functional.
    Up {
        /// Tunnel interface
        tunnel_interface: TunnelInterface,
        /// Connection data
        connection_data: Box<ConnectionData>,
    },

    /// Tunnel went down
    Down {
        /// Error state reason.
        /// When set indicates that the state machine should transition to error state.
        error_state_reason: Option<ErrorStateReason>,
        /// Back channel to acknowledge that the event has been processed
        reply_tx: tokio::sync::oneshot::Sender<()>,
    },

    /// Connection has failed
    ConnectionFailed { exit_gateway_id: NodeIdentity },
}

pub struct TunnelMonitorHandle {
    shutdown_token: CancellationToken,
    join_handle: JoinHandle<Tombstone>,
}

impl TunnelMonitorHandle {
    pub fn cancel(&self) {
        tracing::info!("Cancelling tunnel monitor handle");
        self.shutdown_token.cancel();
    }

    pub async fn wait(self) -> Tombstone {
        self.join_handle
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to join on tunnel monitor handle: {}", e);
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct TunnelParameters {
    pub nym_config: NymConfig,
    pub tunnel_settings: TunnelSettings,
    pub tunnel_constants: TunnelConstants,
    pub selected_gateways: Option<SelectedGateways>,
    pub user_agent: UserAgent,
    #[cfg(not(target_os = "android"))]
    pub filtering_resolver_addr: SocketAddr,
}

pub struct TunnelMonitor {
    tunnel_parameters: TunnelParameters,
    monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    route_handler: RouteHandler,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "android")]
    dns_filter: Option<crate::dns_filter::DnsFilter>,
    account_controller_state: AccountStateReceiver,
    account_command_tx: AccountCommandSender,
    bandwidth_command_tx: BandwidthControllerRequestSender,
    gateway_provider: GatewayProvider<GatewayCacheHandle>,
    custom_topology_provider: VpnTopologyServiceHandle,
    shutdown_token: CancellationToken,
}

/// Poll the exit WireGuard peer's UAPI stats until the handshake completes or we time out.
async fn wait_for_exit_handshake(
    tunnel_handle: &connected_tunnel::TunnelHandle,
    shutdown_token: &CancellationToken,
) {
    const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
    const POLL_INTERVAL: Duration = Duration::from_millis(500);

    tracing::debug!("Waiting for exit WireGuard handshake to complete");

    let started = std::time::Instant::now();

    let result = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
        loop {
            match tunnel_handle.get_exit_stats() {
                Ok(stats) => {
                    if stats.all_peers_connected() {
                        return;
                    }
                }
                Err(err) => {
                    tracing::debug!("Failed to get exit tunnel stats: {err}");
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
                _ = shutdown_token.cancelled() => return,
            }
        }
    })
    .await;

    let elapsed = started.elapsed();
    match result {
        Ok(()) => {
            tracing::debug!("Exit WireGuard handshake completed in {elapsed:.2?}");
        }
        Err(_) => {
            tracing::warn!(
                "Exit WireGuard handshake did not complete within {:.1}s, proceeding",
                elapsed.as_secs_f32()
            );
        }
    }
}

impl TunnelMonitor {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        tunnel_parameters: TunnelParameters,
        account_controller_state: AccountStateReceiver,
        account_command_tx: AccountCommandSender,
        bandwidth_command_tx: BandwidthControllerRequestSender,
        gateway_provider: GatewayProvider<GatewayCacheHandle>,
        custom_topology_provider: VpnTopologyServiceHandle,
        monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
        #[cfg(not(any(target_os = "android", target_os = "ios")))] route_handler: RouteHandler,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        #[cfg(target_os = "android")] dns_filter: Option<crate::dns_filter::DnsFilter>,
    ) -> TunnelMonitorHandle {
        let shutdown_token = CancellationToken::new();
        let tunnel_monitor = Self {
            tunnel_parameters,
            monitor_event_sender,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            #[cfg(target_os = "android")]
            dns_filter,
            account_controller_state,
            account_command_tx,
            bandwidth_command_tx,
            gateway_provider,
            custom_topology_provider,
            shutdown_token: shutdown_token.clone(),
        };
        let join_handle = tokio::spawn(tunnel_monitor.run());

        TunnelMonitorHandle {
            shutdown_token,
            join_handle,
        }
    }

    async fn run(mut self) -> Tombstone {
        let (tombstone, reason) = match Box::pin(self.run_inner()).await {
            Ok(tombstone) => (tombstone, None),
            Err(e) => {
                trace_err_chain!(e, "Tunnel monitor exited with error");
                (Tombstone::default(), e.error_state_reason())
            }
        };

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.send_event(TunnelMonitorEvent::Down {
            error_state_reason: reason,
            reply_tx,
        });
        if tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await.is_err() {
            tracing::warn!("Tunnel down reply timeout.");
        }

        tombstone
    }

    async fn run_inner(&mut self) -> Result<Tombstone> {
        if self.enable_ipv6() && !ipv6_availability::is_ipv6_enabled_in_os().await {
            return Err(Error::Ipv6Unavailable);
        }

        self.send_event(TunnelMonitorEvent::AwaitingAccountReadiness);

        self.shutdown_token
            .clone()
            .run_until_cancelled(self.await_account_readiness_with_retry())
            .await
            .ok_or(tunnel::Error::Cancelled)??;

        self.send_event(TunnelMonitorEvent::RefreshingGateways);

        let gateway_performance_options = self
            .tunnel_parameters
            .tunnel_settings
            .gateway_performance_options;
        let gateway_min_performance = GatewayMinPerformance::from_percentage_values(
            gateway_performance_options
                .mixnet_min_performance
                .map(u64::from),
            gateway_performance_options
                .vpn_min_performance
                .map(u64::from),
        );

        let mut gateway_config = self.tunnel_parameters.nym_config.gateway_config.clone();
        match gateway_min_performance {
            Ok(gateway_min_performance) => {
                gateway_config =
                    gateway_config.with_min_gateway_performance(gateway_min_performance);
            }
            Err(e) => {
                tracing::error!(
                    "Invalid gateway performance values. Will carry on with initial values. Error: {}",
                    e
                );
            }
        }

        let user_agent = self.tunnel_parameters.user_agent.clone();
        let gateway_directory_client =
            GatewayClient::new(gateway_config.clone(), user_agent.clone())
                .map_err(Error::GatewayDirectoryClient)?;

        self.gateway_provider
            .replace_gateway_client(gateway_directory_client)
            .await;

        let selected_gateways =
            if let Some(ref selected_gateways) = self.tunnel_parameters.selected_gateways {
                selected_gateways.clone()
            } else {
                self.send_event(TunnelMonitorEvent::SelectingGateways);

                let new_gateways = self
                    .shutdown_token
                    .run_until_cancelled(self.gateway_provider.next())
                    .await
                    .ok_or(tunnel::Error::Cancelled)?
                    .ok_or(Error::GatewayProviderDown)?
                    .map_err(Box::new)
                    .map_err(tunnel::Error::SelectGateways)?;

                let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                self.send_event(TunnelMonitorEvent::SelectedGateways {
                    gateways: Box::new(new_gateways.clone()),
                    reply_tx,
                });

                // Wait for reply before proceeding to connect to let state machine configure firewall.
                if tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await.is_err() {
                    tracing::warn!("Failed to receive selected gateways reply in time");
                }

                new_gateways
            };

        self.send_event(TunnelMonitorEvent::RegisteringWithGateways);

        #[cfg(target_os = "android")]
        let tun_provider = self.tun_provider.clone();
        #[cfg(target_os = "linux")]
        let fwmark = self.tunnel_parameters.tunnel_constants.fwmark;
        #[cfg(unix)]
        let connection_fd_callback = move |_fd: RawFd| {
            #[cfg(target_os = "android")]
            {
                tracing::debug!("Bypass websocket");
                tun_provider.bypass(_fd);
            }

            #[cfg(target_os = "linux")]
            {
                tracing::debug!("Bypass websocket");
                let borrowed_fd = unsafe { &BorrowedFd::borrow_raw(_fd) };
                if let Err(err) = Mark.set(borrowed_fd, &fwmark) {
                    tracing::error!("Could not set fwmark for websocket fd: {err}");
                }
            }
        };

        let mixnet_client_config = self
            .tunnel_parameters
            .tunnel_settings
            .mixnet_client_config
            .clone()
            .unwrap_or_default();

        tracing::debug!(
            "Mixnet client performance thresholds: min_mixnode={:?}, min_gateway={:?}",
            mixnet_client_config.min_mixnode_performance,
            mixnet_client_config.min_gateway_performance
        );

        let custom_topology_provider = self.custom_topology_provider.clone();
        custom_topology_provider
            .update_config(
                mixnet_client_config
                    .min_mixnode_performance
                    .unwrap_or(DEFAULT_MIN_MIXNODE_PERFORMANCE),
                mixnet_client_config
                    .min_gateway_performance
                    .unwrap_or(DEFAULT_MIN_GATEWAY_PERFORMANCE),
            )
            .await;

        tracing::debug!(
            "Connecting to entry gateway: {}",
            selected_gateways
                .entry_gateway()
                .identity()
                .to_base58_string()
        );
        tracing::debug!(
            "Connecting to exit gateway: {}",
            selected_gateways
                .exit_gateway()
                .identity()
                .to_base58_string()
        );

        let entry_node =
            RegistrationNymNode::try_from(selected_gateways.entry().clone()).map_err(Box::new)?;

        let exit_node =
            RegistrationNymNode::try_from(selected_gateways.exit().clone()).map_err(Box::new)?;

        let network_env = self
            .tunnel_parameters
            .nym_config
            .network_rx
            .borrow()
            .clone();
        let nym_network = network_env.nym_network.clone();
        let mode = match self.tunnel_parameters.tunnel_settings.tunnel_type_used() {
            TunnelType::Mixnet => RegistrationMode::Mixnet,
            TunnelType::Wireguard => RegistrationMode::Wireguard,
        };
        let rcb_config_builder = RegistrationClientBuilderConfig::builder()
            .entry_node(entry_node)
            .exit_node(exit_node)
            .data_path(
                self.tunnel_parameters
                    .nym_config
                    .paths
                    .network_data_dir
                    .clone(),
            )
            .mixnet_client_config(mixnet_client_config)
            .mixnet_client_startup_timeout(REGISTRATION_CLIENT_STARTUP_TIMEOUT)
            .mode(mode)
            .bandwidth_request_sender(self.bandwidth_command_tx.clone())
            .enable_lp_registration(true)
            .user_agent(user_agent)
            .custom_topology_provider(Box::new(
                self.custom_topology_provider.make_topology_provider(),
            ))
            .network_env(nym_network)
            .cancel_token(self.shutdown_token.child_token());

        #[cfg(unix)]
        let rcb_config_builder =
            rcb_config_builder.connection_fd_callback(Arc::new(connection_fd_callback));

        let rc_builder_config = rcb_config_builder.build();

        // Setup shutdown guard to cancel pending tasks that otherwise may continue running upon return
        let shutdown_guard = self.shutdown_token.clone().drop_guard();

        let rc_builder = RegistrationClientBuilder::new(rc_builder_config);

        let registration_client = Box::pin(rc_builder.build()).await?;
        let registration_result =
            Box::pin(registration_client.register())
                .await
                .inspect_err(|_| {
                    self.send_event(TunnelMonitorEvent::RegistrationFailed {
                        gateway_id: selected_gateways.entry_gateway().identity(),
                    })
                })?;

        // Send event upon successful gateway registration
        // The receiver should handle the event and add firewall exceptions for entry gateway
        let tunnel_connection_data = match &registration_result {
            RegistrationResult::Mixnet(result) => {
                TunnelConnectionData::Mixnet(MixnetConnectionData {
                    nym_address: NymAddress::from(result.assigned_addresses.mixnet_client_address),
                    exit_ipr: NymAddress::from(result.assigned_addresses.exit_mix_address),
                    entry_ip: result.assigned_addresses.entry_mixnet_gateway_ip,
                    exit_ip: result.assigned_addresses.exit_mixnet_gateway_ip,
                    ipv4: result.assigned_addresses.interface_addresses.ipv4,
                    ipv6: self
                        .tunnel_parameters
                        .tunnel_settings
                        .enable_ipv6
                        .then_some(result.assigned_addresses.interface_addresses.ipv6),
                })
            }
            RegistrationResult::Wireguard(result) => {
                TunnelConnectionData::Wireguard(WireguardConnectionData {
                    entry_bridge_addr: None, // not known yet
                    entry: WireguardNode::from(result.entry_gateway_data()),
                    exit: WireguardNode::from(result.exit_gateway_data()),
                })
            }
        };
        let connection_data = Box::new(EstablishConnectionData {
            entry_gateway: GatewayLightInfo::from(selected_gateways.entry_gateway().clone()),
            exit_gateway: GatewayLightInfo::from(selected_gateways.exit_gateway().clone()),
            tunnel: Some(tunnel_connection_data),
        });
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.send_event(TunnelMonitorEvent::RegisteredWithGateways {
            connection_data,
            reply_tx,
        });
        if tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await.is_err() {
            tracing::warn!("Registered with gateways reply timeout");
        }

        let (entry_metadata_tx, entry_metadata_rx) =
            tokio::sync::oneshot::channel::<MetadataEvent>();
        let (exit_metadata_tx, exit_metadata_rx) = tokio::sync::oneshot::channel::<MetadataEvent>();

        let (entry_metadata_addr_tx, entry_metadata_addr_rx) = tokio::sync::oneshot::channel();
        let (bridge_close_tx, mut bridge_close_rx) = mpsc::unbounded_channel();

        // todo: refactor
        let (
            StartTunnelResult {
                tunnel_interface,
                tunnel_conn_data,
                mut tunnel_handle,
            },
            wg_tunnel_runtime,
            mixnet_client_token,
            _bridge_close_tx,
        ) = match registration_result {
            RegistrationResult::Mixnet(inner_result) => {
                let mixnet_client_token = inner_result.mixnet_client.cancellation_token();

                (
                    self.start_mixnet_tunnel(*inner_result).await?,
                    None,
                    Some(mixnet_client_token),
                    // Return sender back to avoid it being dropped
                    Some(bridge_close_tx),
                )
            }
            RegistrationResult::Wireguard(inner_result) => {
                let (mut connection_data, mut wg_tunnel_runtime) = self
                    .setup_wireguard_tunnel(
                        *inner_result,
                        entry_metadata_rx,
                        exit_metadata_rx,
                        &selected_gateways,
                    )
                    .await?;

                let bridge_close_tx = if self.tunnel_parameters.tunnel_settings.bridges_enabled() {
                    let (entry_bridge_addr, transport_fwd_handle) = self
                        .start_bridges(&selected_gateways, bridge_close_tx)
                        .await?;

                    wg_tunnel_runtime.transport_fwd_handle = Some(transport_fwd_handle);
                    connection_data.entry_bridge_addr = Some(entry_bridge_addr);

                    None
                } else {
                    // Return bridge_close_tx back to avoid it being dropped
                    Some(bridge_close_tx)
                };

                let connected_tunnel = ConnectedTunnel::new(
                    selected_gateways.entry_keypair().clone(),
                    selected_gateways.exit_keypair().clone(),
                    connection_data,
                );

                let start_tunnel_result = match self
                    .tunnel_parameters
                    .tunnel_settings
                    .wireguard_tunnel_options
                    .multihop_mode
                {
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    WireguardMultihopMode::TunTun => {
                        self.start_wireguard_tunnel(connected_tunnel).await?
                    }
                    WireguardMultihopMode::Netstack => {
                        self.start_wireguard_netstack_tunnel(
                            connected_tunnel,
                            entry_metadata_addr_tx,
                        )
                        .await?
                    }
                };

                let mixnet_client_token = wg_tunnel_runtime.mixnet_client_token();

                (
                    start_tunnel_result,
                    Some(wg_tunnel_runtime),
                    mixnet_client_token,
                    bridge_close_tx,
                )
            }
        };

        let establishing_connection_data = EstablishConnectionData {
            entry_gateway: GatewayLightInfo::from(selected_gateways.entry_gateway().clone()),
            exit_gateway: GatewayLightInfo::from(selected_gateways.exit_gateway().clone()),
            tunnel: Some(tunnel_conn_data.clone()),
        };

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.send_event(TunnelMonitorEvent::InterfaceUp {
            tunnel_interface: tunnel_interface.clone(),
            connection_data: Box::new(establishing_connection_data),
            reply_tx,
        });

        if tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await.is_err() {
            tracing::warn!("Interface up reply timeout");
        }

        // The firewall now allows traffic through the tunnel. Wait for the exit WG handshake.
        if let Some(wg_handle) = tunnel_handle.as_wireguard() {
            wait_for_exit_handshake(wg_handle, &self.shutdown_token).await;
        }

        let (entry_metadata_endpoint_reachable_tx, entry_metadata_endpoint_reachable_rx) =
            tokio::sync::oneshot::channel::<bool>();
        let (exit_metadata_endpoint_reachable_tx, exit_metadata_endpoint_reachable_rx) =
            tokio::sync::oneshot::channel::<bool>();
        let uses_metadata_endpoint = wg_tunnel_runtime.is_some();
        let metadata_endpoints_reachable =
            tokio::time::timeout(METADATA_ENDPOINT_REACHABILITY_TIMEOUT, async move {
                // for mixnet tunnel, we don't have metadata endpoints, so we just return true
                if !uses_metadata_endpoint {
                    return true;
                }
                let entry_reachable = entry_metadata_endpoint_reachable_rx.await.unwrap_or(false);
                let exit_reachable = exit_metadata_endpoint_reachable_rx.await.unwrap_or(false);

                entry_reachable && exit_reachable
            })
            .fuse();
        tokio::pin!(metadata_endpoints_reachable);

        // Send metadata endpoint data to the bandwidth monitor
        match &tunnel_interface {
            TunnelInterface::One(exit) => {
                let _metadata_event_handler = tokio::spawn(async move {
                    if let Ok(entry_metadata_endpoint) = entry_metadata_addr_rx.await {
                        tracing::info!(
                            "Received entry metadata endpoint: {entry_metadata_endpoint}"
                        );
                        entry_metadata_tx
                            .send(MetadataEvent::new_proxy(
                                entry_metadata_endpoint_reachable_tx,
                                entry_metadata_endpoint,
                            ))
                            .ok();
                    }
                });
                exit_metadata_tx
                    .send(MetadataEvent::new_tunnel(
                        exit_metadata_endpoint_reachable_tx,
                        exit.clone(),
                    ))
                    .ok();
            }
            TunnelInterface::Two { entry, exit } => {
                entry_metadata_tx
                    .send(MetadataEvent::new_tunnel(
                        entry_metadata_endpoint_reachable_tx,
                        entry.clone(),
                    ))
                    .ok();
                exit_metadata_tx
                    .send(MetadataEvent::new_tunnel(
                        exit_metadata_endpoint_reachable_tx,
                        exit.clone(),
                    ))
                    .ok();
            }
        }

        let mixnet_monitoring_token = mixnet_client_token
            .map(|token| token.cancelled_owned().fuse())
            .unwrap_or(Fuse::terminated());
        let mut mixnet_monitoring_token = pin!(mixnet_monitoring_token);

        let (tunnel_connection_monitor_tx, mut tunnel_connection_monitor_rx) =
            mpsc::unbounded_channel();
        let tunnel_connection_monitor_handle = self.create_tunnel_connection_monitor(
            tunnel_interface.exit_tunnel_metadata(),
            tunnel_connection_monitor_tx,
        )?;

        let mut last_connection_status = None;
        let mut has_sent_up_event = false;
        let mut ping_viable = false;
        let mut metadata_endpoint_viable = false;
        let mut consecutive_deferred_probe_failures = 0u32;
        let metadata_path_health = wg_tunnel_runtime
            .as_ref()
            .map(|runtime| runtime.metadata_path_health.clone());
        let connection_data = Box::new(ConnectionData {
            entry_gateway: GatewayLightInfo::from(selected_gateways.entry_gateway().clone()),
            exit_gateway: GatewayLightInfo::from(selected_gateways.exit_gateway().clone()),
            connected_at: OffsetDateTime::now_utc(),
            tunnel: tunnel_conn_data,
        });

        loop {
            tokio::select! {
                event = tunnel_connection_monitor_rx.recv() => {
                    let Some(event) = event else {
                        tracing::info!("Event channel with connection monitor is closed");
                        break;
                    };
                    // Prevent repeated messages
                    if last_connection_status != Some(event.status) {
                        last_connection_status = Some(event.status);

                        match event.status {
                            ConnectionStatusEvent::Viable => {
                                ping_viable = true;
                                consecutive_deferred_probe_failures = 0;
                                if !has_sent_up_event && metadata_endpoint_viable {
                                    tracing::info!("Tunnel connection is viable");
                                    has_sent_up_event = true;
                                    self.send_event(TunnelMonitorEvent::Up {
                                        tunnel_interface: tunnel_interface.clone(),
                                        connection_data: connection_data.clone(),
                                    });
                                }
                            }
                            ConnectionStatusEvent::IntermittentFailure { retry } => {
                                tracing::info!("Tunnel connection is failing (retry: {retry})");
                            }
                            ConnectionStatusEvent::Failed => {
                                if should_defer_probe_teardown(
                                    uses_metadata_endpoint,
                                    metadata_path_health.as_ref(),
                                    METADATA_PATH_HEALTH_GRACE,
                                    consecutive_deferred_probe_failures,
                                ) {
                                    consecutive_deferred_probe_failures =
                                        consecutive_deferred_probe_failures.saturating_add(1);
                                    tracing::warn!(
                                        consecutive_deferred_probe_failures,
                                        max_consecutive_deferred_probe_failures =
                                            crate::tunnel_health::MAX_CONSECUTIVE_DEFERRED_PROBE_FAILURES,
                                        "Probe declared tunnel down but in-tunnel metadata path recently succeeded; deferring teardown"
                                    );
                                    last_connection_status = None;
                                } else {
                                    tracing::info!("Tunnel connection is down. Exiting");
                                    self.send_event(TunnelMonitorEvent::ConnectionFailed {
                                        exit_gateway_id: selected_gateways
                                            .exit_gateway()
                                            .identity(),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
                reachable = &mut metadata_endpoints_reachable => {
                    let reachable = reachable.unwrap_or(false);
                    if !reachable {
                        if let Some(health) = metadata_path_health.as_ref() {
                            health.clear_health();
                        }
                        tracing::info!("Metadata endpoints not reachable. Exiting");
                        self.send_event(TunnelMonitorEvent::ConnectionFailed {
                            exit_gateway_id: selected_gateways.exit_gateway().identity(),
                        });
                        break;
                    } else {
                        metadata_endpoint_viable = true;
                        consecutive_deferred_probe_failures = 0;
                        if !has_sent_up_event && ping_viable {
                            tracing::info!("Tunnel connection is viable");
                            has_sent_up_event = true;
                            self.send_event(TunnelMonitorEvent::Up {
                                tunnel_interface: tunnel_interface.clone(),
                                connection_data: connection_data.clone(),
                            });
                        }
                    }
                }
                close_event = bridge_close_rx.recv() => {
                    if close_event.is_some() {
                        tracing::info!("Bridge close signal received. Exiting");
                    } else {
                        tracing::info!("Bridge channel is closed. Exiting");
                    }
                    break;
                }
                _  = &mut mixnet_monitoring_token => {
                    tracing::error!("MixnetClient exited unexpectedly");
                    break;
                }
                _ = self.shutdown_token.cancelled() => {
                    break;
                }
            }
        }

        // Trigger cancellation since many other tasks depend on shutdown token
        drop(shutdown_guard);

        // Shutdown WireGuard tunnel runtime
        if let Some(wg_tunnel_runtime) = wg_tunnel_runtime {
            if let Err(err) = wg_tunnel_runtime.bandwidth_monitor_handle.await {
                tracing::error!("Failed to await bandwidth monitor handle: {}", err);
            }

            if let Some(transport_fwd_handle) = wg_tunnel_runtime.transport_fwd_handle
                && let Err(err) = transport_fwd_handle.await
            {
                tracing::error!("Failed to await transport forward handle: {}", err);
            }

            if let Some(authenticator_listener_handle) =
                wg_tunnel_runtime.authenticator_listener_handle
            {
                authenticator_listener_handle.stop().await;
            }
        }

        if let Err(e) = tunnel_connection_monitor_handle.await {
            tracing::error!("Tunnel connection monitor exited with error: {}", e);
        }

        tracing::info!("Waiting for tunnel to exit");
        tunnel_handle.cancel();

        let tun_devices = tunnel_handle
            .wait()
            .await
            .inspect_err(|e| {
                trace_err_chain!(e, "Failed to gracefully shutdown the tunnel");
            })
            .unwrap_or_default();

        tracing::info!("Tunnel monitor finished");

        Ok(tun_devices)
    }

    async fn await_account_readiness_with_retry(&mut self) -> Result<(), Error> {
        match self
            .account_controller_state
            .wait_for_account_ready_to_connect()
            .await
        {
            Ok(()) => Ok(()),
            Err(AccountControllerError::ErrorState(reason)) if reason.is_retryable() => {
                tracing::debug!(
                    "Account controller is in a retryable error state : {reason}. Forcing a refresh"
                );
                self.account_command_tx
                    .refresh_account_state(false)
                    .await
                    .map_err(|e| Error::Account(account::Error::Command(e)))?;
                self.account_controller_state
                    .wait_for_account_ready_to_connect()
                    .await
            }
            Err(e) => Err(e),
        }
        .map_err(|e| Error::Account(account::Error::ControllerState(e)))
    }

    fn send_event(&mut self, event: TunnelMonitorEvent) {
        if let Err(e) = self.monitor_event_sender.send(event)
            && !self.shutdown_token.is_cancelled()
        {
            tracing::error!("Failed to send monitor event: {}", e);
        }
    }

    async fn start_mixnet_tunnel(
        &mut self,
        registration_result: MixnetRegistrationResult,
    ) -> Result<StartTunnelResult> {
        let assigned_addresses = registration_result.assigned_addresses;
        let mtu = if let Some(mtu) = self
            .tunnel_parameters
            .tunnel_settings
            .mixnet_tunnel_options
            .mtu
        {
            mtu
        } else {
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                use nym_common::ErrorExt;
                self.route_handler
                    .get_mtu_for_route(assigned_addresses.entry_mixnet_gateway_ip)
                    .await
                    .inspect_err(|e| {
                        tracing::warn!(
                            "{}",
                            e.display_chain_with_msg("Failed to detect mtu for route")
                        );
                    })
                    .unwrap_or(DEFAULT_TUN_MTU)
            }

            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                DEFAULT_TUN_MTU
            }
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let tun_device = Self::create_mixnet_device(
            assigned_addresses.interface_addresses.ipv4,
            self.enable_ipv6()
                .then_some(assigned_addresses.interface_addresses.ipv6),
            mtu,
        )
        .await?;

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let tun_device = {
            let mut interface_addresses = vec![IpNetwork::V4(Ipv4Network::from(
                assigned_addresses.interface_addresses.ipv4,
            ))];

            if self.enable_ipv6() {
                interface_addresses.push(IpNetwork::V6(Ipv6Network::from(
                    assigned_addresses.interface_addresses.ipv6,
                )));
            }

            let dns_servers = self.get_dns_addresses();
            let packet_tunnel_settings = crate::tunnel_provider::TunnelSettings {
                dns_servers,
                interface_addresses,
                remote_addresses: vec![assigned_addresses.entry_mixnet_gateway_ip],
                mtu,
            };

            self.create_tun_device(packet_tunnel_settings).await?
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let tun_name = tun_device
            .deref()
            .tun_name()
            .map_err(Error::GetTunDeviceName)?;

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let tun_name = {
            let tun_fd = unsafe { BorrowedFd::borrow_raw(tun_device.deref().as_raw_fd()) };
            tun_name::get_tun_name(&tun_fd).map_err(Error::GetTunDeviceName)?
        };

        tracing::info!("Created tun device: {}", tun_name);

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let routing_config = RoutingConfig::Mixnet {
                tun_name: tun_name.clone(),
                tun_mtu: mtu,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address: assigned_addresses.entry_mixnet_gateway_ip,
            };

            self.set_routes(routing_config, self.enable_ipv6()).await?;
        }

        let tunnel_conn_data = TunnelConnectionData::Mixnet(MixnetConnectionData {
            nym_address: NymAddress::from(assigned_addresses.mixnet_client_address),
            exit_ipr: NymAddress::from(assigned_addresses.exit_mix_address),
            entry_ip: assigned_addresses.entry_mixnet_gateway_ip,
            exit_ip: assigned_addresses.exit_mixnet_gateway_ip,
            ipv4: assigned_addresses.interface_addresses.ipv4,
            ipv6: self
                .tunnel_parameters
                .tunnel_settings
                .enable_ipv6
                .then_some(assigned_addresses.interface_addresses.ipv6),
        });

        let mut ips = vec![IpAddr::V4(assigned_addresses.interface_addresses.ipv4)];
        if self.enable_ipv6() {
            ips.push(IpAddr::V6(assigned_addresses.interface_addresses.ipv6));
        }
        let tunnel_metadata = TunnelMetadata {
            interface: tun_name,
            ips,
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        let tunnel_handle = mixnet::connected_tunnel::start_mixnet_tunnel(
            registration_result.mixnet_client,
            assigned_addresses,
            tun_device,
            self.shutdown_token.child_token(),
            registration_result.event_rx,
        )
        .await
        .map_err(|e| Error::Tunnel(Box::new(e)))?;

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_conn_data,
            tunnel_handle: AnyTunnelHandle::from(tunnel_handle),
        })
    }

    async fn setup_wireguard_tunnel(
        &self,
        registration_result: WireguardRegistrationResult,
        entry_metadata_rx: MetadataReceiver,
        exit_metadata_rx: MetadataReceiver,
        selected_gateways: &SelectedGateways,
    ) -> Result<(WgConnectionData, WgTunnelRuntime)> {
        let (entry_signal_tx, entry_signal_rx) = tokio::sync::oneshot::channel();
        let (exit_signal_tx, exit_signal_rx) = tokio::sync::oneshot::channel();

        let _metadata_event_handler = tokio::spawn(async move {
            if let Ok(entry) = entry_metadata_rx.await {
                entry_signal_tx.send(entry.into()).ok();
            }
            if let Ok(exit) = exit_metadata_rx.await {
                exit_signal_tx.send(exit.into()).ok();
            }
        });

        let (
            entry_gateway_client,
            exit_gateway_client,
            entry_gateway_data,
            exit_gateway_data,
            authenticator_listener_handle,
        ) = match registration_result {
            WireguardRegistrationResult::Legacy(res) => (
                Some(res.entry_gateway_client),
                Some(res.exit_gateway_client),
                res.entry_gateway_data,
                res.exit_gateway_data,
                Some(res.authenticator_listener_handle),
            ),
            WireguardRegistrationResult::LewesProtocol(res) => (
                None,
                None,
                res.entry_gateway_data,
                res.exit_gateway_data,
                None,
            ),
        };

        let gw_update_version = self
            .tunnel_parameters
            .nym_config
            .network_rx
            .borrow()
            .gw_update_version();

        let metadata_path_health = MetadataPathHealth::new();

        let bw = BandwidthMonitor::create(
            Box::new(self.bandwidth_command_tx.clone()),
            self.account_command_tx.clone(),
            selected_gateways,
            entry_gateway_client,
            exit_gateway_client,
            &entry_gateway_data,
            &exit_gateway_data,
            entry_signal_rx,
            exit_signal_rx,
            gw_update_version,
            self.shutdown_token.clone(),
            metadata_path_health.clone(),
        );

        let authenticator_listener_handle = match authenticator_listener_handle {
            Some(handle) if bw.is_using_latest_client() => {
                // We don't need the mixnet client anymore
                tracing::info!(
                    "Disconnecting mixnet client as we are using the latest bandwidth monitor"
                );
                handle.stop().await;
                None
            }
            Some(handle) => Some(handle),
            None => None,
        };
        let bandwidth_monitor_handle = tokio::spawn(bw.run());

        let rt = WgTunnelRuntime {
            bandwidth_monitor_handle,
            transport_fwd_handle: None,
            authenticator_listener_handle,
            metadata_path_health,
        };

        let connection_data = WgConnectionData {
            entry_bridge_addr: None,
            entry: entry_gateway_data,
            exit: exit_gateway_data,
        };

        Ok((connection_data, rt))
    }

    async fn start_bridges(
        &self,
        selected_gateways: &SelectedGateways,
        bridge_close_tx: mpsc::UnboundedSender<()>,
    ) -> Result<(BridgeAddress, JoinHandle<()>)> {
        let entry_bridge_params = selected_gateways
            .entry_gateway()
            .get_bridge_params()
            .ok_or(TransportError::config_err(
                "attempted to open transport connection without bridge params",
            ))?;

        // Attempt transport Connection. If successful a listening UDP connection is created
        // and the bind address of that UDP listener is provided to the entry wireguard tunnel
        // as the endpoint address.
        tracing::info!("Establishing DVPN QUIC transport tunnel");

        #[cfg(target_os = "linux")]
        let fwmark = self.tunnel_parameters.tunnel_constants.fwmark;
        #[cfg(target_os = "android")]
        let tun_provider = self.tun_provider.clone();
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let on_quic_socket_open = move |fd| {
            #[cfg(target_os = "android")]
            {
                tracing::debug!("Bypass quic socket");
                tun_provider.bypass(fd);
            }

            #[cfg(target_os = "linux")]
            {
                tracing::debug!("Bypass quic socket");
                let borrowed_fd = unsafe { &BorrowedFd::borrow_raw(fd) };
                if let Err(err) = Mark.set(borrowed_fd, &fwmark) {
                    tracing::error!("Could not set fwmark for quic socket fd: {err}");
                }
            }
        };
        let bridge_conn = transports::BridgeConn::try_connect(
            entry_bridge_params,
            self.shutdown_token.child_token(),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            on_quic_socket_open,
        )
        .await?;
        let remote_addr = bridge_conn.endpoint;
        let (listen_addr, join_handle) = transports::UdpForwarder::launch(
            bridge_conn,
            None,
            bridge_close_tx,
            self.shutdown_token.child_token(),
        )
        .await?;

        tracing::info!("quic transport connected, udp forwarder open on {listen_addr}");

        let bridge_addr = BridgeAddress {
            listen_addr,
            remote_addr,
        };

        Ok((bridge_addr, join_handle))
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_tunnel: ConnectedTunnel,
        entry_metadata_tx: tokio::sync::oneshot::Sender<SocketAddr>,
    ) -> Result<StartTunnelResult> {
        let conn_data = connected_tunnel.connection_data();
        let use_bridges = self.tunnel_parameters.tunnel_settings.bridges_enabled();

        // Prepare network environment for the wireguard connection to the entry gateway
        let exit_tun_mtu = connected_tunnel.exit_mtu();
        let exit_tun = Self::create_wireguard_device(
            conn_data.exit.private_ipv4,
            self.enable_ipv6().then_some(conn_data.exit.private_ipv6),
            Some(conn_data.entry.private_ipv4.into()),
            exit_tun_mtu,
        )?;
        let exit_tun_name = exit_tun
            .deref()
            .tun_name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        #[cfg(not(target_os = "linux"))]
        let entry_endpoint = conn_data.effective_remote_entry_endpoint();

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: exit_tun_name.clone(),
            exit_tun_mtu,
            #[cfg(not(target_os = "linux"))]
            entry_gateway_address: entry_endpoint.ip(),
        };

        self.set_routes(routing_config, self.enable_ipv6()).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr.clone(),
            entry: WireguardNode::from(&conn_data.entry),
            exit: WireguardNode::from(&conn_data.exit),
        });

        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            metadata_proxy_tx: entry_metadata_tx,
            exit_tun,
            dns: self.get_dns_addresses(),
        });

        let tunnel_metadata = TunnelMetadata {
            interface: exit_tun_name,
            ips: vec![
                IpAddr::V4(conn_data.exit.private_ipv4),
                IpAddr::V6(conn_data.exit.private_ipv6),
            ],
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: Some(conn_data.entry.private_ipv6),
        };

        let tunnel_handle = connected_tunnel
            .run(
                tunnel_options,
                self.tunnel_parameters.tunnel_constants,
                !use_bridges,
            )
            .await?;
        let tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_conn_data,
            tunnel_handle,
        })
    }

    #[cfg(windows)]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_tunnel: ConnectedTunnel,
        entry_metadata_tx: tokio::sync::oneshot::Sender<SocketAddr>,
    ) -> Result<StartTunnelResult> {
        let conn_data = connected_tunnel.connection_data();
        let use_bridges = self.tunnel_parameters.tunnel_settings.bridges_enabled();

        // Prepare network environment for the wireguard connection to the entry gateway
        let exit_mtu = connected_tunnel.exit_mtu();

        let exit_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.exit.private_ipv4,
            interface_ipv6: self
                .tunnel_parameters
                .tunnel_settings
                .enable_ipv6
                .then_some(conn_data.exit.private_ipv6),
            gateway_ipv4: Some(conn_data.entry.private_ipv4),
            gateway_ipv6: self
                .tunnel_parameters
                .tunnel_settings
                .enable_ipv6
                .then_some(conn_data.entry.private_ipv6),
        };
        let mut tunnel_metadata = TunnelMetadata {
            interface: "".to_owned(),
            ips: vec![
                IpAddr::V4(conn_data.exit.private_ipv4),
                IpAddr::V6(conn_data.exit.private_ipv6),
            ],
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: Some(conn_data.entry.private_ipv6),
        };

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr.clone(),
            entry: WireguardNode::from(&conn_data.entry),
            exit: WireguardNode::from(&conn_data.exit),
        });

        #[cfg(not(target_os = "linux"))]
        let entry_endpoint = conn_data.effective_remote_entry_endpoint();

        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            metadata_proxy_tx: entry_metadata_tx,
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: self.get_dns_addresses(),
        });

        let mut tunnel_handle = connected_tunnel
            .run(
                #[cfg(windows)]
                self.route_handler.clone(),
                tunnel_options,
                self.tunnel_parameters.tunnel_constants,
                !use_bridges,
            )
            .await?;

        let wintun_exit_interface = tunnel_handle
            .exit_wintun_interface()
            .expect("failed to obtain wintun exit interface");

        tracing::info!("Created wintun device: {}", wintun_exit_interface.name);

        wintun::setup_wintun_adapter(wintun_exit_interface.windows_luid(), exit_adapter_config)?;
        wintun::wait_for_interfaces(
            wintun_exit_interface.windows_luid(),
            true,
            self.enable_ipv6(),
        )
        .await?;
        wintun::initialize_interfaces(
            wintun_exit_interface.windows_luid(),
            Some(exit_mtu),
            self.enable_ipv6().then_some(exit_mtu),
        )?;
        wintun::wait_for_addresses(wintun_exit_interface.windows_luid()).await?;

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: wintun_exit_interface.name.clone(),
            exit_tun_mtu: exit_mtu,
            #[cfg(not(target_os = "linux"))]
            entry_gateway_address: entry_endpoint.ip(),
        };

        if let Err(err) = self.set_routes(routing_config, self.enable_ipv6()).await {
            tunnel_handle.cancel();
            _ = tunnel_handle.wait().await;
            return Err(err);
        }

        // Update interface name in tunnel metadata
        tunnel_metadata.interface = wintun_exit_interface.name.clone();

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_handle: AnyTunnelHandle::from(tunnel_handle),
            tunnel_conn_data,
        })
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    async fn start_wireguard_tunnel(
        &mut self,
        connected_tunnel: ConnectedTunnel,
    ) -> Result<StartTunnelResult> {
        let conn_data = connected_tunnel.connection_data();
        let use_bridges = self.tunnel_parameters.tunnel_settings.bridges_enabled();

        // Prepare network environment for the wireguard connection to the entry gateway
        let entry_mtu = connected_tunnel.entry_mtu();
        let entry_tun = Self::create_wireguard_device(
            conn_data.entry.private_ipv4,
            self.enable_ipv6().then_some(conn_data.entry.private_ipv6),
            None,
            entry_mtu,
        )?;
        let entry_tun_name = entry_tun
            .deref()
            .tun_name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created entry tun device: {}", entry_tun_name);

        let mut ips = vec![IpAddr::V4(conn_data.entry.private_ipv4)];
        if self.enable_ipv6() {
            ips.push(IpAddr::V6(conn_data.entry.private_ipv6));
        }
        let entry_tunnel_metadata = TunnelMetadata {
            interface: entry_tun_name.clone(),
            ips,
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        let exit_mtu = connected_tunnel.exit_mtu();
        let exit_tun = Self::create_wireguard_device(
            conn_data.exit.private_ipv4,
            self.enable_ipv6().then_some(conn_data.exit.private_ipv6),
            // todo: this needs to be able to set both destinations?
            Some(conn_data.entry.private_ipv4.into()),
            exit_mtu,
        )?;
        let exit_tun_name = exit_tun
            .deref()
            .tun_name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        let mut ips = vec![IpAddr::V4(conn_data.exit.private_ipv4)];
        if self.enable_ipv6() {
            ips.push(IpAddr::V6(conn_data.exit.private_ipv6));
        }

        let exit_tunnel_metadata = TunnelMetadata {
            interface: exit_tun_name.clone(),
            ips,
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: self
                .tunnel_parameters
                .tunnel_settings
                .enable_ipv6
                .then_some(conn_data.entry.private_ipv6),
        };

        #[cfg(not(target_os = "linux"))]
        let entry_endpoint = conn_data.effective_remote_entry_endpoint();

        let routing_config = RoutingConfig::Wireguard {
            entry_tun_name: entry_tunnel_metadata.interface.clone(),
            exit_tun_name: exit_tunnel_metadata.interface.clone(),
            entry_tun_mtu: entry_mtu,
            exit_tun_mtu: exit_mtu,
            private_entry_gateway_address: self
                .tunnel_parameters
                .tunnel_constants
                .private_entry_gateway_address,
            #[cfg(not(target_os = "linux"))]
            entry_gateway_address: entry_endpoint.ip(),
            exit_gateway_address: conn_data.exit.endpoint.ip(),
        };
        self.set_routes(routing_config, self.enable_ipv6()).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr.clone(),
            entry: WireguardNode::from(&conn_data.entry),
            exit: WireguardNode::from(&conn_data.exit),
        });

        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun,
            exit_tun,
            dns: self.get_dns_addresses(),
        });

        let tunnel_handle = connected_tunnel
            .run(
                tunnel_options,
                self.tunnel_parameters.tunnel_constants,
                !use_bridges,
            )
            .await?;
        let tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::Two {
                entry: entry_tunnel_metadata,
                exit: exit_tunnel_metadata,
            },
            tunnel_conn_data,
            tunnel_handle,
        })
    }

    #[cfg(windows)]
    async fn start_wireguard_tunnel(
        &mut self,
        connected_tunnel: ConnectedTunnel,
    ) -> Result<StartTunnelResult> {
        let conn_data = connected_tunnel.connection_data();
        let use_bridges = self.tunnel_parameters.tunnel_settings.bridges_enabled();

        // Prepare network environment for the wireguard connection to the entry gateway
        let entry_tun_mtu = connected_tunnel.entry_mtu();
        let exit_tun_mtu = connected_tunnel.exit_mtu();

        let exit_gateway_address = conn_data.exit.endpoint.ip();
        let entry_gateway_address = conn_data.effective_remote_entry_endpoint().ip();

        let entry_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.entry.private_ipv4,
            interface_ipv6: self.enable_ipv6().then_some(conn_data.entry.private_ipv6),
            gateway_ipv4: None,
            gateway_ipv6: None,
        };

        let mut ips = vec![IpAddr::V4(conn_data.entry.private_ipv4)];
        if self.enable_ipv6() {
            ips.push(IpAddr::V6(conn_data.entry.private_ipv6))
        }
        let mut entry_tunnel_metadata = TunnelMetadata {
            interface: "".to_owned(),
            ips,
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        let exit_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.exit.private_ipv4,
            interface_ipv6: self.enable_ipv6().then_some(conn_data.exit.private_ipv6),
            gateway_ipv4: Some(conn_data.entry.private_ipv4),
            gateway_ipv6: self.enable_ipv6().then_some(conn_data.entry.private_ipv6),
        };
        let mut ips = vec![IpAddr::V4(conn_data.exit.private_ipv4)];
        if self.enable_ipv6() {
            ips.push(IpAddr::V6(conn_data.exit.private_ipv6));
        }
        let mut exit_tunnel_metadata = TunnelMetadata {
            interface: "".to_owned(),
            ips,
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: self.enable_ipv6().then_some(conn_data.entry.private_ipv6),
        };

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr.clone(),
            entry: WireguardNode::from(&conn_data.entry),
            exit: WireguardNode::from(&conn_data.exit),
        });

        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun_name: WG_ENTRY_WINTUN_NAME.to_owned(),
            entry_tun_guid: WG_ENTRY_WINTUN_GUID.to_owned(),
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: self.get_dns_addresses(),
        });

        let mut tunnel_handle = connected_tunnel
            .run(
                self.route_handler.clone(),
                tunnel_options,
                self.tunnel_parameters.tunnel_constants,
                !use_bridges,
            )
            .await?;

        let wintun_entry_interface = tunnel_handle
            .entry_wintun_interface()
            .expect("failed to obtain wintun entry interface");
        let wintun_exit_interface = tunnel_handle
            .exit_wintun_interface()
            .expect("failed to obtain wintun exit interface");

        tracing::info!(
            "Created entry wintun device: {}",
            wintun_entry_interface.name
        );
        tracing::info!("Created exit wintun device: {}", wintun_exit_interface.name);

        wintun::setup_wintun_adapter(wintun_entry_interface.windows_luid(), entry_adapter_config)?;
        wintun::setup_wintun_adapter(wintun_exit_interface.windows_luid(), exit_adapter_config)?;

        wintun::wait_for_interfaces(
            wintun_entry_interface.windows_luid(),
            true,
            self.enable_ipv6(),
        )
        .await?;
        wintun::wait_for_interfaces(
            wintun_exit_interface.windows_luid(),
            true,
            self.enable_ipv6(),
        )
        .await?;
        wintun::initialize_interfaces(
            wintun_entry_interface.windows_luid(),
            Some(entry_tun_mtu),
            self.enable_ipv6().then_some(entry_tun_mtu),
        )?;
        wintun::initialize_interfaces(
            wintun_exit_interface.windows_luid(),
            Some(exit_tun_mtu),
            self.enable_ipv6().then_some(exit_tun_mtu),
        )?;
        wintun::wait_for_addresses(wintun_entry_interface.windows_luid()).await?;
        wintun::wait_for_addresses(wintun_exit_interface.windows_luid()).await?;

        // Update interface names in tunnel metadata
        entry_tunnel_metadata.interface = wintun_entry_interface.name.clone();
        exit_tunnel_metadata.interface = wintun_exit_interface.name.clone();

        let tunnel_interface = TunnelInterface::Two {
            entry: entry_tunnel_metadata,
            exit: exit_tunnel_metadata,
        };

        let routing_config = RoutingConfig::Wireguard {
            entry_tun_name: wintun_entry_interface.name.clone(),
            exit_tun_name: wintun_exit_interface.name.clone(),
            entry_tun_mtu,
            exit_tun_mtu,
            private_entry_gateway_address: self
                .tunnel_parameters
                .tunnel_constants
                .private_entry_gateway_address,
            entry_gateway_address,
            exit_gateway_address,
        };

        if let Err(err) = self.set_routes(routing_config, self.enable_ipv6()).await {
            tunnel_handle.cancel();
            tunnel_handle.wait().await.ok();
            return Err(err);
        }

        Ok(StartTunnelResult {
            tunnel_interface,
            tunnel_handle: AnyTunnelHandle::from(tunnel_handle),
            tunnel_conn_data,
        })
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn start_wireguard_netstack_tunnel(
        &self,
        connected_tunnel: ConnectedTunnel,
        entry_metadata_tx: tokio::sync::oneshot::Sender<SocketAddr>,
    ) -> Result<StartTunnelResult> {
        let mtu = connected_tunnel.exit_mtu();
        let conn_data = connected_tunnel.connection_data();
        let use_bridges = self.tunnel_parameters.tunnel_settings.bridges_enabled();

        let mut interface_addresses = vec![IpNetwork::V4(Ipv4Network::from(
            conn_data.exit.private_ipv4,
        ))];
        if self.enable_ipv6() {
            interface_addresses.push(IpNetwork::V6(Ipv6Network::from(
                conn_data.exit.private_ipv6,
            )));
        }

        let entry_endpoint = conn_data.effective_remote_entry_endpoint().ip();
        let dns_servers = self.get_dns_addresses();

        let packet_tunnel_settings = crate::tunnel_provider::TunnelSettings {
            dns_servers: dns_servers.clone(),
            interface_addresses,
            remote_addresses: vec![entry_endpoint],
            mtu,
        };

        let tun_device = self.create_tun_device(packet_tunnel_settings).await?;
        let tun_fd = unsafe { BorrowedFd::borrow_raw(tun_device.deref().as_raw_fd()) };
        let interface = tun_name::get_tun_name(&tun_fd).map_err(Error::GetTunDeviceName)?;
        let mut ips = vec![IpAddr::V4(conn_data.exit.private_ipv4)];
        if self.enable_ipv6() {
            ips.push(IpAddr::V6(conn_data.exit.private_ipv6));
        }
        let tunnel_metadata = TunnelMetadata {
            interface,
            ips,
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        tracing::info!("Created tun device: {}", tunnel_metadata.interface);

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr.clone(),
            entry: WireguardNode::from(&conn_data.entry),
            exit: WireguardNode::from(&conn_data.exit),
        });

        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            metadata_proxy_tx: entry_metadata_tx,
            exit_tun: tun_device,
            dns: dns_servers,
            #[cfg(target_os = "android")]
            dns_filter: self.dns_filter.clone(),
        });

        let tunnel_handle = connected_tunnel
            .run(
                #[cfg(target_os = "android")]
                self.tun_provider.clone(),
                tunnel_options,
                self.tunnel_parameters.tunnel_constants,
                !use_bridges,
            )
            .await?;

        Ok(StartTunnelResult {
            tunnel_conn_data,
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_handle: AnyTunnelHandle::from(tunnel_handle),
        })
    }

    #[cfg(not(target_os = "android"))]
    fn get_dns_addresses(&self) -> Vec<IpAddr> {
        vec![self.tunnel_parameters.filtering_resolver_addr.ip()]
    }

    #[cfg(target_os = "android")]
    fn get_dns_addresses(&self) -> Vec<IpAddr> {
        self.tunnel_parameters.tunnel_settings.android_tunnel_dns()
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn set_routes(&mut self, routing_config: RoutingConfig, enable_ipv6: bool) -> Result<()> {
        self.route_handler
            .add_routes(routing_config, enable_ipv6)
            .await
            .map_err(Error::AddRoutes)?;

        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn create_mixnet_device(
        interface_ipv4: Ipv4Addr,
        interface_ipv6: Option<Ipv6Addr>,
        mtu: u16,
    ) -> Result<AsyncDevice> {
        let tun_device = {
            let mut tun_config = tun::Configuration::default();

            // rust-tun uses the same name for tunnel type.
            #[cfg(windows)]
            tun_config.tun_name(MIXNET_WINTUN_NAME);

            tun_config
                .address(interface_ipv4)
                .netmask(Ipv4Addr::BROADCAST)
                .mtu(mtu)
                .up();

            #[cfg(target_os = "macos")]
            tun_config.platform_config(|platform_config| {
                platform_config.enable_routing(false);
            });

            tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?
        };

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Some(interface_ipv6) = interface_ipv6 {
            let tun_name = tun_device
                .deref()
                .tun_name()
                .map_err(Error::GetTunDeviceName)?;
            tun_ipv6::set_ipv6_addr(&tun_name, interface_ipv6)
                .map_err(Error::SetTunDeviceIpv6Addr)?;
        }

        #[cfg(windows)]
        {
            let interface_luid = NET_LUID_LH {
                Value: tun_device.tun_luid(),
            };

            if let Some(interface_ipv6) = interface_ipv6 {
                wintun::add_ipv6_address(interface_luid, interface_ipv6)?;
            }

            wintun::wait_for_interfaces(interface_luid, true, interface_ipv6.is_some()).await?;
            wintun::initialize_interfaces(
                interface_luid,
                Some(mtu),
                interface_ipv6.is_some().then_some(mtu),
            )?;
            wintun::wait_for_addresses(interface_luid).await?;
        }

        Ok(tun_device)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn create_wireguard_device(
        interface_ipv4: Ipv4Addr,
        interface_ipv6: Option<Ipv6Addr>,
        destination: Option<IpAddr>,
        mtu: u16,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        tun_config
            .address(interface_ipv4)
            .netmask(Ipv4Addr::BROADCAST)
            .mtu(mtu)
            .up();

        if let Some(destination) = destination {
            tun_config.destination(destination);
        }

        #[cfg(target_os = "macos")]
        tun_config.platform_config(|platform_config| {
            platform_config.enable_routing(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        let tun_name = tun_device
            .deref()
            .tun_name()
            .map_err(Error::GetTunDeviceName)?;

        if let Some(interface_ipv6) = interface_ipv6 {
            tun_ipv6::set_ipv6_addr(&tun_name, interface_ipv6)
                .map_err(Error::SetTunDeviceIpv6Addr)?;
        }

        Ok(tun_device)
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn create_tun_device(
        &self,
        packet_tunnel_settings: crate::tunnel_provider::TunnelSettings,
    ) -> Result<AsyncDevice> {
        #[cfg(target_os = "ios")]
        let owned_tun_fd =
            crate::tunnel_provider::ios::get_tun_fd().map_err(Error::LocateTunDevice)?;

        #[cfg(target_os = "android")]
        let owned_tun_fd = {
            let raw_tun_fd = self
                .tun_provider
                .configure_tunnel(packet_tunnel_settings)
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?;
            unsafe { OwnedFd::from_raw_fd(raw_tun_fd) }
        };

        let mut tun_config = tun::Configuration::default();
        tun_config.raw_fd(owned_tun_fd.as_raw_fd());

        #[cfg(target_os = "ios")]
        tun_config.close_fd_on_drop(false);

        #[cfg(target_os = "ios")]
        {
            self.tun_provider
                .set_tunnel_network_settings(packet_tunnel_settings)
                .await
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?
        }

        let device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        // Consume the owned fd, since the device is now responsible for closing the underlying raw fd.
        let _ = owned_tun_fd.into_raw_fd();

        Ok(device)
    }

    fn enable_ipv6(&self) -> bool {
        self.tunnel_parameters.tunnel_settings.enable_ipv6
    }

    fn create_icmp_probe(&self, exit_tunnel_metadata: &TunnelMetadata) -> Result<IcmpProbe> {
        let mut icmp_probe_config = IcmpProbeConfig::default_v4();

        // Prefer bind to interface on supported platforms
        #[cfg(any(target_os = "linux", target_os = "ios", target_os = "macos"))]
        {
            icmp_probe_config =
                icmp_probe_config.with_interface(exit_tunnel_metadata.interface.clone());
        }

        // Bind to local interface IP on other platforms
        #[cfg(not(any(target_os = "linux", target_os = "ios", target_os = "macos")))]
        {
            let local_addr = exit_tunnel_metadata
                .ips
                .iter()
                .find(|v| v.is_ipv4())
                .ok_or(Error::ProbeRequiresIPv4Addr)?;
            icmp_probe_config = icmp_probe_config.with_local_address(*local_addr);
        }

        IcmpProbe::new(icmp_probe_config).map_err(Error::CreateIcmpProbe)
    }

    fn create_tcp_probe(&self, exit_tunnel_metadata: &TunnelMetadata) -> Result<TcpProbe> {
        let mut tcp_probe_config = TcpProbeConfig::default_v4();

        // Prefer bind to interface on supported platforms
        #[cfg(any(target_os = "linux", target_os = "ios", target_os = "macos"))]
        {
            tcp_probe_config =
                tcp_probe_config.with_interface(exit_tunnel_metadata.interface.clone());
        }

        // Bind to local interface IP on other platforms
        #[cfg(not(any(target_os = "linux", target_os = "ios", target_os = "macos")))]
        {
            let local_addr = exit_tunnel_metadata
                .ips
                .iter()
                .find(|v| v.is_ipv4())
                .ok_or(Error::ProbeRequiresIPv4Addr)?;
            tcp_probe_config = tcp_probe_config.with_local_address(SocketAddr::new(*local_addr, 0));
        }

        TcpProbe::new(tcp_probe_config).map_err(Error::CreateTcpProbe)
    }

    fn create_tunnel_connection_monitor(
        &self,
        exit_tunnel_metadata: &TunnelMetadata,
        event_tx: mpsc::UnboundedSender<ConnectionEvent>,
    ) -> Result<JoinHandle<Result<(), nym_connection_monitor::Error>>> {
        // Create ICMP probe first, fallback to TCP probe on failure.
        match self.create_icmp_probe(exit_tunnel_metadata) {
            Ok(icmp_probe) => {
                tracing::info!(
                    probe_type = "icmp",
                    probe_selection = "default_primary",
                    "Selected ICMP connectivity probe"
                );

                let timing_config = match self.tunnel_parameters.tunnel_settings.tunnel_type_used()
                {
                    TunnelType::Mixnet => TimingConfig::mixnet(),
                    TunnelType::Wireguard => TimingConfig::two_hop_icmp(),
                };

                Ok(ConnectionMonitor::spawn(
                    icmp_probe,
                    timing_config,
                    event_tx,
                    self.shutdown_token.child_token(),
                ))
            }
            Err(err) => {
                tracing::warn!(
                    probe_type = "tcp",
                    probe_selection = "icmp_setup_failed",
                    fallback_reason = %err.display_chain(),
                    "ICMP probe setup failed, falling back to TCP"
                );

                let timing_config = match self.tunnel_parameters.tunnel_settings.tunnel_type_used()
                {
                    TunnelType::Mixnet => TimingConfig::mixnet(),
                    TunnelType::Wireguard => TimingConfig::two_hop_tcp(),
                };

                let tcp_probe = self.create_tcp_probe(exit_tunnel_metadata)?;
                tracing::info!(
                    probe_type = "tcp",
                    probe_selection = "icmp_setup_failed_fallback",
                    "Selected TCP connectivity probe after ICMP setup failure"
                );
                Ok(ConnectionMonitor::spawn(
                    tcp_probe,
                    timing_config,
                    event_tx,
                    self.shutdown_token.child_token(),
                ))
            }
        }
    }
}

struct StartTunnelResult {
    tunnel_interface: TunnelInterface,
    tunnel_conn_data: TunnelConnectionData,
    tunnel_handle: AnyTunnelHandle,
}

struct WgTunnelRuntime {
    bandwidth_monitor_handle: JoinHandle<()>,
    transport_fwd_handle: Option<JoinHandle<()>>,
    authenticator_listener_handle: Option<AuthClientMixnetListenerHandle>,
    metadata_path_health: MetadataPathHealth,
}

impl WgTunnelRuntime {
    // Returns the mixnet cancellation token, to monitor mixnet client unexpected stop.
    // Returns None if we already stopped it (in new Wireguard mode) and we don't need to monitor it.
    fn mixnet_client_token(&self) -> Option<CancellationToken> {
        self.authenticator_listener_handle
            .as_ref()
            .map(|handle| handle.mixnet_cancel_token())
    }
}
