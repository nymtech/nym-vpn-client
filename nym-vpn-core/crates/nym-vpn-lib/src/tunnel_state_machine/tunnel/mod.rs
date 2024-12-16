// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod any_tunnel_handle;
mod gateway_selector;
pub mod mixnet;
mod status_listener;
mod tombstone;
pub mod wireguard;

use std::{error::Error as StdError, fmt, path::PathBuf, time::Duration};
#[cfg(target_os = "android")]
use std::{os::fd::RawFd, sync::Arc};

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayClient, Recipient};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_task::{TaskManager, TaskStatus};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[cfg(windows)]
use super::route_handler;
use super::{MixnetEvent, TunnelType};
use crate::{
    bandwidth_controller::ReconnectMixnetClientData, mixnet::SharedMixnetClient,
    GatewayDirectoryError, MixnetClientConfig, MixnetError,
};
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
    reconnect_mixnet_client_data: ReconnectMixnetClientData,
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
    ) -> JoinHandle<()> {
        let (status_tx, status_rx) = futures::channel::mpsc::channel(10);

        self.task_manager
            .start_status_listener(status_tx, TaskStatus::Ready)
            .await;

        StatusListener::spawn(status_rx, event_sender)
    }

    /// Creates a tunnel over Mixnet.
    pub async fn connect_mixnet_tunnel(
        self,
        interface_addresses: Option<IpPair>, // known as config.nym_ips
    ) -> Result<mixnet::connected_tunnel::ConnectedTunnel> {
        let connector = mixnet::connector::Connector::new(
            self.task_manager,
            self.mixnet_client,
            self.gateway_directory_client,
        );

        match connector
            .connect(self.selected_gateways, interface_addresses)
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
        enable_credentials_mode: bool,
    ) -> Result<wireguard::connected_tunnel::ConnectedTunnel> {
        let connector = wireguard::connector::Connector::new(
            self.task_manager,
            self.mixnet_client,
            self.gateway_directory_client,
        );

        match connector
            .connect(
                enable_credentials_mode,
                self.selected_gateways,
                self.data_path,
                self.reconnect_mixnet_client_data,
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
        shutdown_task_manager(self.task_manager).await;
    }
}

#[derive(Debug, Clone)]
pub struct MixnetConnectOptions {
    pub data_path: Option<PathBuf>,
    pub gateway_config: nym_gateway_directory::Config,
    pub mixnet_client_config: Option<MixnetClientConfig>,
    pub tunnel_type: TunnelType,
    pub enable_credentials_mode: bool,
    pub stats_recipient_address: Option<Recipient>,
    pub selected_gateways: SelectedGateways,
    pub user_agent: Option<UserAgent>,
}

pub async fn select_gateways(
    gateway_config: nym_gateway_directory::Config,
    tunnel_type: TunnelType,
    entry_point: Box<EntryPoint>,
    exit_point: Box<ExitPoint>,
    user_agent: Option<UserAgent>,
    cancel_token: CancellationToken,
) -> Result<SelectedGateways> {
    let user_agent =
        user_agent.unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));
    let gateway_directory_client =
        GatewayClient::new(gateway_config, user_agent).map_err(Error::CreateGatewayClient)?;

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
    cancel_token: CancellationToken,
    #[cfg(target_os = "android")] bypass_fn: Arc<dyn Fn(RawFd) + Send + Sync>,
) -> Result<ConnectedMixnet> {
    let task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);
    let bw_controller_task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);

    let task_client = match options.tunnel_type {
        TunnelType::Mixnet => task_manager.subscribe_named("mixnet_client_main"),
        TunnelType::Wireguard => bw_controller_task_manager.subscribe_named("mixnet_client_main"),
    };

    let mut mixnet_client_config = options.mixnet_client_config.clone().unwrap_or_default();
    let reconnect_mixnet_client_data = ReconnectMixnetClientData::new(
        options.clone(),
        bw_controller_task_manager,
        mixnet_client_config.clone(),
    );
    let user_agent = options
        .user_agent
        .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));
    let gateway_directory_client = GatewayClient::new(options.gateway_config, user_agent)
        .map_err(Error::CreateGatewayClient)?;

    match options.tunnel_type {
        TunnelType::Mixnet => {}
        TunnelType::Wireguard => {
            // Always disable poisson process for outbound traffic in wireguard.
            mixnet_client_config.disable_poisson_rate = true;
            // Always disable background cover traffic in wireguard, except for android
            mixnet_client_config.disable_background_cover_traffic = !cfg!(target_os = "android");
        }
    };

    let connect_fut = tokio::time::timeout(
        MIXNET_CLIENT_STARTUP_TIMEOUT,
        crate::mixnet::setup_mixnet_client(
            options.selected_gateways.entry.identity(),
            &options.data_path,
            task_client,
            mixnet_client_config,
            options.enable_credentials_mode,
            options.stats_recipient_address,
            options.tunnel_type == TunnelType::Wireguard,
            #[cfg(target_os = "android")]
            bypass_fn,
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
            reconnect_mixnet_client_data,
        }),
        Err(e) => {
            shutdown_task_manager(task_manager).await;
            Err(e)
        }
    }
}

async fn shutdown_task_manager(mut task_manager: TaskManager) {
    log::debug!("Shutting down task manager");
    if task_manager.signal_shutdown().is_err() {
        log::error!("Failed to signal task manager shutdown");
    }

    task_manager.wait_for_shutdown().await;
    log::debug!("Task manager finished");
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

    #[error("failed to connect ot ip packet router: {}", _0)]
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

    #[cfg(target_os = "ios")]
    #[error("failed to set default path observer: {0}")]
    SetDefaultPathObserver(String),

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
