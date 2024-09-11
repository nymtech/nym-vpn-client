use std::{io, net::Ipv6Addr, path::PathBuf, time::Duration};

use nym_connection_monitor::ConnectionMonitorTask;
use nym_gateway_directory::{EntryPoint, ExitPoint, GatewayClient};
use nym_ip_packet_client::IprClientConnect;
use nym_sdk::{
    mixnet::{MixnetClientBuilder, NodeIdentity, StoragePaths},
    NymNetworkDetails, UserAgent,
};
use nym_task::TaskManager;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tun2::{AbstractDevice, AsyncDevice};

use crate::{
    mixnet::SharedMixnetClient, GatewayDirectoryError, GenericNymVpnConfig, MixnetClientConfig,
    MixnetError,
};

const MIXNET_CLIENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;
const DEFAULT_TUN_MTU: u16 = 1500;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create gateway client")]
    CreateGatewayClient(#[source] nym_gateway_directory::Error),

    #[error("failed to select gateways")]
    SelectGateways(#[source] GatewayDirectoryError),

    #[error("failed to setup mixnet")]
    SetupMixnet(#[source] MixnetError),

    #[error("start mixnet timeout")]
    StartMixnetClientTimeout,

    #[error("failed to setup mixnet client")]
    FailedToSetupMixnetClient(#[source] MixnetError),

    #[error("failed to connect ot ip packet router")]
    ConnectToIpPacketRouter(#[source] nym_ip_packet_client::Error),

    #[error("failed to create tun device")]
    CreateTunDevice(#[source] tun2::Error),

    #[error("failed to set ipv6 address on tunnel interface")]
    SetTunIpv6Addr(#[source] io::Error),

    #[error("failed to obtain tun name")]
    ObtainTunName(#[source] tun2::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct MixnetTunnel {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    shutdown_token: CancellationToken,
    processor_handle: JoinHandle<Result<AsyncDevice, MixnetError>>,
}

impl MixnetTunnel {
    pub async fn spawn(
        nym_config: GenericNymVpnConfig,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        // Craft user agent.
        let user_agent = nym_config
            .user_agent
            .unwrap_or(UserAgent::from(nym_bin_common::bin_info_local_vergen!()));

        // Select gateways
        let gateway_directory_client = GatewayClient::new(nym_config.gateway_config, user_agent)
            .map_err(Error::CreateGatewayClient)?;
        let SelectedGateways { entry, exit } = select_gateways(
            &gateway_directory_client,
            false,
            nym_config.entry_point,
            nym_config.exit_point,
        )
        .await
        .map_err(Error::SelectGateways)?;

        // Create mixnet client
        let task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS);
        let mixnet_client = tokio::time::timeout(
            MIXNET_CLIENT_STARTUP_TIMEOUT,
            setup_mixnet_client(
                entry.identity(),
                &nym_config.data_path,
                task_manager.subscribe_named("mixnet_client_main"),
                nym_config.mixnet_client_config,
            ),
        )
        .await
        .map_err(|_| Error::StartMixnetClientTimeout)?
        .map_err(Error::FailedToSetupMixnetClient)?;

        // Setup mixnet routing
        let mixnet_client_address = mixnet_client.nym_address().await;
        let exit_mix_addresses = exit.ipr_address.unwrap();
        let mut ipr_client = IprClientConnect::new_from_inner(mixnet_client.inner()).await;

        // Create tun device
        let tun_addresses = ipr_client
            .connect(exit_mix_addresses.0, nym_config.nym_ips)
            .await
            .map_err(Error::ConnectToIpPacketRouter)?;
        let mut tun_config = tun2::Configuration::default();
        tun_config
            .mtu(nym_config.nym_mtu.unwrap_or(DEFAULT_TUN_MTU))
            .address(tun_addresses.ipv4)
            .up();
        let tun_device = tun2::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;
        let device_name = tun_device.tun_name().map_err(Error::ObtainTunName)?;
        set_tun_ipv6_addr(&device_name, tun_addresses.ipv6).map_err(Error::SetTunIpv6Addr)?;

        // Create connection monitor
        let connection_monitor = ConnectionMonitorTask::setup();

        // Create packet processor
        let processor_config = crate::mixnet::Config::new(exit_mix_addresses.0);
        let processor_handle = crate::mixnet::start_processor(
            processor_config,
            tun_device,
            mixnet_client.clone(),
            &task_manager,
            tun_addresses,
            &connection_monitor,
        )
        .await;

        let mixnet_client_sender = mixnet_client.split_sender().await;
        connection_monitor.start(
            mixnet_client_sender,
            mixnet_client_address,
            tun_addresses,
            exit_mix_addresses.0,
            &task_manager,
        );

        let tunnel = Self {
            task_manager,
            mixnet_client,
            shutdown_token,
            processor_handle,
        };

        Ok(tokio::spawn(tunnel.run()))
    }

    async fn run(mut self) {
        let mut shutdown_task_client = self.task_manager.subscribe();

        tokio::select! {
            _ = shutdown_task_client.recv() => {
                tracing::debug!("Task manager received shutdown.");
            }
            _ = self.shutdown_token.cancelled() => {
                tracing::debug!("Received cancellation. Shutting down task manager.");
                _ = self.task_manager.signal_shutdown();
            }
        }

        self.task_manager.wait_for_shutdown().await;
        self.mixnet_client.disconnect().await;
    }
}

struct SelectedGateways {
    entry: nym_gateway_directory::Gateway,
    exit: nym_gateway_directory::Gateway,
}

async fn select_gateways(
    gateway_directory_client: &GatewayClient,
    is_wireguard: bool,
    entry_point: EntryPoint,
    exit_point: ExitPoint,
) -> Result<SelectedGateways, GatewayDirectoryError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    let (mut entry_gateways, exit_gateways) = if is_wireguard {
        let all_gateways = gateway_directory_client
            .lookup_all_gateways()
            .await
            .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;
        (all_gateways.clone(), all_gateways)
    } else {
        // Setup the gateway that we will use as the exit point
        let exit_gateways = gateway_directory_client
            .lookup_exit_gateways()
            .await
            .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;
        // Setup the gateway that we will use as the entry point
        let entry_gateways = gateway_directory_client
            .lookup_entry_gateways()
            .await
            .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;
        (entry_gateways, exit_gateways)
    };

    let exit_gateway = entry_point
        .lookup_gateway(&exit_gateways)
        .await
        .map_err(|source| GatewayDirectoryError::FailedToSelectExitGateway { source })?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_gateway =
        exit_point
            .lookup_gateway(&entry_gateways)
            .map_err(|source| match source {
                nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation {
                    requested_location,
                    available_countries: _,
                } if Some(requested_location.as_str())
                    == exit_gateway.two_letter_iso_country_code() =>
                {
                    GatewayDirectoryError::SameEntryAndExitGatewayFromCountry {
                        requested_location: requested_location.to_string(),
                    }
                }
                _ => GatewayDirectoryError::FailedToSelectEntryGateway { source },
            })?;

    tracing::info!("Found {} entry gateways", entry_gateways.len());
    tracing::info!("Found {} exit gateways", exit_gateways.len());
    tracing::info!(
        "Using entry gateway: {}, location: {}, performance: {}",
        *entry_gateway.identity(),
        entry_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        entry_gateway
            .performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    tracing::info!(
        "Using exit gateway: {}, location: {}, performance: {}",
        *exit_gateway.identity(),
        exit_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        entry_gateway
            .performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    tracing::info!(
        "Using exit router address {}",
        exit_gateway
            .ipr_address
            .map_or_else(|| "none".to_string(), |ipr| ipr.to_string())
    );

    Ok(SelectedGateways {
        entry: entry_gateway,
        exit: exit_gateway,
    })
}

async fn setup_mixnet_client(
    mixnet_entry_gateway: &NodeIdentity,
    mixnet_client_key_storage_path: &Option<PathBuf>,
    mut task_client: nym_task::TaskClient,
    mixnet_client_config: MixnetClientConfig,
) -> Result<SharedMixnetClient, MixnetError> {
    let mut debug_config = nym_client_core::config::DebugConfig::default();
    apply_mixnet_client_config(&mixnet_client_config, &mut debug_config);

    let user_agent = nym_bin_common::bin_info_owned!().into();

    let mixnet_client = if let Some(path) = mixnet_client_key_storage_path {
        tracing::debug!("Using custom key storage path: {:?}", path);

        let gateway_id = mixnet_entry_gateway.to_base58_string();
        if let Err(err) =
            crate::credentials::check_imported_credential(path.to_path_buf(), &gateway_id).await
        {
            // UGLY: flow needs to restructured to sort this out, but I don't want to refactor all
            // that just before release.
            task_client.disarm();
            return Err(MixnetError::InvalidCredential {
                reason: err,
                path: path.to_path_buf(),
                gateway_id,
            });
        };

        let key_storage_path = StoragePaths::new_from_dir(path)
            .map_err(MixnetError::FailedToSetupMixnetStoragePaths)?;
        MixnetClientBuilder::new_with_default_storage(key_storage_path)
            .await
            .map_err(MixnetError::FailedToCreateMixnetClientWithDefaultStorage)?
            .with_user_agent(user_agent)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .credentials_mode(mixnet_client_config.enable_credentials_mode)
            .build()
            .map_err(MixnetError::FailedToBuildMixnetClient)?
            .connect_to_mixnet()
            .await
            .map_err(map_mixnet_connect_error)?
    } else {
        tracing::debug!("Using ephemeral key storage");
        MixnetClientBuilder::new_ephemeral()
            .with_user_agent(user_agent)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .credentials_mode(mixnet_client_config.enable_credentials_mode)
            .build()
            .map_err(MixnetError::FailedToBuildMixnetClient)?
            .connect_to_mixnet()
            .await
            .map_err(map_mixnet_connect_error)?
    };

    Ok(SharedMixnetClient::new(mixnet_client))
}

fn apply_mixnet_client_config(
    mixnet_client_config: &MixnetClientConfig,
    debug_config: &mut nym_client_core::config::DebugConfig,
) {
    let MixnetClientConfig {
        enable_poisson_rate,
        disable_background_cover_traffic,
        enable_credentials_mode: _enable_credentials_mode,
        min_mixnode_performance,
        min_gateway_performance,
    } = mixnet_client_config;

    // Disable Poisson rate limiter by default
    tracing::info!(
        "mixnet client poisson rate limiting: {}",
        true_to_enabled(*enable_poisson_rate)
    );
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = !enable_poisson_rate;

    tracing::info!(
        "mixnet client background loop cover traffic stream: {}",
        true_to_disabled(*disable_background_cover_traffic)
    );
    debug_config.cover_traffic.disable_loop_cover_traffic_stream =
        *disable_background_cover_traffic;

    if let Some(min_mixnode_performance) = min_mixnode_performance {
        debug_config.topology.minimum_mixnode_performance = *min_mixnode_performance;
    }
    tracing::info!(
        "mixnet client minimum mixnode performance: {}",
        debug_config.topology.minimum_mixnode_performance,
    );

    if let Some(min_gateway_performance) = min_gateway_performance {
        debug_config.topology.minimum_gateway_performance = *min_gateway_performance;
    }
    tracing::info!(
        "mixnet client minimum gateway performance: {}",
        debug_config.topology.minimum_gateway_performance,
    );
}

// Map some specific mixnet errors to more specific ones
fn map_mixnet_connect_error(err: nym_sdk::Error) -> MixnetError {
    match err {
        nym_sdk::Error::ClientCoreError(
            nym_client_core::error::ClientCoreError::GatewayClientError { gateway_id, source },
        ) => MixnetError::EntryGateway {
            gateway_id: gateway_id.to_string(),
            source: Box::new(source),
        },
        _ => MixnetError::FailedToConnectToMixnet(err),
    }
}

fn true_to_enabled(val: bool) -> &'static str {
    if val {
        "enabled"
    } else {
        "disabled"
    }
}

fn true_to_disabled(val: bool) -> &'static str {
    if val {
        "disabled"
    } else {
        "enabled"
    }
}

fn set_tun_ipv6_addr(_device_name: &str, _ipv6_addr: Ipv6Addr) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("ip")
        .args([
            "-6",
            "addr",
            "add",
            &_ipv6_addr.to_string(),
            "dev",
            _device_name,
        ])
        .output()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("ifconfig")
        .args([_device_name, "inet6", "add", &_ipv6_addr.to_string()])
        .output()?;

    Ok(())
}
