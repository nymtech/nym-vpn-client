// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod any_tunnel_handle;
mod gateway_selector;
pub mod mixnet;
mod status_listener;
mod tombstone;
pub mod wireguard;

use std::{error::Error as StdError, fmt, path::PathBuf, time::Duration};
#[cfg(unix)]
use std::{os::fd::RawFd, sync::Arc};

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayClient, Recipient};
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::UserAgent;
use nym_task::{TaskManager, TaskStatus};
use nym_vpn_network_config::Network;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[cfg(windows)]
use super::route_handler;
use super::{MixnetEvent, TunnelType};
use crate::{GatewayDirectoryError, MixnetClientConfig, MixnetError};
pub use any_tunnel_handle::AnyTunnelHandle;
use status_listener::StatusListener;
pub use tombstone::Tombstone;

pub(crate) const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct ConnectedMixnet {
    task_manager: TaskManager,
    gateway_directory_client: GatewayClient,
    selected_gateways: SelectedGateways,
    data_path: Option<PathBuf>,
    mixnet_client: SharedMixnetClient,
}

impl ConnectedMixnet {
    /// Returns the websocket fd owned by mixnet client.
    #[cfg(target_os = "android")]
    pub async fn websocket_fd(&self) -> Option<std::os::fd::RawFd> {
        self.mixnet_client.gateway_ws_fd().await
    }

    pub fn selected_gateways(&self) -> &SelectedGateways {
        &self.selected_gateways
    }

    pub async fn start_event_listener(
        &mut self,
        event_sender: mpsc::UnboundedSender<MixnetEvent>,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        let (status_tx, status_rx) = futures::channel::mpsc::channel(10);

        self.task_manager
            .start_status_listener(status_tx, TaskStatus::Ready)
            .await;

        StatusListener::spawn(status_rx, event_sender, cancel_token)
    }

    /// Creates a tunnel over Mixnet.
    pub async fn connect_mixnet_tunnel(
        self,
        cancel_token: CancellationToken,
    ) -> Result<mixnet::connected_tunnel::ConnectedTunnel> {
        let connector = mixnet::connector::Connector::new(
            self.task_manager,
            self.mixnet_client,
            self.gateway_directory_client,
        );

        match connector
            .connect(self.selected_gateways, cancel_token)
            .await
        {
            Ok(connected_tunnel) => Ok(connected_tunnel),
            Err(connector_error) => {
                connector_error.connector.dispose().await;
                Err(connector_error.error)
            }
        }
    }

    /// Creates a tunnel over WireGuard.
    pub async fn connect_wireguard_tunnel(
        self,
        network: &Network,
        enable_credentials_mode: bool,
        cancel_token: CancellationToken,
    ) -> Result<wireguard::connected_tunnel::ConnectedTunnel> {
        let connector = wireguard::connector::Connector::new(
            self.task_manager,
            self.mixnet_client,
            self.gateway_directory_client,
        );

        match connector
            .connect(
                network,
                enable_credentials_mode,
                self.selected_gateways,
                self.data_path,
                cancel_token,
            )
            .await
        {
            Ok(connected_tunnel) => Ok(connected_tunnel),
            Err(connector_error) => {
                connector_error.connector.dispose().await;
                Err(connector_error.error)
            }
        }
    }

    /// Gracefully shutdown the mixnet client and consume the struct.
    pub async fn dispose(self) {
        tracing::debug!("Shutting down connected mixnet");
        shutdown_mixnet_client(self.task_manager, self.mixnet_client).await;
    }
}

#[derive(Debug, Clone)]
pub struct MixnetConnectOptions {
    pub data_path: Option<PathBuf>,
    pub gateway_config: nym_gateway_directory::Config,
    pub resolved_gateway_config: nym_gateway_directory::ResolvedConfig,
    pub mixnet_client_config: Option<MixnetClientConfig>,
    pub tunnel_type: TunnelType,
    pub enable_credentials_mode: bool,
    pub stats_recipient_address: Option<Recipient>,
    pub selected_gateways: SelectedGateways,
    pub user_agent: Option<UserAgent>,
}

pub async fn select_gateways(
    gateway_config: nym_gateway_directory::Config,
    resolved_gateway_config: nym_gateway_directory::ResolvedConfig,
    tunnel_type: TunnelType,
    entry_point: Box<EntryPoint>,
    exit_point: Box<ExitPoint>,
    user_agent: Option<UserAgent>,
    cancel_token: CancellationToken,
) -> Result<SelectedGateways> {
    let user_agent =
        user_agent.unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));
    let gateway_directory_client = GatewayClient::new_with_resolver_overrides(
        gateway_config,
        user_agent,
        resolved_gateway_config.nym_vpn_api_socket_addrs.as_deref(),
    )
    .map_err(Error::CreateGatewayClient)?;

    let select_gateways_fut = gateway_selector::select_gateways(
        &gateway_directory_client,
        tunnel_type,
        entry_point,
        exit_point,
    );
    cancel_token
        .run_until_cancelled(select_gateways_fut)
        .await
        .ok_or(Error::Cancelled)?
        .map_err(Error::SelectGateways)
}

pub async fn connect_mixnet(
    options: MixnetConnectOptions,
    network_env: &Network,
    cancel_token: CancellationToken,
    #[cfg(unix)] connection_fd_callback: Arc<dyn Fn(RawFd) + Send + Sync>,
) -> Result<ConnectedMixnet> {
    let task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);
    let task_client = task_manager.subscribe_named("mixnet_client_main");

    let mut mixnet_client_config = options.mixnet_client_config.clone().unwrap_or_default();
    let user_agent = options
        .user_agent
        .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));
    let gateway_directory_client = GatewayClient::new_with_resolver_overrides(
        options.gateway_config,
        user_agent,
        options
            .resolved_gateway_config
            .nym_vpn_api_socket_addrs
            .as_deref(),
    )
    .map_err(Error::CreateGatewayClient)?;

    match options.tunnel_type {
        TunnelType::Mixnet => {}
        TunnelType::Wireguard => {
            // Always disable poisson process for outbound traffic in wireguard.
            mixnet_client_config.disable_poisson_rate = true;
            // Always disable background cover traffic in wireguard.
            mixnet_client_config.disable_background_cover_traffic = false;
        }
    };

    let connect_fut = tokio::time::timeout(
        MIXNET_CLIENT_STARTUP_TIMEOUT,
        crate::mixnet::setup_mixnet_client(
            network_env,
            options.selected_gateways.entry.identity(),
            &options.data_path,
            task_client,
            mixnet_client_config,
            options.enable_credentials_mode,
            options.stats_recipient_address,
            options.tunnel_type == TunnelType::Wireguard,
            #[cfg(unix)]
            connection_fd_callback,
        ),
    );

    let res = cancel_token
        .run_until_cancelled(connect_fut)
        .await
        .ok_or(Error::Cancelled)
        .and_then(|res| {
            res.map_err(|_| Error::StartMixnetClientTimeout)
                .and_then(|x| x.map_err(Error::MixnetClient))
        });

    match res {
        Ok(mixnet_client) => Ok(ConnectedMixnet {
            task_manager,
            selected_gateways: options.selected_gateways,
            data_path: options.data_path,
            gateway_directory_client,
            mixnet_client,
        }),
        Err(e) => {
            shutdown_task_manager(task_manager).await;
            Err(e)
        }
    }
}

async fn shutdown_task_manager(mut task_manager: TaskManager) {
    if task_manager.signal_shutdown().is_err() {
        tracing::error!("Failed to signal task manager shutdown");
    }

    tracing::debug!("Waiting for task manager to shutdown");
    task_manager.wait_for_graceful_shutdown().await;
}

async fn shutdown_mixnet_client(mut task_manager: TaskManager, mixnet_client: SharedMixnetClient) {
    if let Err(e) = task_manager.signal_shutdown() {
        tracing::error!("Failed to signal task manager shutdown: {}", e);
    }

    tracing::debug!("Disposing mixnet client");
    mixnet_client.dispose().await;

    tracing::debug!("Waiting for task manager to shutdown");
    task_manager.wait_for_graceful_shutdown().await;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create gateway client: {}", _0)]
    CreateGatewayClient(#[source] nym_gateway_directory::Error),

    #[error("failed to select gateways: {}", _0)]
    SelectGateways(#[source] GatewayDirectoryError),

    #[error("start mixnet client timeout")]
    StartMixnetClientTimeout,

    #[error("mixnet tunnel has failed: {}", _0)]
    MixnetClient(#[from] MixnetError),

    #[error("failed to lookup gateway: {}", gateway_id)]
    LookupGatewayIp {
        gateway_id: String,
        #[source]
        source: nym_gateway_directory::Error,
    },

    #[error("failed to connect to ip packet router: {}", _0)]
    ConnectToIpPacketRouter(#[source] nym_ip_packet_client::Error),

    #[error("wireguard authentication is not possible due to one of the gateways not running the authenticator process: {0}")]
    AuthenticationNotPossible(String),

    #[error("failed to find authenticator address")]
    AuthenticatorAddressNotFound,

    #[error("failed to setup storage paths: {0}")]
    SetupStoragePaths(#[source] nym_sdk::Error),

    #[error("bandwidth controller error: {0}")]
    BandwidthController(#[from] crate::bandwidth_controller::Error),

    #[cfg(target_os = "ios")]
    #[error("failed to resolve using dns64")]
    ResolveDns64(#[from] wireguard::dns64::Error),

    #[error("WireGuard error: {0}")]
    Wireguard(#[from] nym_wg_go::Error),

    #[error("failed to dup tunnel file descriptor: {0}")]
    DupFd(#[source] std::io::Error),

    #[cfg(windows)]
    #[error("failed to add default route listener: {0}")]
    AddDefaultRouteListener(#[source] route_handler::Error),

    #[error("connection cancelled")]
    Cancelled,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Tunnel connector container.
pub enum AnyConnector {
    Mixnet(mixnet::connector::Connector),
    Wireguard(wireguard::connector::Connector),
}

impl AnyConnector {
    pub async fn dispose(self) {
        match self {
            Self::Mixnet(connector) => connector.dispose().await,
            Self::Wireguard(connector) => connector.dispose().await,
        }
    }
}

/// Error returned when connector is unable to connect the tunnel.
pub struct ConnectorError {
    /// The error returned during the attempt to connect the tunnel.
    pub error: Error,

    /// The source connector.
    pub connector: AnyConnector,
}

impl ConnectorError {
    fn new(error: Error, connector: AnyConnector) -> Self {
        Self { error, connector }
    }
}

impl StdError for ConnectorError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

impl fmt::Debug for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectorError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
