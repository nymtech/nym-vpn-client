// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod any_tunnel_handle;
mod gateway_selector;
pub mod mixnet;
mod status_listener;
mod tombstone;
pub mod transports;
pub mod wireguard;

#[cfg(unix)]
use std::os::fd::RawFd;
use std::{path::PathBuf, sync::Arc, time::Duration};

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayCacheHandle};
use nym_sdk::UserAgent;
#[allow(deprecated)]
// We should not migrate this to use an SDK task management of any sort, VPN should handle this how they want, this is a leaky abstraction
use nym_task::{TaskManager, TaskStatus};
use nym_vpn_api_client::types::ScoreThresholds;
use nym_vpn_network_config::Network;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[cfg(windows)]
use super::route_handler;
use super::{MixnetEvent, TunnelType};
use crate::{
    GatewayDirectoryError, MixnetClientConfig, MixnetError, VpnTopologyProvider,
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::wireguard::connector::MetadataReceiver,
};
pub use any_tunnel_handle::AnyTunnelHandle;
use status_listener::StatusListener;
pub use tombstone::Tombstone;

pub(crate) const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ConnectedMixnet {
    gateway_cache_handle: GatewayCacheHandle,
    selected_gateways: SelectedGateways,
    data_path: Option<PathBuf>,
    mixnet_client: SharedMixnetClient,
}

impl ConnectedMixnet {
    pub fn selected_gateways(&self) -> &SelectedGateways {
        &self.selected_gateways
    }

    #[allow(deprecated)] // We should not migrate this to use an SDK task management of any sort, VPN should handle this how they want, this is a leaky abstraction
    pub async fn start_event_listener(
        &mut self,
        task_manager: &mut TaskManager,
        event_sender: mpsc::UnboundedSender<MixnetEvent>,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        let (status_tx, status_rx) = futures::channel::mpsc::channel(10);

        task_manager
            .start_status_listener(status_tx, TaskStatus::Ready)
            .await;

        StatusListener::spawn(status_rx, event_sender, cancel_token)
    }

    /// Creates a tunnel over Mixnet.
    pub async fn connect_mixnet_tunnel(
        self,
        cancel_token: CancellationToken,
    ) -> Result<mixnet::connected_tunnel::ConnectedTunnel> {
        let connector =
            mixnet::connector::Connector::new(self.mixnet_client, self.gateway_cache_handle);

        connector
            .connect(self.selected_gateways, cancel_token)
            .await
    }

    /// Creates a tunnel over WireGuard.
    #[allow(deprecated)] // We should not migrate this to use an SDK task management of any sort, VPN should handle this how they want, this is a leaky abstraction
    pub async fn connect_wireguard_tunnel(
        self,
        task_manager: &TaskManager,
        network: &Network,
        cancel_token: CancellationToken,
        entry_metadata_rx: MetadataReceiver,
        exit_metadata_rx: MetadataReceiver,
    ) -> Result<wireguard::connected_tunnel::ConnectedTunnel> {
        let connector =
            wireguard::connector::Connector::new(self.mixnet_client, self.gateway_cache_handle);

        connector
            .connect(
                task_manager,
                network,
                self.selected_gateways,
                self.data_path,
                cancel_token,
                entry_metadata_rx,
                exit_metadata_rx,
            )
            .await
    }
}

#[derive(Debug, Clone)]
pub struct MixnetConnectOptions {
    pub data_path: Option<PathBuf>,
    pub gateway_config: nym_gateway_directory::Config,
    pub resolved_gateway_config: nym_gateway_directory::ResolvedConfig,
    pub mixnet_client_config: Option<MixnetClientConfig>,
    pub tunnel_type: TunnelType,
    pub selected_gateways: SelectedGateways,
    pub user_agent: Option<UserAgent>,
    pub custom_topology_provider: VpnTopologyProvider,
}

pub async fn select_gateways(
    gateway_cache_handle: GatewayCacheHandle,
    tunnel_type: TunnelType,
    entry_point: Box<EntryPoint>,
    exit_point: Box<ExitPoint>,
    wg_score_thresholds: Option<ScoreThresholds>,
    mix_score_thresholds: Option<ScoreThresholds>,
    cancel_token: CancellationToken,
) -> Result<SelectedGateways> {
    let select_gateways_fut = gateway_selector::select_gateways(
        gateway_cache_handle,
        tunnel_type,
        entry_point,
        exit_point,
        wg_score_thresholds,
        mix_score_thresholds,
    );
    cancel_token
        .run_until_cancelled(select_gateways_fut)
        .await
        .ok_or(Error::Cancelled)?
        .map_err(|err| Error::SelectGateways(Box::new(err)))
}

#[allow(deprecated)] // We should not migrate this to use an SDK task management of any sort, VPN should handle this how they want, this is a leaky abstraction
pub async fn connect_mixnet(
    options: MixnetConnectOptions,
    network_env: &Network,
    gateway_cache_handle: GatewayCacheHandle,
    cancel_token: CancellationToken,
    #[cfg(unix)] connection_fd_callback: Arc<dyn Fn(RawFd) + Send + Sync>,
) -> Result<ConnectedMixnet> {
    let mut mixnet_client_config = options.mixnet_client_config.clone().unwrap_or_default();

    match options.tunnel_type {
        TunnelType::Mixnet => {}
        TunnelType::Wireguard | TunnelType::WrappedWireguard => {
            // Always disable poisson process for outbound traffic in wireguard.
            mixnet_client_config.disable_poisson_rate = true;
            // Always disable background cover traffic in wireguard.
            mixnet_client_config.disable_background_cover_traffic = true;
        }
    };

    let setup_mixnet_options = crate::mixnet::SetupMixnetClientOptions {
        network_env: network_env.clone(),
        mixnet_entry_gateway: options.selected_gateways.entry.identity(),
        two_hop_mode: options.tunnel_type == TunnelType::Wireguard,
        custom_topology_provider: options.custom_topology_provider.clone(),
        #[cfg(unix)]
        connection_fd_callback,
    };

    let connect_fut = tokio::time::timeout(
        MIXNET_CLIENT_STARTUP_TIMEOUT,
        crate::mixnet::setup_mixnet_client(
            &options.data_path,
            mixnet_client_config,
            setup_mixnet_options,
            cancel_token.clone(),
        ),
    );

    let mixnet_client = cancel_token
        .run_until_cancelled(connect_fut)
        .await
        .ok_or(Error::Cancelled)
        .and_then(|res| {
            res.map_err(|_| Error::StartMixnetClientTimeout)
                .and_then(|x| x.map_err(Error::MixnetClient))
        })?;

    Ok(ConnectedMixnet {
        selected_gateways: options.selected_gateways,
        data_path: options.data_path,
        gateway_cache_handle,
        mixnet_client: Arc::new(Mutex::new(Some(mixnet_client))),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create gateway client")]
    CreateGatewayClient(#[source] Box<nym_gateway_directory::Error>),

    #[error("failed to select gateways")]
    SelectGateways(#[source] Box<GatewayDirectoryError>),

    #[error("start mixnet client timeout")]
    StartMixnetClientTimeout,

    #[error("mixnet tunnel has failed")]
    MixnetClient(#[from] MixnetError),

    #[error("failed to lookup gateway: {}", gateway_id)]
    LookupGatewayIp {
        gateway_id: String,
        #[source]
        source: Box<nym_gateway_directory::Error>,
    },

    #[error("failed to connect to ip packet router")]
    ConnectToIpPacketRouter(#[source] nym_ip_packet_client::Error),

    #[error(
        "wireguard authentication is not possible due to one of the gateways not running the authenticator process: {0}"
    )]
    AuthenticationNotPossible(String),

    #[error("failed to find authenticator address")]
    AuthenticatorAddressNotFound,

    #[error("failed to setup storage paths")]
    SetupStoragePaths(#[source] Box<nym_sdk::Error>),

    #[error("bandwidth controller error")]
    BandwidthController(#[from] crate::bandwidth_controller::Error),

    #[cfg(target_os = "ios")]
    #[error("failed to resolve using dns64")]
    ResolveDns64(#[from] wireguard::dns64::Error),

    #[error("WireGuard error")]
    Wireguard(#[from] nym_wg_go::Error),

    #[error("failed to dup tunnel file descriptor")]
    DupFd(#[source] std::io::Error),

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[cfg(windows)]
    #[error("failed to add default route listener")]
    AddDefaultRouteListener(#[source] route_handler::Error),

    #[error("transport error")]
    Transport(#[from] transports::TransportError),

    #[error("connection cancelled")]
    Cancelled,

    /// Indicates that a mixnet client has been moved out of shared reference (`Arc<Mutex<Option<MixnetClient>>`)
    /// Typically this is done for the purpose of disconnecting and disposing a mixnet client.
    ///
    /// If this error occurs, it's likely that two or more parties have been racing for access to mixnet client.
    /// One of parties then moved the mixnet client out of shared reference.
    #[error("mixnet client is already disposed")]
    MixnetClientDisposed,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Tunnel connector container.
pub enum AnyConnector {
    Mixnet(mixnet::connector::Connector),
    Wireguard(wireguard::connector::Connector),
}
