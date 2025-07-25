// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod installation;
mod persistent_service_status;

use std::{ffi::OsString, path::PathBuf, sync::LazyLock, time::Duration};

use anyhow::Context;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use windows_service::{
    service::{ServiceControl, ServiceExitCode, ServiceType},
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

use crate::{
    command_interface,
    logging::{LogFileRemover, LoggingSetup},
    runtime,
    service::{NymVpnService, NymVpnServiceParameters},
};
use persistent_service_status::PersistentServiceStatus;

windows_service::define_windows_service!(ffi_service_main, service_main);

pub static SERVICE_NAME: &str = "nym-vpnd";
pub static SERVICE_DISPLAY_NAME: &str = "NymVPN Service";
pub static SERVICE_DESCRIPTION: &str = "A service that creates and runs tunnels to the Nym network";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

static SHARED_SERVICE_STATE: LazyLock<Mutex<SharedServiceState>> =
    LazyLock::new(|| Mutex::new(SharedServiceState::default()));

/// Exit codes used by Nym windows service.
#[repr(u32)]
pub enum ServiceSpecificExitCode {
    /// Failure to fetch network environment
    FetchNetworkEnvironment = 1,
}

impl From<ServiceSpecificExitCode> for ServiceExitCode {
    fn from(value: ServiceSpecificExitCode) -> Self {
        match value {
            ServiceSpecificExitCode::FetchNetworkEnvironment => {
                ServiceExitCode::ServiceSpecific(value as u32)
            }
        }
    }
}

enum ServiceEvent {
    Stop { completion_tx: oneshot::Sender<()> },
    PreShutdown { completion_tx: oneshot::Sender<()> },
}

#[derive(Debug, Clone, Default)]
pub struct ServiceNetworkConfig {
    pub network: Option<String>,
    pub config_env_file: Option<PathBuf>,
}

#[derive(Default)]
struct SharedServiceState {
    network_config: ServiceNetworkConfig,
    logging_setup: Option<LoggingSetup>,
    sentry_enabled: bool,
}

pub fn start(
    network_config: ServiceNetworkConfig,
    logging_setup: Option<LoggingSetup>,
    sentry_enabled: bool,
) -> Result<(), windows_service::Error> {
    // Important: release mutex lock before starting service dispatcher to avoid deadlock.
    *SHARED_SERVICE_STATE.blocking_lock() = SharedServiceState {
        network_config,
        logging_setup,
        sentry_enabled,
    };

    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

fn service_main(_arguments: Vec<OsString>) {
    let rt = runtime::new_runtime();

    if let Err(err) = rt.block_on(run_service()) {
        tracing::error!("service_main: {:?}", err);
    }
}

async fn run_service() -> anyhow::Result<()> {
    tracing::info!("Setting up event handler");

    let (service_event_tx, mut service_event_rx) = mpsc::unbounded_channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                let (completion_tx, completion_rx) = oneshot::channel();
                if service_event_tx
                    .send(ServiceEvent::Stop { completion_tx })
                    .inspect_err(|e| {
                        tracing::error!("Failed to send stop: {}", e);
                    })
                    .is_ok()
                {
                    let _ = completion_rx.blocking_recv();
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Preshutdown => {
                let (completion_tx, completion_rx) = oneshot::channel();
                if service_event_tx
                    .send(ServiceEvent::PreShutdown { completion_tx })
                    .inspect_err(|e| {
                        tracing::error!("Failed to send preshutdown: {}", e);
                    })
                    .is_ok()
                {
                    let _ = completion_rx.blocking_recv();
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
    let mut persistent_status = PersistentServiceStatus::new(SERVICE_TYPE, status_handle);

    let shutdown_token = CancellationToken::new();
    let cloned_shutdown_token = shutdown_token.clone();
    let mut cloned_persistent_status = persistent_status.clone();
    tokio::spawn(async move {
        while let Some(service_event) = service_event_rx.recv().await {
            match service_event {
                ServiceEvent::Stop { completion_tx } => {
                    tracing::info!("Received stop.");

                    if !cloned_shutdown_token.is_cancelled() {
                        if let Err(e) =
                            cloned_persistent_status.set_pending_stop(Duration::from_secs(20))
                        {
                            tracing::error!("Failed to set pending stop: {}", e);
                        }
                        cloned_shutdown_token.cancel();
                    }

                    _ = completion_tx.send(());
                }
                ServiceEvent::PreShutdown { completion_tx } => {
                    tracing::info!("Received shutdown.");
                    // todo: lock firewall and initiate shutdown
                    _ = completion_tx.send(());
                }
            }
        }
        tracing::debug!("Exiting service event handler.");
    });

    tracing::info!("Service is starting...");
    persistent_status.set_pending_start(Duration::from_secs(20))?;

    let mut shared_service_state = SHARED_SERVICE_STATE.lock().await;
    let network_config = shared_service_state.network_config.clone();
    let logging_setup = shared_service_state.logging_setup.take();
    let sentry_enabled = shared_service_state.sentry_enabled;
    let log_path = shared_service_state
        .logging_setup
        .as_ref()
        .map(|setup| setup.log_path().clone());
    // explicitly release mutex lock
    _ = shared_service_state;

    let global_config_file = crate::setup_global_config(network_config.network.clone())?;
    let netstats_enabled = global_config_file.collect_network_statistics;

    let network_env = match crate::environment::setup_environment(
        &global_config_file,
        network_config.config_env_file.as_deref(),
    )
    .await
    {
        Ok(network_env) => network_env,
        Err(err) => {
            persistent_status.set_stopped(ServiceExitCode::from(
                ServiceSpecificExitCode::FetchNetworkEnvironment,
            ))?;

            tracing::error!(
                "Failed to fetch network environment for '{}': {}",
                network_config.network.as_deref().unwrap_or("mainnet"),
                err
            );

            return Err(err).with_context(|| "Failed to fetch network environment");
        }
    };

    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let (file_logging_event_tx, file_logging_event_rx) = mpsc::unbounded_channel();

    let file_logging_handle = logging_setup.map(|logging_setup| {
        tokio::spawn(
            LogFileRemover::new(
                file_logging_event_rx,
                logging_setup,
                shutdown_token.child_token(),
            )
            .run(),
        )
    });

    let (command_handle, vpn_command_rx) =
        command_interface::start_command_interface(tunnel_event_rx, shutdown_token.child_token())
            .await?;

    let user_agent = crate::user_agent::construct_user_agent();
    let parameters = NymVpnServiceParameters {
        network_env,
        sentry_enabled,
        netstats_enabled,
        stats_id_seed: None,
        log_path,
        user_agent,
    };
    let service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        file_logging_event_tx,
        parameters,
        shutdown_token.child_token(),
    );

    tracing::info!("Service has started");
    persistent_status.set_running()?;

    if let Err(e) = service_handle.await {
        tracing::error!("Failed to join on vpn service: {}", e);
    }

    if let Err(e) = command_handle.await {
        tracing::error!("Failed to join on command interface: {}", e);
    }

    let _worker_guard = if let Some(file_logging_handle) = file_logging_handle {
        file_logging_handle
            .await
            .inspect_err(|e| tracing::error!("Failed to join on file logging: {}", e))
            .ok()
    } else {
        None
    };

    tracing::info!("Service is stopping!");
    persistent_status.set_stopped(ServiceExitCode::NO_ERROR)?;

    tracing::info!("Service has stopped!");

    Ok(())
}
