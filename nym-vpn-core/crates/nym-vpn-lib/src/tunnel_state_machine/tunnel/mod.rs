// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod any_tunnel_handle;
mod gateway_selector;
pub mod mixnet;
mod status_listener;
pub mod wireguard;

use std::{path::PathBuf, time::Duration};

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayClient};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_task::{manager::TaskStatus, TaskManager};
use tokio::{sync::mpsc, task::JoinHandle};

use super::{MixnetEvent, TunnelType};
use crate::{mixnet::SharedMixnetClient, GatewayDirectoryError, MixnetClientConfig, MixnetError};
use status_listener::StatusListener;

const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct ConnectedMixnet {
    task_manager: TaskManager,
    gateway_directory_client: GatewayClient,
    selected_gateways: SelectedGateways,
    data_path: Option<PathBuf>,
    mixnet_client: SharedMixnetClient,
}

impl ConnectedMixnet {
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
        connector
            .connect(self.selected_gateways, interface_addresses)
            .await
    }

    /// Creates a tunnel over WireGuard.
    pub async fn connect_wireguard_tunnel(
        self,
    ) -> Result<wireguard::connected_tunnel::ConnectedTunnel> {
        let connector = wireguard::connector::Connector::new(
            self.task_manager,
            self.mixnet_client,
            self.gateway_directory_client,
        );
        connector
            .connect(self.selected_gateways, self.data_path)
            .await
    }
}

pub struct MixnetConnectOptions {
    pub data_path: Option<PathBuf>,
    pub gateway_config: nym_gateway_directory::Config,
    pub mixnet_client_config: Option<MixnetClientConfig>,
    pub tunnel_type: TunnelType,
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub user_agent: Option<UserAgent>,
}

pub async fn connect_mixnet(options: MixnetConnectOptions) -> Result<ConnectedMixnet> {
    let task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);
    let user_agent = options
        .user_agent
        .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));

    // Select gateways
    let gateway_directory_client = GatewayClient::new(options.gateway_config, user_agent)
        .map_err(Error::CreateGatewayClient)?;
    let selected_gateways = gateway_selector::select_gateways(
        &gateway_directory_client,
        options.tunnel_type,
        options.entry_point,
        options.exit_point,
    )
    .await
    .map_err(Error::SelectGateways)?;

    let mut mixnet_client_config = options.mixnet_client_config.unwrap_or_default();
    match options.tunnel_type {
        TunnelType::Mixnet => {}
        TunnelType::Wireguard => {
            // Always disable background cover traffic in wireguard.
            mixnet_client_config.disable_background_cover_traffic = true;
        }
    };

    let mixnet_client = tokio::time::timeout(
        MIXNET_CLIENT_STARTUP_TIMEOUT,
        crate::mixnet::setup_mixnet_client(
            selected_gateways.entry.identity(),
            &options.data_path,
            task_manager.subscribe_named("mixnet_client_main"),
            mixnet_client_config,
        ),
    )
    .await
    .map_err(|_| Error::StartMixnetClientTimeout)?
    .map_err(Error::MixnetClient)?;

    Ok(ConnectedMixnet {
        task_manager,
        selected_gateways,
        data_path: options.data_path,
        gateway_directory_client,
        mixnet_client,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create gateway client: {}", _0)]
    CreateGatewayClient(#[source] nym_gateway_directory::Error),

    #[error("failed to select gateways: {}", _0)]
    SelectGateways(#[source] GatewayDirectoryError),

    #[error("start mixnet timeout")]
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

    #[error("failed to start wireguard: {}", _0)]
    StartWireguard(#[source] nym_wg_go::Error),

    #[error("failed to setup storage paths: {0}")]
    SetupStoragePaths(#[source] nym_sdk::Error),

    #[error("bandwidth controller error: {0}")]
    BandwidthController(#[from] crate::bandwidth_controller::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
