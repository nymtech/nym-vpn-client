// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
use nix::sys::socket::{sockopt::Mark, SetSockOpt};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::net::Ipv4Addr;
#[cfg(any(target_os = "linux", target_os = "ios", target_os = "android"))]
use std::os::fd::BorrowedFd;
#[cfg(any(target_os = "android", target_os = "ios"))]
use std::os::fd::{AsRawFd, IntoRawFd};
#[cfg(target_os = "android")]
use std::os::fd::{FromRawFd, OwnedFd};
use std::{cmp, net::IpAddr, time::Duration};
#[cfg(unix)]
use std::{os::fd::RawFd, sync::Arc};

#[cfg(windows)]
use super::wintun::{self, WintunAdapterConfig};
#[cfg(any(target_os = "ios", target_os = "android"))]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_gateway_directory::{GatewayMinPerformance, ResolvedConfig};
use nym_vpn_account_controller::{AccountCommand, AccountControllerCommander};
use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use tun::Device;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_ip_packet_requests::IpPair;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use super::route_handler::RouteHandler;
#[cfg(any(target_os = "ios", target_os = "android"))]
use super::tun_name;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use super::{route_handler::RoutingConfig, tun_ipv6};
use super::{
    tunnel::{
        self, AnyTunnelHandle, ConnectedMixnet, MixnetConnectOptions, SelectedGateways, Tombstone,
    },
    Error, NymConfig, Result, TunnelInterface, TunnelMetadata, TunnelSettings,
};
use nym_vpn_lib_types::{
    ConnectionData, ErrorStateReason, Gateway, MixnetConnectionData, MixnetEvent, NymAddress,
    RequestZkNymError, TunnelConnectionData, TunnelType, WireguardConnectionData, WireguardNode,
};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use super::tunnel::wireguard::connected_tunnel::{
    NetstackTunnelOptions, TunTunTunnelOptions, TunnelOptions,
};
#[cfg(any(target_os = "ios", target_os = "android"))]
use crate::tunnel_provider;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
#[cfg(target_os = "linux")]
use crate::tunnel_state_machine::route_handler::TUNNEL_FWMARK;
use crate::tunnel_state_machine::{account, WireguardMultihopMode};

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

/// Initial delay between retry attempts.
const INITIAL_WAIT_DELAY: Duration = Duration::from_secs(2);

/// Wait delay multiplier used for each subsequent retry attempt.
const DELAY_MULTIPLIER: u32 = 2;

/// Max wait delay between retry attempts.
const MAX_WAIT_DELAY: Duration = Duration::from_secs(15);

/// Timeout when waiting for reply from the event handler.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum TunnelMonitorEvent {
    /// Initializing mixnet client
    InitializingClient,

    /// Syncronizing account with vpn-api
    SyncingAccount,

    /// Registering device with vpn-api
    RegisteringDevice,

    /// Requesting and downloading zknym credentials from vpn-api
    RequestingZkNyms,

    /// Selecting gateways
    SelectingGateways,

    /// Selected gateways
    SelectedGateways {
        gateways: Box<SelectedGateways>,
        /// Back channel to acknowledge that the event has been processed
        reply_tx: tokio::sync::oneshot::Sender<()>,
    },

    /// Tunnel interface is up.
    InterfaceUp {
        /// Tunnel interface
        tunnel_interface: TunnelInterface,
        /// Connection data
        connection_data: Box<ConnectionData>,
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
}

pub struct TunnelMonitorHandle {
    cancel_token: CancellationToken,
    join_handle: JoinHandle<Tombstone>,
}

impl TunnelMonitorHandle {
    pub fn cancel(&self) {
        tracing::info!("Cancelling tunnel monitor handle");
        self.cancel_token.cancel();
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
    pub selected_gateways: Option<SelectedGateways>,
    pub retry_attempt: u32,
}

pub struct TunnelMonitor {
    tunnel_parameters: TunnelParameters,
    monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
    mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    route_handler: RouteHandler,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
}

impl TunnelMonitor {
    pub fn start(
        tunnel_parameters: TunnelParameters,
        account_controller_tx: AccountControllerCommander,
        monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
        mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        route_handler: RouteHandler,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
    ) -> TunnelMonitorHandle {
        let cancel_token = CancellationToken::new();
        let tunnel_monitor = Self {
            tunnel_parameters,
            monitor_event_sender,
            mixnet_event_sender,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            route_handler,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            account_controller_tx,
            cancel_token: cancel_token.clone(),
        };
        let join_handle = tokio::spawn(tunnel_monitor.run());

        TunnelMonitorHandle {
            cancel_token,
            join_handle,
        }
    }

    async fn run(mut self) -> Tombstone {
        let (tombstone, reason) = match self.run_inner().await {
            Ok(tombstone) => (tombstone, None),
            Err(e) => {
                tracing::error!("Tunnel monitor exited with error: {}", e);
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
        if self.tunnel_parameters.retry_attempt > 0 {
            let delay = wait_delay(self.tunnel_parameters.retry_attempt);
            tracing::debug!("Waiting for {}s before connecting.", delay.as_secs());

            self.cancel_token
                .run_until_cancelled(tokio::time::sleep(delay))
                .await
                .ok_or(Error::Tunnel(tunnel::Error::Cancelled))?;
        }

        self.send_event(TunnelMonitorEvent::InitializingClient);

        self.setup_account().await?;

        self.send_event(TunnelMonitorEvent::SelectingGateways);

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
                    "Invalid gateway performance values. Will carry on with initial values. Error: {}"
                , e);
            }
        }

        let selected_gateways =
            if let Some(selected_gateways) = self.tunnel_parameters.selected_gateways.clone() {
                selected_gateways
            } else {
                let new_gateways = tunnel::select_gateways(
                    gateway_config.clone(),
                    self.tunnel_parameters.resolved_gateway_config.clone(),
                    self.tunnel_parameters.tunnel_settings.tunnel_type,
                    self.tunnel_parameters.tunnel_settings.entry_point.clone(),
                    self.tunnel_parameters.tunnel_settings.exit_point.clone(),
                    self.tunnel_parameters.tunnel_settings.user_agent.clone(),
                    self.cancel_token.child_token(),
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

        let connect_options = MixnetConnectOptions {
            data_path: self.tunnel_parameters.nym_config.data_path.clone(),
            gateway_config,
            resolved_gateway_config: self.tunnel_parameters.resolved_gateway_config.clone(),
            mixnet_client_config: self
                .tunnel_parameters
                .tunnel_settings
                .mixnet_client_config
                .clone(),
            tunnel_type: self.tunnel_parameters.tunnel_settings.tunnel_type,
            enable_credentials_mode: self
                .tunnel_parameters
                .tunnel_settings
                .enable_credentials_mode,
            stats_recipient_address: self
                .tunnel_parameters
                .tunnel_settings
                .statistics_recipient
                .as_deref()
                .copied(),
            selected_gateways: selected_gateways.clone(),
            user_agent: None, // todo: provide user-agent
        };

        #[cfg(target_os = "android")]
        let tun_provider = self.tun_provider.clone();
        #[cfg(unix)]
        let connection_fd_callback = move |_fd: RawFd| {
            tracing::debug!("Callback on connection fd");
            #[cfg(target_os = "android")]
            tun_provider.bypass(_fd);
            #[cfg(target_os = "linux")]
            if let Err(err) = Mark.set(unsafe { &BorrowedFd::borrow_raw(_fd) }, &TUNNEL_FWMARK) {
                tracing::error!("Could not fwmark mixnet fd: {err}");
            }
        };
        let mut connected_mixnet = tunnel::connect_mixnet(
            connect_options,
            &self.tunnel_parameters.nym_config.network_env,
            self.cancel_token.child_token(),
            #[cfg(unix)]
            Arc::new(connection_fd_callback),
        )
        .await?;

        // Route mixnet client outside the tunnel.
        #[cfg(target_os = "android")]
        match connected_mixnet.websocket_fd().await {
            Some(fd) => {
                self.tun_provider.bypass(fd);
            }
            None => {
                tracing::error!("Failed to obtain websocket for bypass");
            }
        }

        let status_listener_handle = connected_mixnet
            .start_event_listener(
                self.mixnet_event_sender.clone(),
                self.cancel_token.child_token(),
            )
            .await;

        let selected_gateways = connected_mixnet.selected_gateways().clone();
        let StartTunnelResult {
            tunnel_interface,
            tunnel_conn_data,
            mut tunnel_handle,
        } = match self.tunnel_parameters.tunnel_settings.tunnel_type {
            TunnelType::Mixnet => self.start_mixnet_tunnel(connected_mixnet).await?,
            TunnelType::Wireguard => {
                match self
                    .tunnel_parameters
                    .tunnel_settings
                    .wireguard_tunnel_options
                    .multihop_mode
                {
                    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                    WireguardMultihopMode::TunTun => {
                        self.start_wireguard_tunnel(connected_mixnet).await?
                    }
                    WireguardMultihopMode::Netstack => {
                        self.start_wireguard_netstack_tunnel(connected_mixnet)
                            .await?
                    }
                }
            }
        };

        let connection_data = ConnectionData {
            entry_gateway: Gateway::from(*selected_gateways.entry),
            exit_gateway: Gateway::from(*selected_gateways.exit),
            connected_at: None,
            tunnel: tunnel_conn_data,
        };

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.send_event(TunnelMonitorEvent::InterfaceUp {
            tunnel_interface: tunnel_interface.clone(),
            connection_data: Box::new(connection_data.clone()),
            reply_tx,
        });

        if tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await.is_err() {
            tracing::warn!("Interface up reply timeout");
        }

        // todo: do initial ping

        let connection_data = ConnectionData {
            connected_at: Some(OffsetDateTime::now_utc()),
            ..connection_data
        };
        self.send_event(TunnelMonitorEvent::Up {
            tunnel_interface,
            connection_data: Box::new(connection_data),
        });

        let task_error = self
            .cancel_token
            .run_until_cancelled(tunnel_handle.recv_error())
            .await;

        if let Some(Some(task_error)) = task_error {
            tracing::error!("Task manager quit with error: {}", task_error);
        }

        tracing::debug!("Wait for tunnel to exit");
        tunnel_handle.cancel().await;

        let tun_devices = tunnel_handle
            .wait()
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to gracefully shutdown the tunnel: {}", e);
            })
            .unwrap_or_default();

        tracing::debug!("Wait for status listener to exit");
        if let Err(e) = status_listener_handle.await {
            tracing::error!("Failed to join on status listener: {}", e);
        }

        Ok(tun_devices)
    }

    fn send_event(&mut self, event: TunnelMonitorEvent) {
        if let Err(e) = self.monitor_event_sender.send(event) {
            if !self.cancel_token.is_cancelled() {
                tracing::error!("Failed to send monitor event: {}", e);
            }
        }
    }

    async fn setup_account(&mut self) -> Result<()> {
        // Check if we have ticketbooks already stored, then we can sidestep the account and device
        // sync
        let is_already_tickets_stored = self
            .account_controller_tx
            .get_available_tickets()
            .await
            .map_err(|err| {
                account::Error::from(RequestZkNymError::CredentialStorage(err.to_string()))
            })?
            .is_all_ticket_types_above_soft_threshold();

        if is_already_tickets_stored {
            // If we have tickets stored, trigger sync and register in the background while we
            // proceed anyway.
            self.send_event(TunnelMonitorEvent::SyncingAccount);
            self.account_controller_tx
                .send(AccountCommand::SyncAccountState(None))
                .ok();

            self.account_controller_tx
                .send(AccountCommand::SyncDeviceState(None))
                .ok();

            self.send_event(TunnelMonitorEvent::RegisteringDevice);
            self.account_controller_tx
                .send(AccountCommand::RegisterDevice(None))
                .ok();
        } else {
            // If we don't have ticket stored, go through the steps one by one, syncing and
            // registering and getting credentials.
            self.send_event(TunnelMonitorEvent::SyncingAccount);
            account::wait_for_account_sync(
                self.account_controller_tx.clone(),
                self.cancel_token.clone(),
            )
            .await?;

            account::wait_for_device_sync(
                self.account_controller_tx.clone(),
                self.cancel_token.clone(),
            )
            .await?;

            self.send_event(TunnelMonitorEvent::RegisteringDevice);
            account::wait_for_device_register(
                self.account_controller_tx.clone(),
                self.cancel_token.clone(),
            )
            .await?;
        }

        if self
            .tunnel_parameters
            .tunnel_settings
            .enable_credentials_mode
        {
            self.send_event(TunnelMonitorEvent::RequestingZkNyms);
            account::wait_for_credentials_ready(
                self.account_controller_tx.clone(),
                self.cancel_token.clone(),
            )
            .await?;
        }

        Ok(())
    }

    async fn start_mixnet_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<StartTunnelResult> {
        let connected_tunnel = connected_mixnet
            .connect_mixnet_tunnel(self.cancel_token.clone())
            .await?;
        let assigned_addresses = connected_tunnel.assigned_addresses();

        let mtu: u16 = self
            .tunnel_parameters
            .tunnel_settings
            .mixnet_tunnel_options
            .mtu
            .unwrap_or(DEFAULT_TUN_MTU);

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let tun_device = Self::create_mixnet_device(assigned_addresses.interface_addresses, mtu)?;

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let tun_device = {
            let packet_tunnel_settings = tunnel_provider::tunnel_settings::TunnelSettings {
                dns_servers: self
                    .tunnel_parameters
                    .tunnel_settings
                    .dns
                    .ip_addresses(&crate::DEFAULT_DNS_SERVERS)
                    .to_vec(),
                interface_addresses: vec![
                    IpNetwork::V4(Ipv4Network::from(
                        assigned_addresses.interface_addresses.ipv4,
                    )),
                    IpNetwork::V6(Ipv6Network::from(
                        assigned_addresses.interface_addresses.ipv6,
                    )),
                ],
                remote_addresses: vec![assigned_addresses.entry_mixnet_gateway_ip],
                mtu,
            };

            self.create_tun_device(packet_tunnel_settings).await?
        };

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
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

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let routing_config = RoutingConfig::Mixnet {
                tun_name: tun_name.clone(),
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address: assigned_addresses.entry_mixnet_gateway_ip,
            };

            self.set_routes(routing_config).await?;
        }

        let tunnel_conn_data = TunnelConnectionData::Mixnet(MixnetConnectionData {
            nym_address: NymAddress::from(assigned_addresses.mixnet_client_address),
            exit_ipr: NymAddress::from(assigned_addresses.exit_mix_addresses),
            entry_ip: assigned_addresses.entry_mixnet_gateway_ip,
            exit_ip: assigned_addresses.exit_mixnet_gateway_ip,
            ipv4: assigned_addresses.interface_addresses.ipv4,
            ipv6: assigned_addresses.interface_addresses.ipv6,
        });

        let tunnel_metadata = TunnelMetadata {
            interface: tun_name,
            ips: vec![
                IpAddr::V4(assigned_addresses.interface_addresses.ipv4),
                IpAddr::V6(assigned_addresses.interface_addresses.ipv6),
            ],
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        let tunnel_handle = AnyTunnelHandle::from(connected_tunnel.run(tun_device).await);

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_conn_data,
            tunnel_handle,
        })
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<StartTunnelResult> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.tunnel_parameters.nym_config.network_env,
                self.tunnel_parameters
                    .tunnel_settings
                    .enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();

        let exit_tun = Self::create_wireguard_device(
            IpPair {
                ipv4: conn_data.exit.private_ipv4,
                ipv6: conn_data.exit.private_ipv6,
            },
            Some(conn_data.entry.private_ipv4),
            connected_tunnel.exit_mtu(),
        )?;
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: exit_tun_name.clone(),
            #[cfg(not(target_os = "linux"))]
            entry_gateway_address: conn_data.entry.endpoint.ip(),
        };

        self.set_routes(routing_config).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self
            .tunnel_parameters
            .tunnel_settings
            .dns
            .to_dns_config()
            .resolve(
                &crate::DEFAULT_DNS_SERVERS,
                #[cfg(target_os = "macos")]
                53,
            );
        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
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

        let tunnel_handle = AnyTunnelHandle::from(connected_tunnel.run(tunnel_options).await?);

        Ok(StartTunnelResult {
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_conn_data,
            tunnel_handle,
        })
    }

    #[cfg(windows)]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<StartTunnelResult> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.tunnel_parameters.nym_config.network_env,
                self.tunnel_parameters
                    .tunnel_settings
                    .enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();
        let entry_gateway_address = conn_data.entry.endpoint.ip();

        let exit_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.exit.private_ipv4,
            interface_ipv6: conn_data.exit.private_ipv6,
            gateway_ipv4: Some(conn_data.entry.private_ipv4),
            gateway_ipv6: Some(conn_data.entry.private_ipv6),
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
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self
            .tunnel_parameters
            .tunnel_settings
            .dns
            .to_dns_config()
            .resolve(&crate::DEFAULT_DNS_SERVERS);
        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: dns_config.tunnel_config().to_vec(),
        });

        let tunnel_handle = connected_tunnel
            .run(
                #[cfg(windows)]
                self.route_handler.clone(),
                tunnel_options,
            )
            .await?;

        let wintun_exit_interface = tunnel_handle
            .exit_wintun_interface()
            .expect("failed to obtain wintun exit interface");

        tracing::info!("Created wintun device: {}", wintun_exit_interface.name);

        wintun::setup_wintun_adapter(wintun_exit_interface.windows_luid(), exit_adapter_config)?;

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: wintun_exit_interface.name.clone(),
            entry_gateway_address,
        };
        // todo: make sure to shutdown tunnel_handle on failure!
        self.set_routes(routing_config).await?;

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
        connected_mixnet: ConnectedMixnet,
    ) -> Result<StartTunnelResult> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.tunnel_parameters.nym_config.network_env,
                self.tunnel_parameters
                    .tunnel_settings
                    .enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();

        let entry_tun = Self::create_wireguard_device(
            IpPair {
                ipv4: conn_data.entry.private_ipv4,
                ipv6: conn_data.entry.private_ipv6,
            },
            None,
            connected_tunnel.entry_mtu(),
        )?;
        let entry_tun_name = entry_tun
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created entry tun device: {}", entry_tun_name);

        let entry_tunnel_metadata = TunnelMetadata {
            interface: entry_tun_name,
            ips: vec![
                IpAddr::V4(conn_data.entry.private_ipv4),
                IpAddr::V6(conn_data.entry.private_ipv6),
            ],
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        let exit_tun = Self::create_wireguard_device(
            IpPair {
                ipv4: conn_data.exit.private_ipv4,
                ipv6: conn_data.exit.private_ipv6,
            },
            // todo: this needs to be able to set both destinations?
            Some(conn_data.entry.private_ipv4),
            connected_tunnel.exit_mtu(),
        )?;
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        let exit_tunnel_metadata = TunnelMetadata {
            interface: exit_tun_name.clone(),
            ips: vec![
                IpAddr::V4(conn_data.exit.private_ipv4),
                IpAddr::V6(conn_data.exit.private_ipv6),
            ],
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: Some(conn_data.entry.private_ipv6),
        };

        let routing_config = RoutingConfig::Wireguard {
            entry_tun_name: entry_tunnel_metadata.interface.clone(),
            exit_tun_name: exit_tunnel_metadata.interface.clone(),
            #[cfg(not(target_os = "linux"))]
            entry_gateway_address: conn_data.entry.endpoint.ip(),
            exit_gateway_address: conn_data.exit.endpoint.ip(),
        };
        self.set_routes(routing_config).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self
            .tunnel_parameters
            .tunnel_settings
            .dns
            .to_dns_config()
            .resolve(
                &crate::DEFAULT_DNS_SERVERS,
                #[cfg(target_os = "macos")]
                53,
            );
        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun,
            exit_tun,
            dns: dns_config.tunnel_config().to_vec(),
        });

        let tunnel_handle = AnyTunnelHandle::from(connected_tunnel.run(tunnel_options).await?);

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
        connected_mixnet: ConnectedMixnet,
    ) -> Result<StartTunnelResult> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.tunnel_parameters.nym_config.network_env,
                self.tunnel_parameters
                    .tunnel_settings
                    .enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();

        let entry_gateway_address = conn_data.entry.endpoint.ip();
        let exit_gateway_address = conn_data.exit.endpoint.ip();

        let entry_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.entry.private_ipv4,
            interface_ipv6: conn_data.entry.private_ipv6,
            gateway_ipv4: None,
            gateway_ipv6: None,
        };
        let mut entry_tunnel_metadata = TunnelMetadata {
            interface: "".to_owned(),
            ips: vec![
                IpAddr::V4(conn_data.entry.private_ipv4),
                IpAddr::V6(conn_data.entry.private_ipv6),
            ],
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        let exit_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.exit.private_ipv4,
            interface_ipv6: conn_data.exit.private_ipv6,
            gateway_ipv4: Some(conn_data.entry.private_ipv4),
            gateway_ipv6: Some(conn_data.entry.private_ipv6),
        };
        let mut exit_tunnel_metadata = TunnelMetadata {
            interface: "".to_owned(),
            ips: vec![
                IpAddr::V4(conn_data.exit.private_ipv4),
                IpAddr::V6(conn_data.exit.private_ipv6),
            ],
            ipv4_gateway: Some(conn_data.entry.private_ipv4),
            ipv6_gateway: Some(conn_data.entry.private_ipv6),
        };

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_config = self
            .tunnel_parameters
            .tunnel_settings
            .dns
            .to_dns_config()
            .resolve(&crate::DEFAULT_DNS_SERVERS);
        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun_name: WG_ENTRY_WINTUN_NAME.to_owned(),
            entry_tun_guid: WG_ENTRY_WINTUN_GUID.to_owned(),
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: dns_config.tunnel_config().to_vec(),
        });

        let tunnel_handle = connected_tunnel
            .run(
                #[cfg(windows)]
                self.route_handler.clone(),
                tunnel_options,
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
            entry_gateway_address,
            exit_gateway_address,
        };
        // todo: make sure to shutdown tunnel_handle on failure!
        self.set_routes(routing_config).await?;

        Ok(StartTunnelResult {
            tunnel_interface,
            tunnel_handle: AnyTunnelHandle::from(tunnel_handle),
            tunnel_conn_data,
        })
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn start_wireguard_netstack_tunnel(
        &self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<StartTunnelResult> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.tunnel_parameters.nym_config.network_env,
                self.tunnel_parameters
                    .tunnel_settings
                    .enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;

        let conn_data = connected_tunnel.connection_data();

        let packet_tunnel_settings = tunnel_provider::tunnel_settings::TunnelSettings {
            dns_servers: self
                .tunnel_parameters
                .tunnel_settings
                .dns
                .ip_addresses(&crate::DEFAULT_DNS_SERVERS)
                .to_vec(),
            interface_addresses: vec![
                IpNetwork::V4(Ipv4Network::from(conn_data.exit.private_ipv4)),
                IpNetwork::V6(Ipv6Network::from(conn_data.exit.private_ipv6)),
            ],
            remote_addresses: vec![conn_data.entry.endpoint.ip()],
            mtu: connected_tunnel.exit_mtu(),
        };

        let tun_device = self.create_tun_device(packet_tunnel_settings).await?;
        let tun_fd = unsafe { BorrowedFd::borrow_raw(tun_device.get_ref().as_raw_fd()) };
        let interface = tun_name::get_tun_name(&tun_fd).map_err(Error::GetTunDeviceName)?;
        let tunnel_metadata = TunnelMetadata {
            interface,
            ips: vec![
                IpAddr::V4(conn_data.exit.private_ipv4),
                IpAddr::V6(conn_data.exit.private_ipv6),
            ],
            ipv4_gateway: None,
            ipv6_gateway: None,
        };

        tracing::info!("Created tun device: {}", tunnel_metadata.interface);

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let dns_servers = self
            .tunnel_parameters
            .tunnel_settings
            .dns
            .ip_addresses(&crate::DEFAULT_DNS_SERVERS)
            .to_vec();

        let tunnel_handle = connected_tunnel
            .run(
                tun_device,
                dns_servers,
                #[cfg(target_os = "android")]
                self.tun_provider.clone(),
            )
            .await?;

        Ok(StartTunnelResult {
            tunnel_conn_data,
            tunnel_interface: TunnelInterface::One(tunnel_metadata),
            tunnel_handle: AnyTunnelHandle::from(tunnel_handle),
        })
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_routes(&mut self, routing_config: RoutingConfig) -> Result<()> {
        self.route_handler
            .add_routes(routing_config)
            .await
            .map_err(Error::AddRoutes)?;

        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn create_mixnet_device(interface_addresses: IpPair, mtu: u16) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        // rust-tun uses the same name for tunnel type.
        #[cfg(windows)]
        tun_config.name(MIXNET_WINTUN_NAME);

        tun_config
            .address(interface_addresses.ipv4)
            .mtu(i32::from(mtu))
            .up();

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        tun_ipv6::set_ipv6_addr(&tun_name, interface_addresses.ipv6)
            .map_err(Error::SetTunDeviceIpv6Addr)?;

        Ok(tun_device)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn create_wireguard_device(
        interface_addresses: IpPair,
        destination: Option<Ipv4Addr>,
        mtu: u16,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        tun_config
            .address(interface_addresses.ipv4)
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

        tun_ipv6::set_ipv6_addr(&tun_name, interface_addresses.ipv6)
            .map_err(Error::SetTunDeviceIpv6Addr)?;

        Ok(tun_device)
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn create_tun_device(
        &self,
        packet_tunnel_settings: tunnel_provider::tunnel_settings::TunnelSettings,
    ) -> Result<AsyncDevice> {
        #[cfg(target_os = "ios")]
        let owned_tun_fd =
            tunnel_provider::ios::interface::get_tun_fd().map_err(Error::LocateTunDevice)?;

        #[cfg(target_os = "android")]
        let owned_tun_fd = {
            let raw_tun_fd = self
                .tun_provider
                .configure_tunnel(packet_tunnel_settings.into_tunnel_network_settings())
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?;
            unsafe { OwnedFd::from_raw_fd(raw_tun_fd) }
        };

        let mut tun_config = tun::Configuration::default();
        tun_config.raw_fd(owned_tun_fd.as_raw_fd());

        #[cfg(target_os = "ios")]
        {
            self.tun_provider
                .set_tunnel_network_settings(packet_tunnel_settings.into_tunnel_network_settings())
                .await
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?
        }

        let device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        // Consume the owned fd, since the device is now responsible for closing the underlying raw fd.
        let _ = owned_tun_fd.into_raw_fd();

        Ok(device)
    }
}

fn wait_delay(retry_attempt: u32) -> Duration {
    let multiplier = retry_attempt.saturating_mul(DELAY_MULTIPLIER);
    let delay = INITIAL_WAIT_DELAY.saturating_mul(multiplier);
    cmp::min(delay, MAX_WAIT_DELAY)
}

pub struct StartTunnelResult {
    tunnel_interface: TunnelInterface,
    tunnel_conn_data: TunnelConnectionData,
    tunnel_handle: AnyTunnelHandle,
}
