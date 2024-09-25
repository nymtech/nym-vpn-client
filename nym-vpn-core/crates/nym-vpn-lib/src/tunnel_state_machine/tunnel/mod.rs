pub mod any_tunnel_handle;
mod gateway_selector;
pub mod mixnet;
pub mod wireguard;

use std::{path::PathBuf, time::Duration};

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::GatewayClient;
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_task::TaskManager;

use crate::{mixnet::SharedMixnetClient, GatewayDirectoryError, GenericNymVpnConfig, MixnetError};

const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct ConnectedMixnet {
    pub task_manager: TaskManager,
    pub gateway_directory_client: GatewayClient,
    pub selected_gateways: SelectedGateways,
    pub data_path: Option<PathBuf>,
    pub mixnet_client: SharedMixnetClient,
}

impl ConnectedMixnet {
    /// Creates a tunnel over mixnet.
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

pub async fn connect_mixnet(
    nym_config: GenericNymVpnConfig,
    enable_wireguard: bool,
) -> Result<ConnectedMixnet> {
    let task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);
    let user_agent = nym_config
        .user_agent
        .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));

    // Select gateways
    let gateway_directory_client = GatewayClient::new(nym_config.gateway_config, user_agent)
        .map_err(Error::CreateGatewayClient)?;
    let selected_gateways = gateway_selector::select_gateways(
        &gateway_directory_client,
        enable_wireguard,
        nym_config.entry_point,
        nym_config.exit_point,
    )
    .await
    .map_err(Error::SelectGateways)?;

    let mixnet_client = tokio::time::timeout(
        MIXNET_CLIENT_STARTUP_TIMEOUT,
        crate::mixnet::setup_mixnet_client(
            selected_gateways.entry.identity(),
            &nym_config.data_path,
            task_manager.subscribe_named("mixnet_client_main"),
            nym_config.mixnet_client_config,
        ),
    )
    .await
    .map_err(|_| Error::StartMixnetClientTimeout)?
    .map_err(Error::MixnetClient)?;

    Ok(ConnectedMixnet {
        task_manager,
        selected_gateways,
        data_path: nym_config.data_path,
        gateway_directory_client,
        mixnet_client,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create gateway client")]
    CreateGatewayClient(#[source] nym_gateway_directory::Error),

    #[error("failed to select gateways")]
    SelectGateways(#[source] GatewayDirectoryError),

    #[error("start mixnet timeout")]
    StartMixnetClientTimeout,

    #[error("mixnet tunnel has failed")]
    MixnetClient(#[from] MixnetError),

    #[error("failed to lookup gateway: {}", gateway_id)]
    LookupGatewayIp {
        gateway_id: String,
        #[source]
        source: nym_gateway_directory::Error,
    },

    #[error("failed to connect ot ip packet router")]
    ConnectToIpPacketRouter(#[source] nym_ip_packet_client::Error),

    #[error("wireguard gateway failure: {0}")]
    WgGatewayClientFailure(#[from] nym_wg_gateway_client::Error),

    #[error("wireguard authentication is not possible due to one of the gateways not running the authenticator process: {0}")]
    AuthenticationNotPossible(String),

    #[error("failed to find authenticator address")]
    AuthenticatorAddressNotFound,

    #[error("not enough bandwidth")]
    NotEnoughBandwidth,

    #[error("failed to lookup gateway ip: {gateway_id}: {source}")]
    FailedToLookupGatewayIp {
        gateway_id: String,
        source: nym_gateway_directory::Error,
    },

    #[error("failed to start wireguard")]
    StartWireguard(#[source] nym_wg_go::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
