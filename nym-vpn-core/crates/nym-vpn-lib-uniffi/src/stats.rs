// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use nym_statistics::StatisticsSender;
use nym_vpn_lib::new_user_agent;
use nym_vpn_network_config::Network;

use super::{STATISTICS_CONTROLLER_HANDLE, error::VpnError};

pub(super) async fn init_statistics_controller(
    data_dir: PathBuf,
    network: Network,
    enabled: bool,
) -> Result<(), VpnError> {
    let mut guard = STATISTICS_CONTROLLER_HANDLE.lock().await;

    if guard.is_none() {
        let statistics_controller_handle =
            start_statistics_controller(data_dir, network, enabled).await;
        *guard = Some(statistics_controller_handle);
        Ok(())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Statistics controller is already running.".to_owned(),
        })
    }
}

pub(super) async fn stop_statistics_controller() -> Result<(), VpnError> {
    let mut guard = STATISTICS_CONTROLLER_HANDLE.lock().await;

    match guard.take() {
        Some(statistics_controller_handle) => {
            statistics_controller_handle.shutdown_and_wait().await;
            Ok(())
        }
        None => Err(VpnError::InvalidStateError {
            details: "Statistics controller is not running.".to_owned(),
        }),
    }
}

async fn start_statistics_controller(
    data_dir: PathBuf,
    network_env: Network,
    enabled: bool,
) -> StatisticsControllerHandle {
    // TODO: pass in as argument
    let user_agent = new_user_agent!();
    let shutdown_token = CancellationToken::new();

    let statistics_controller_config = nym_vpn_lib_types::NetworkStatisticsConfig {
        enabled,
        allow_disconnected: false,
    };

    let statistics_api_url = network_env
        .system_configuration
        .as_ref()
        .and_then(|config| config.statistics_api.clone());

    let stats_api_client = statistics_api_url.and_then(|url| nym_statistics_api_client::StatisticsApiClient::new(url.clone(), user_agent.clone()).inspect_err(|e| tracing::error!("Failed to build Statistics API client. Statistics collection will be disabled : {e}")).ok());

    let statistics_controller = nym_statistics::StatisticsController::new(
        statistics_controller_config,
        stats_api_client,
        data_dir,
        shutdown_token.child_token(),
    )
    .await;

    let statistics_events_sender = statistics_controller.get_statistics_sender();

    let statistics_controller_handle = tokio::spawn(statistics_controller.run());

    StatisticsControllerHandle {
        handle: statistics_controller_handle,
        statistics_events_sender,
        shutdown_token,
    }
}

pub(super) struct StatisticsControllerHandle {
    handle: JoinHandle<()>,
    statistics_events_sender: StatisticsSender,
    shutdown_token: CancellationToken,
}

impl StatisticsControllerHandle {
    async fn shutdown_and_wait(self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.handle.await {
            tracing::error!("Failed to join on statistics controller handle: {}", e);
        }
    }
}

pub(super) async fn get_events_sender() -> Result<StatisticsSender, VpnError> {
    if let Some(guard) = &*STATISTICS_CONTROLLER_HANDLE.lock().await {
        Ok(guard.statistics_events_sender.clone())
    } else {
        Err(VpnError::InvalidStateError {
            details: "Statistics controller is not running.".to_owned(),
        })
    }
}
