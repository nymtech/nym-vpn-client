mod gateway_selector;
mod mixnet_tunnel;
mod wireguard_tunnel;

use std::{io, net::IpAddr, time::Duration};

use nym_gateway_directory::GatewayClient;
use nym_sdk::UserAgent;
use nym_task::TaskManager;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{GatewayDirectoryError, GenericNymVpnConfig, MixnetError};

const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;

#[derive(Debug)]
pub enum Event {
    Up {
        entry_mixnet_gateway_ip: IpAddr,
        tun_name: String,
    },
    Down(Option<Error>),
}

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;

pub fn spawn(
    nym_config: GenericNymVpnConfig,
    enable_wireguard: bool,
    event_tx: mpsc::UnboundedSender<Event>,
    shutdown_token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let err = run(
            nym_config,
            enable_wireguard,
            event_tx.clone(),
            shutdown_token,
        )
        .await
        .err();
        _ = event_tx.send(Event::Down(err));
    })
}

pub async fn run(
    nym_config: GenericNymVpnConfig,
    enable_wireguard: bool,
    event_tx: mpsc::UnboundedSender<Event>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    // Craft user agent.
    let user_agent = nym_config
        .user_agent
        .clone()
        .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));

    // Select gateways
    let gateway_directory_client =
        GatewayClient::new(nym_config.gateway_config.clone(), user_agent)
            .map_err(Error::CreateGatewayClient)?;
    let selected_gateways = gateway_selector::select_gateways(
        &gateway_directory_client,
        enable_wireguard,
        nym_config.entry_point.clone(),
        nym_config.exit_point.clone(),
    )
    .await
    .map_err(Error::SelectGateways)?;

    // Create mixnet client
    let mut task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);

    let connect_mixnet_fut = tokio::time::timeout(
        MIXNET_CLIENT_STARTUP_TIMEOUT,
        crate::mixnet::setup_mixnet_client(
            selected_gateways.entry.identity(),
            &nym_config.data_path,
            task_manager.subscribe_named("mixnet_client_main"),
            nym_config.mixnet_client_config.clone(),
        ),
    );

    // Connect mixnet or catch cancellation
    // todo: look into whether this will work?
    let mixnet_client = tokio::select! {
        _ = shutdown_token.cancelled() => {
            _ = task_manager.signal_shutdown();
            task_manager.wait_for_shutdown().await;
            return Ok(())
        }
        timeout_result = connect_mixnet_fut => {
            timeout_result.map_err(|_| Error::StartMixnetClientTimeout)?
            .map_err(Error::SetupMixnetClient)?
        }
    };

    if enable_wireguard {
        let wireguard_tunnel = wireguard_tunnel::WireGuardTunnel::new(
            nym_config,
            task_manager,
            mixnet_client,
            gateway_directory_client,
            selected_gateways,
            shutdown_token,
        )
        .await?;

        wireguard_tunnel.run().await
    } else {
        mixnet_tunnel::MixnetTunnel::run(
            nym_config,
            task_manager,
            mixnet_client,
            gateway_directory_client,
            selected_gateways,
            shutdown_token,
        )
        .await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create gateway client")]
    CreateGatewayClient(#[source] nym_gateway_directory::Error),

    #[error("failed to select gateways")]
    SelectGateways(#[source] GatewayDirectoryError),

    #[error("start mixnet timeout")]
    StartMixnetClientTimeout,

    #[error("failed to setup mixnet client")]
    SetupMixnetClient(#[source] MixnetError),

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

    #[error("failed to lookup gateway ip: {gateway_id}: {source}")]
    FailedToLookupGatewayIp {
        gateway_id: String,
        source: nym_gateway_directory::Error,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
