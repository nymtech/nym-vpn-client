// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{FutureExt, future::Fuse, pin_mut};

use nym_registration_client::{
    MixnetRegistrationResult, RegistrationClientBuilder, RegistrationClientBuilderConfig,
    RegistrationNymNode, RegistrationResult, WireguardRegistrationResult,
};
use nym_registration_common::NymNode;
use nym_sdk::UserAgent;
use nym_vpn_account_controller::AccountStateReceiver;
use nym_vpn_network_config::{Network, start_background_file_refresh};
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
    time::Duration,
};
#[cfg(unix)]
use std::{os::fd::RawFd, sync::Arc};

#[cfg(target_os = "linux")]
use nix::sys::socket::{SetSockOpt, sockopt::Mark};

#[cfg(windows)]
use super::wintun::{self, WintunAdapterConfig};
#[cfg(any(target_os = "ios", target_os = "android"))]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_gateway_directory::{
    GatewayCacheHandle, GatewayClient, GatewayMinPerformance, ResolvedConfig,
};
use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tun::Device;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use super::route_handler::{RouteHandler, RoutingConfig};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use super::tun_ipv6;
#[cfg(any(target_os = "ios", target_os = "android"))]
use super::tun_name;
use super::{
    Error, NymConfig, Result, TunnelInterface, TunnelMetadata, TunnelSettings,
    tunnel::{self, AnyTunnelHandle, SelectedGateways, Tombstone},
};
use nym_common::trace_err_chain;
use nym_vpn_lib_types::{
    ConnectionData, ErrorStateReason, EstablishConnectionData, GatewayId, MixnetConnectionData,
    NymAddress, TunnelConnectionData, TunnelType, WireguardConnectionData, WireguardNode,
};
use nym_vpn_store::keys::wireguard::WireguardKeysDb;

#[cfg(not(any(target_os = "ios", target_os = "android")))]
use super::tunnel::wireguard::connected_tunnel::TunTunTunnelOptions;
use super::tunnel::wireguard::connected_tunnel::{NetstackTunnelOptions, TunnelOptions};
#[cfg(target_os = "android")]
use crate::tunnel_provider::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::OSTunProvider;
#[cfg(not(target_os = "linux"))]
use crate::tunnel_state_machine::tunnel::transports::TransportError;
use crate::{
    VpnTopologyProvider,
    bandwidth_controller::BandwidthController,
    tunnel_state_machine::{
        TunnelConstants, WireguardMultihopMode, account, ipv6_availability,
        tunnel::{
            mixnet, transports,
            wireguard::{
                self, ConnectionData as WgConnectionData, MetadataEvent, MetadataReceiver,
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

    /// Connecting mixnet client
    ConnectingMixnetClient,

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

    /// A new network environment was discovered
    NewNetworkEnv {
        /// The new network environment
        network: Box<Network>,
    },
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
    pub resolved_gateway_config: ResolvedConfig,
    pub tunnel_settings: TunnelSettings,
    pub tunnel_constants: TunnelConstants,
    pub selected_gateways: Option<SelectedGateways>,
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
    account_controller_state: AccountStateReceiver,
    gateway_cache_handle: GatewayCacheHandle,
    custom_topology_provider: VpnTopologyProvider,
    wg_keys_db: WireguardKeysDb,
    shutdown_token: CancellationToken,
}

impl TunnelMonitor {
    pub fn start(
        tunnel_parameters: TunnelParameters,
        account_controller_state: AccountStateReceiver,
        gateway_cache_handle: GatewayCacheHandle,
        custom_topology_provider: VpnTopologyProvider,
        monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
        wg_keys_db: WireguardKeysDb,
        #[cfg(not(any(target_os = "android", target_os = "ios")))] route_handler: RouteHandler,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
    ) -> TunnelMonitorHandle {
        let shutdown_token = CancellationToken::new();
        let tunnel_monitor = Self {
            tunnel_parameters,
            monitor_event_sender,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            route_handler,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            account_controller_state,
            gateway_cache_handle,
            custom_topology_provider,
            wg_keys_db,
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

        self.account_controller_state
            .wait_for_account_ready_to_connect()
            .await
            .map_err(|e| Error::Account(account::Error::ControllerState(e)))?;

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

        let user_agent = self
            .tunnel_parameters
            .tunnel_settings
            .user_agent
            .clone()
            .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));

        // TODO: user_agent must not be a part of tunnel_settings
        let gateway_directory_client = GatewayClient::new_with_resolver_overrides(
            gateway_config.clone(),
            user_agent.clone(),
            Some(
                &self
                    .tunnel_parameters
                    .resolved_gateway_config
                    .nym_vpn_api_resolver_overrides,
            ),
        )
        .map_err(Error::GatewayDirectoryClient)?;

        self.gateway_cache_handle
            .replace_gateway_client(gateway_directory_client)
            .ok();
        self.gateway_cache_handle.refresh_all().await.ok();

        let selected_gateways =
            if let Some(selected_gateways) = self.tunnel_parameters.selected_gateways.clone() {
                selected_gateways
            } else {
                self.send_event(TunnelMonitorEvent::SelectingGateways);

                let new_gateways = tunnel::select_gateways(
                    self.gateway_cache_handle.clone(),
                    &self.tunnel_parameters.tunnel_settings,
                    self.wg_keys_db.clone(),
                    self.shutdown_token.child_token(),
                )
                .await?;

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

        self.send_event(TunnelMonitorEvent::ConnectingMixnetClient);

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
        let custom_topology_provider = self.custom_topology_provider.clone();
        custom_topology_provider
            .update_config(
                mixnet_client_config.min_mixnode_performance,
                mixnet_client_config.min_gateway_performance,
            )
            .await;

        let entry_ip = selected_gateways
            .entry_gateway()
            .lookup_ip()
            .ok_or(tunnel::Error::NoIpAddressAnnounced {
                gateway_id: selected_gateways
                    .entry_gateway()
                    .identity()
                    .to_base58_string(),
            })
            .map_err(Box::new)?;

        let exit_ip = selected_gateways
            .exit_gateway()
            .lookup_ip()
            .ok_or(tunnel::Error::NoIpAddressAnnounced {
                gateway_id: selected_gateways
                    .exit_gateway()
                    .identity()
                    .to_base58_string(),
            })
            .map_err(Box::new)?;

        let entry_node = RegistrationNymNode {
            node: NymNode {
                identity: selected_gateways.entry_gateway().identity,
                ipr_address: selected_gateways
                    .entry_gateway()
                    .ipr_address
                    .map(Into::into),
                authenticator_address: selected_gateways
                    .entry_gateway()
                    .authenticator_address
                    .map(Into::into),
                ip_address: entry_ip,
                version: selected_gateways.entry_gateway().version.clone().into(),
            },
            keys: selected_gateways.entry_keypair().clone(),
        };

        let exit_node = RegistrationNymNode {
            node: NymNode {
                identity: selected_gateways.exit_gateway().identity,
                ipr_address: selected_gateways.exit_gateway().ipr_address.map(Into::into),
                authenticator_address: selected_gateways
                    .exit_gateway()
                    .authenticator_address
                    .map(Into::into),
                ip_address: exit_ip,
                version: selected_gateways.exit_gateway().version.clone().into(),
            },
            keys: selected_gateways.exit_keypair().clone(),
        };

        let rc_builder_config = RegistrationClientBuilderConfig {
            entry_node,
            exit_node,
            data_path: self.tunnel_parameters.nym_config.data_path.clone(),
            mixnet_client_config,
            two_hops: self.tunnel_parameters.tunnel_settings.tunnel_type == TunnelType::Wireguard,
            user_agent,
            custom_topology_provider: Box::new(self.custom_topology_provider.clone()),
            network_env: self
                .tunnel_parameters
                .nym_config
                .network_env
                .nym_network
                .network
                .clone(),
            cancel_token: self.shutdown_token.child_token(),
            #[cfg(unix)]
            connection_fd_callback: Arc::new(connection_fd_callback),
        };

        let rc_builder = RegistrationClientBuilder::new(rc_builder_config);

        let registration_client = Box::pin(rc_builder.build()).await?;
        let registration_result = Box::pin(registration_client.register()).await?;

        let (entry_metadata_tx, entry_metadata_rx) =
            tokio::sync::oneshot::channel::<MetadataEvent>();
        let (exit_metadata_tx, exit_metadata_rx) = tokio::sync::oneshot::channel::<MetadataEvent>();

        let (entry_metadata_addr_tx, entry_metadata_addr_rx) = tokio::sync::oneshot::channel();

        let StartTunnelResult {
            tunnel_interface,
            tunnel_conn_data,
            mut tunnel_handle,
        } = match registration_result {
            RegistrationResult::Mixnet(inner_result) => {
                self.start_mixnet_tunnel(*inner_result).await?
            }
            RegistrationResult::Wireguard(inner_result) => {
                // As a first step, spin the bw controller here
                let connected_tunnel = self
                    .setup_wireguard_tunnel(
                        *inner_result,
                        entry_metadata_rx,
                        exit_metadata_rx,
                        selected_gateways.clone(),
                    )
                    .await?;

                match self
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
                }
            }
        };

        let establishing_connection_data = EstablishConnectionData {
            entry_gateway: GatewayId::from(selected_gateways.entry_gateway().clone()),
            exit_gateway: GatewayId::from(selected_gateways.exit_gateway().clone()),
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

        // todo: do initial ping

        let (discovery_refresher_tx, mut discovery_refresher_rx) = tokio::sync::mpsc::channel(1);
        let discovery_refresher_handle = if let Some(config_path) =
            self.tunnel_parameters.nym_config.config_path.as_ref()
            && let Some(config_dir) = config_path.parent()
        {
            Some(start_background_file_refresh(
                config_dir.to_path_buf(),
                self.tunnel_parameters.nym_config.network_env.clone(),
                discovery_refresher_tx.clone(),
                self.shutdown_token.child_token(),
            ))
        } else {
            None
        };

        let connection_data = ConnectionData {
            entry_gateway: GatewayId::from(selected_gateways.entry_gateway().clone()),
            exit_gateway: GatewayId::from(selected_gateways.exit_gateway().clone()),
            connected_at: OffsetDateTime::now_utc(),
            tunnel: tunnel_conn_data,
        };
        self.send_event(TunnelMonitorEvent::Up {
            tunnel_interface: tunnel_interface.clone(),
            connection_data: Box::new(connection_data),
        });

        // Send metadata endpoint data to the bandwidth controller
        match tunnel_interface {
            TunnelInterface::One(exit) => {
                let _metadata_event_handler = tokio::spawn(async move {
                    if let Ok(entry_metadata_endpoint) = entry_metadata_addr_rx.await {
                        tracing::info!(
                            "Received entry metadata endpoint: {entry_metadata_endpoint}"
                        );
                        entry_metadata_tx
                            .send(MetadataEvent::MetadataProxy(entry_metadata_endpoint))
                            .ok();
                    }
                });
                exit_metadata_tx
                    .send(MetadataEvent::TunnelMetadata(exit.clone()))
                    .ok();
            }
            TunnelInterface::Two { entry, exit } => {
                entry_metadata_tx
                    .send(MetadataEvent::TunnelMetadata(entry.clone()))
                    .ok();
                exit_metadata_tx
                    .send(MetadataEvent::TunnelMetadata(exit.clone()))
                    .ok();
            }
        }

        let mixnet_monitoring_token = tunnel_handle
            .mixnet_client_token()
            .map(|token| token.cancelled_owned().fuse())
            .unwrap_or(Fuse::terminated());
        pin_mut!(mixnet_monitoring_token);

        loop {
            tokio::select! {
                _  = &mut mixnet_monitoring_token => {
                    tracing::error!("MixnetClient exited unexpectedly");
                    break;
                }
                _ = self.shutdown_token.cancelled() => {
                    break;
                }
                ret = discovery_refresher_rx.recv() => {
                    match ret {
                        Some(Ok(network)) => {
                            tracing::info!("Refreshed discovery file");
                            self.send_event(TunnelMonitorEvent::NewNetworkEnv { network: Box::new(network) });
                        }
                        Some(Err(err)) => {
                            trace_err_chain!(err, "Failed to refresh discovery file");
                        }
                        None => {
                            // channel closed
                            break;
                        }
                    }
                }
            }
        }

        // Trigger cancellation since many other tasks depend on shutdown token
        self.shutdown_token.cancel();

        tracing::info!("Waiting for tunnel to exit");
        tunnel_handle.cancel();

        let tun_devices = tunnel_handle
            .wait()
            .await
            .inspect_err(|e| {
                trace_err_chain!(e, "Failed to gracefully shutdown the tunnel");
            })
            .unwrap_or_default();

        if let Some(discovery_refresher_handle) = discovery_refresher_handle {
            tracing::debug!("Wait for discovery refresher to exit");
            if let Err(e) = discovery_refresher_handle.await {
                tracing::error!("Failed to join on discovery refresher: {}", e);
            }
        }
        tracing::info!("Tunnel monitor finished");

        Ok(tun_devices)
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
        let connected_tunnel = mixnet::connected_tunnel::ConnectedTunnel::new(
            registration_result.mixnet_client,
            assigned_addresses,
            self.shutdown_token.clone(),
        );

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
        )?;

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
            let packet_tunnel_settings = crate::tunnel_provider::TunnelSettings {
                dns_servers: self
                    .tunnel_parameters
                    .tunnel_settings
                    .dns
                    .ip_addresses(&self.tunnel_parameters.tunnel_settings.default_dns_ips())
                    .to_vec(),
                interface_addresses,
                remote_addresses: vec![assigned_addresses.entry_mixnet_gateway_ip],
                mtu,
            };

            self.create_tun_device(packet_tunnel_settings).await?
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let tun_name = {
            let tun_fd = unsafe { BorrowedFd::borrow_raw(tun_device.get_ref().as_raw_fd()) };
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

        let tunnel_handle = AnyTunnelHandle::from(
            connected_tunnel
                .run(tun_device)
                .await
                .map_err(|e| Error::Tunnel(Box::new(e)))?,
        );

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_conn_data,
            tunnel_handle,
        })
    }

    async fn setup_wireguard_tunnel(
        &self,
        registration_result: WireguardRegistrationResult,
        entry_metadata_rx: MetadataReceiver,
        exit_metadata_rx: MetadataReceiver,
        selected_gateways: SelectedGateways,
    ) -> Result<wireguard::connected_tunnel::ConnectedTunnel> {
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

        let WireguardRegistrationResult {
            entry_gateway_client,
            exit_gateway_client,
            entry_gateway_data,
            exit_gateway_data,
            authenticator_listener_handle,
            bw_controller,
        } = registration_result;

        let bw = BandwidthController::create(
            bw_controller,
            &selected_gateways,
            entry_gateway_client,
            exit_gateway_client,
            entry_gateway_data.clone(),
            exit_gateway_data.clone(),
            entry_signal_rx,
            exit_signal_rx,
            self.tunnel_parameters
                .nym_config
                .network_env
                .gw_update_version(),
            self.shutdown_token.clone(),
        )
        .await
        .map_err(|e| Box::new(tunnel::Error::from(e)))?;

        let authenticator_listener_handle = if bw.is_using_latest_client() {
            // We don't need the mixnet client anymore
            tracing::info!(
                "Disconnecting mixnet client as we are using the latest bandwidth controller"
            );
            authenticator_listener_handle.stop().await;
            None
        } else {
            Some(authenticator_listener_handle)
        };
        let bandwidth_controller_handle = tokio::spawn(bw.run());

        let mut connection_data = WgConnectionData {
            entry_bridge_addr: None,
            entry: entry_gateway_data,
            exit: exit_gateway_data,
        };

        let mut transport_fwd_handle = None;
        if self.tunnel_parameters.tunnel_settings.bridges_enabled() {
            let entry_bridge_params = selected_gateways
                .entry_gateway()
                .get_bridge_params()
                .ok_or(transports::TransportError::config_err(
                    "attempted to open transport connection without bridge params",
                ))?;

            // Attempt transport Connection. If successful a listening UDP connection is created
            // and the bind address of that UDP listener is provided to the entry wireguard tunnel
            // as the endpoint address.
            tracing::info!("Establishing DVPN QUIC transport tunnel");

            let bridge_conn = transports::BridgeConn::try_connect(
                entry_bridge_params,
                self.shutdown_token.clone(),
            )
            .await
            .inspect_err(|_| self.shutdown_token.cancel())?;
            connection_data.entry_bridge_addr = Some(bridge_conn.endpoint);
            let (local_fwd_listen_addr, fwd_handle) =
                transports::UdpForwarder::launch(bridge_conn, None, self.shutdown_token.clone())
                    .await
                    .inspect_err(|_| self.shutdown_token.cancel())?;
            transport_fwd_handle = Some(fwd_handle);
            tracing::info!(
                "quic transport connected, udp forwarder open on {local_fwd_listen_addr:?}"
            );
            connection_data.entry.endpoint = local_fwd_listen_addr;
        };

        Ok(wireguard::connected_tunnel::ConnectedTunnel::new(
            selected_gateways.entry_keypair().clone(),
            selected_gateways.exit_keypair().clone(),
            connection_data,
            bandwidth_controller_handle,
            transport_fwd_handle,
            authenticator_listener_handle,
        ))
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_tunnel: wireguard::connected_tunnel::ConnectedTunnel,
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
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        #[cfg(not(target_os = "linux"))]
        let entry_endpoint = if use_bridges {
            conn_data.entry_bridge_addr.ok_or(TransportError::other(
                "missing bridge address after connect", // this should not be possible
            ))?
        } else {
            conn_data.entry.endpoint
        };

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: exit_tun_name.clone(),
            exit_tun_mtu,
            #[cfg(not(target_os = "linux"))]
            entry_gateway_address: entry_endpoint.ip(),
        };

        self.set_routes(routing_config, self.enable_ipv6()).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr,
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self.tunnel_parameters.tunnel_settings.resolved_dns_config();
        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            metadata_proxy_tx: entry_metadata_tx,
            exit_tun,
            dns: dns_config.tunnel_config().to_vec(),
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
        connected_tunnel: wireguard::connected_tunnel::ConnectedTunnel,
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
            entry_bridge_addr: conn_data.entry_bridge_addr,
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        #[cfg(not(target_os = "linux"))]
        let entry_endpoint = if use_bridges {
            conn_data.entry_bridge_addr.ok_or(TransportError::other(
                "missing bridge address after connect", // this should not be possible
            ))?
        } else {
            conn_data.entry.endpoint
        };

        let dns_config = self.tunnel_parameters.tunnel_settings.resolved_dns_config();
        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            metadata_proxy_tx: entry_metadata_tx,
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: dns_config.tunnel_config().to_vec(),
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
        wintun::initialize_interfaces(
            wintun_exit_interface.windows_luid(),
            Some(exit_mtu),
            self.enable_ipv6().then_some(exit_mtu),
        )?;

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
        connected_tunnel: wireguard::connected_tunnel::ConnectedTunnel,
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
            .get_ref()
            .name()
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
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
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
        let entry_endpoint = if use_bridges {
            conn_data.entry_bridge_addr.ok_or(TransportError::other(
                "missing bridge address after connect", // this should not be possible
            ))?
        } else {
            conn_data.entry.endpoint
        };

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
            entry_bridge_addr: conn_data.entry_bridge_addr,
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self.tunnel_parameters.tunnel_settings.resolved_dns_config();
        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun,
            exit_tun,
            dns: dns_config.tunnel_config().to_vec(),
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
        connected_tunnel: wireguard::connected_tunnel::ConnectedTunnel,
    ) -> Result<StartTunnelResult> {
        let conn_data = connected_tunnel.connection_data();
        let use_bridges = self.tunnel_parameters.tunnel_settings.bridges_enabled();

        // Prepare network environment for the wireguard connection to the entry gateway
        let entry_tun_mtu = connected_tunnel.entry_mtu();
        let exit_tun_mtu = connected_tunnel.exit_mtu();

        let exit_gateway_address = conn_data.exit.endpoint.ip();
        let entry_gateway_address = if use_bridges {
            conn_data
                .entry_bridge_addr
                .ok_or(TransportError::other(
                    "missing bridge address after connect", // this should not be possible
                ))?
                .ip()
        } else {
            conn_data.entry.endpoint.ip()
        };

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
            ips.push(IpAddr::V6(conn_data.entry.private_ipv6));
        }
        let mut exit_tunnel_metadata = TunnelMetadata {
            interface: "".to_owned(),
            ips,
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: self.enable_ipv6().then_some(conn_data.entry.private_ipv6),
        };

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: conn_data.entry_bridge_addr,
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self.tunnel_parameters.tunnel_settings.resolved_dns_config();
        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun_name: WG_ENTRY_WINTUN_NAME.to_owned(),
            entry_tun_guid: WG_ENTRY_WINTUN_GUID.to_owned(),
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: dns_config.tunnel_config().to_vec(),
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
        connected_tunnel: wireguard::connected_tunnel::ConnectedTunnel,
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

        let entry_endpoint = if use_bridges {
            conn_data
                .entry_bridge_addr
                .ok_or(TransportError::other(
                    "missing bridge address after connect", // this should not be possible
                ))?
                .ip()
        } else {
            conn_data.entry.endpoint.ip()
        };

        let packet_tunnel_settings = crate::tunnel_provider::TunnelSettings {
            dns_servers: self
                .tunnel_parameters
                .tunnel_settings
                .dns
                .ip_addresses(&self.tunnel_parameters.tunnel_settings.default_dns_ips())
                .to_vec(),
            interface_addresses,
            remote_addresses: vec![entry_endpoint],
            mtu,
        };

        let tun_device = self.create_tun_device(packet_tunnel_settings).await?;
        let tun_fd = unsafe { BorrowedFd::borrow_raw(tun_device.get_ref().as_raw_fd()) };
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
            entry_bridge_addr: conn_data.entry_bridge_addr,
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_servers = self
            .tunnel_parameters
            .tunnel_settings
            .dns
            .ip_addresses(&self.tunnel_parameters.tunnel_settings.default_dns_ips())
            .to_vec();

        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            metadata_proxy_tx: entry_metadata_tx,
            exit_tun: tun_device,
            dns: dns_servers,
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

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn set_routes(&mut self, routing_config: RoutingConfig, enable_ipv6: bool) -> Result<()> {
        self.route_handler
            .add_routes(routing_config, enable_ipv6)
            .await
            .map_err(Error::AddRoutes)?;

        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn create_mixnet_device(
        interface_ipv4: Ipv4Addr,
        interface_ipv6: Option<Ipv6Addr>,
        mtu: u16,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        // rust-tun uses the same name for tunnel type.
        #[cfg(windows)]
        tun_config.name(MIXNET_WINTUN_NAME);

        tun_config.address(interface_ipv4).mtu(i32::from(mtu)).up();

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Some(interface_ipv6) = interface_ipv6 {
            tun_ipv6::set_ipv6_addr(&tun_name, interface_ipv6)
                .map_err(Error::SetTunDeviceIpv6Addr)?;
        }

        #[cfg(windows)]
        {
            let interface_luid = wintun::get_interface_luid_for_alias(&tun_name)?;

            if let Some(interface_ipv6) = interface_ipv6 {
                wintun::add_ipv6_address(interface_luid, interface_ipv6)?;
            }

            wintun::initialize_interfaces(
                interface_luid,
                Some(mtu),
                interface_ipv6.is_some().then_some(mtu),
            )?;
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
            .mtu(i32::from(mtu))
            .up();

        if let Some(destination) = destination {
            tun_config.destination(destination);
        }

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        let tun_name = tun_device
            .get_ref()
            .name()
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
}

pub struct StartTunnelResult {
    tunnel_interface: TunnelInterface,
    tunnel_conn_data: TunnelConnectionData,
    tunnel_handle: AnyTunnelHandle,
}
