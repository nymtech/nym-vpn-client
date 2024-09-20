mod gateway_selector;
pub mod mixnet;
mod tun_ipv6;
mod wireguard_tunnel;

use std::{io, net::IpAddr, time::Duration};

pub use gateway_selector::SelectedGateways;
use nym_gateway_directory::GatewayClient;
use nym_sdk::{TaskClient, UserAgent};
use nym_task::TaskManager;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{mixnet::SharedMixnetClient, GatewayDirectoryError, GenericNymVpnConfig, MixnetError};

const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct ConnectedMixnet {
    pub task_manager: TaskManager,
    pub gateway_directory_client: GatewayClient,
    pub selected_gateways: SelectedGateways,
    pub mixnet_client: SharedMixnetClient,
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
        gateway_directory_client,
        mixnet_client,
    })
}

pub async fn run_mixnet_tunnel(
    nym_config: GenericNymVpnConfig,
    connected_mixnet: ConnectedMixnet,
) -> Result<mixnet::connected_tunnel::TunnelHandle> {
    let connector = mixnet::connector::Connector::new(
        connected_mixnet.task_manager,
        connected_mixnet.mixnet_client,
        connected_mixnet.gateway_directory_client,
    );
    let connected_tunnel = connector
        .connect(connected_mixnet.selected_gateways, nym_config.nym_ips)
        .await?;

    let interface_addresses = connected_tunnel.interface_addresses();

    let mut tun_config = tun2::Configuration::default();
    tun_config
        .address(interface_addresses.ipv4)
        .mtu(nym_config.nym_mtu.unwrap_or(1500))
        .up();

    let tun_device = tun2::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

    Ok(connected_tunnel.run(tun_device).await)
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

    #[error("failed to create tun device")]
    CreateTunDevice(#[source] tun2::Error),

    #[error("failed to set ipv6 address on tunnel interface")]
    SetTunIpv6Addr(#[source] io::Error),

    #[error("failed to obtain tun name")]
    ObtainTunName(#[source] tun2::Error),

    #[error("authenticator address is not found")]
    AuthenticatorAddressNotFound,

    #[error("not enough bandwidth")]
    NotEnoughBandwidth,

    #[error("wireguard authentication is not possible due to one of the gateways not running the authenticator process: {0}")]
    AuthenticationNotPossible(String),

    #[error("wireguard gateway failure: {0}")]
    WgGatewayClientFailure(#[from] nym_wg_gateway_client::Error),

    #[error("failed to start wireguard: {0}")]
    StartWireguard(#[source] nym_wg_go::Error),

    #[error("failed to lookup gateway ip: {gateway_id}: {source}")]
    FailedToLookupGatewayIp {
        gateway_id: String,
        source: nym_gateway_directory::Error,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
