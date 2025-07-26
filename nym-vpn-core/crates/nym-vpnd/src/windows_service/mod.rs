// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod installation;
mod persistent_service_status;

use std::{ffi::OsString, path::PathBuf, sync::LazyLock, time::Duration};

use anyhow::Context;
use nym_common::trace_err_chain;
use nym_vpnd_types::log_path::LogPath;
use tokio::{
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use windows_service::{
    service::{ServiceControl, ServiceExitCode, ServiceType},
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
};

use crate::logging::RemoveLogFileHandle;
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
    /// Failure to start command interface
    StartCommandInterface = 2,
}

impl From<ServiceSpecificExitCode> for ServiceExitCode {
    fn from(value: ServiceSpecificExitCode) -> Self {
        ServiceExitCode::ServiceSpecific(value as u32)
    }
}

impl From<crate::SetupServiceError> for ServiceSpecificExitCode {
    fn from(error: crate::SetupServiceError) -> Self {
        match error {
            crate::SetupServiceError::StartCommandInterface(_) => Self::StartCommandInterface,
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
    log_path: Option<LogPath>,
    network_config: ServiceNetworkConfig,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    sentry_enabled: bool,
    shutdown_token: CancellationToken,
    runtime_handle: Option<tokio::runtime::Handle>,
}

pub async fn start(
    log_path: Option<LogPath>,
    network_config: ServiceNetworkConfig,
    sentry_enabled: bool,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    shutdown_token: CancellationToken,
    runtime_handle: tokio::runtime::Handle,
) -> anyhow::Result<()> {
    *SHARED_SERVICE_STATE.lock().await = SharedServiceState {
        log_path,
        network_config,
        remove_log_file_handle,
        sentry_enabled,
        shutdown_token,
        runtime_handle: Some(runtime_handle),
    };

    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    tokio::task::spawn_blocking(|| service_dispatcher::start(SERVICE_NAME, ffi_service_main))
        .await
        .with_context(|| "failed to join on service dispatcher task")??;
    Ok(())
}

fn service_main(_arguments: Vec<OsString>) {
    let rt = SHARED_SERVICE_STATE
        .blocking_lock()
        .runtime_handle
        .as_ref()
        .expect("runtime must be set prior to the call to service_main()")
        .clone();

    if let Err(err) = rt.block_on(run_service()) {
        tracing::error!("service_main: {:?}", err);
    }
}

async fn run_service() -> anyhow::Result<()> {
    tracing::info!("Setting up event handler");

    let shutdown_token = SHARED_SERVICE_STATE.lock().await.shutdown_token.clone();
    let (service_event_tx, service_event_rx) = mpsc::unbounded_channel();
    let status_handle = register_service_event_handler(service_event_tx)?;
    let mut persistent_status = PersistentServiceStatus::new(SERVICE_TYPE, status_handle);
    let _event_processor_handle = start_service_event_processor(
        service_event_rx,
        persistent_status.clone(),
        shutdown_token.clone(),
    );

    tracing::info!("Service is starting...");
    persistent_status.set_pending_start(Duration::from_secs(20))?;

    let shared_service_state = SHARED_SERVICE_STATE.lock().await;
    let log_path = shared_service_state.log_path.clone();
    let remove_log_file_handle = shared_service_state.remove_log_file_handle.clone();
    let network_config = shared_service_state.network_config.clone();
    let sentry_enabled = shared_service_state.sentry_enabled;
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
            tracing::error!(
                "Failed to fetch network environment for '{}': {}",
                network_config.network.as_deref().unwrap_or("mainnet"),
                err
            );

            persistent_status.set_stopped(ServiceExitCode::from(
                ServiceSpecificExitCode::FetchNetworkEnvironment,
            ))?;

            return Err(err).with_context(|| "Failed to fetch network environment");
        }
    };

    let vpn_service_run_config = crate::VpnServiceSetupParameters {
        log_path,
        network_env,
        sentry_enabled,
        netstats_enabled,
        stats_id_seed: None,
        user_agent: None,
    };

    match crate::setup_vpn_service(
        vpn_service_run_config,
        remove_log_file_handle,
        shutdown_token,
    )
    .await
    {
        Ok(vpn_service_runtime) => {
            tracing::info!("Service has started");
            persistent_status.set_running()?;

            vpn_service_runtime.wait_until_shutdown().await;

            tracing::info!("Service is stopping!");
            persistent_status.set_stopped(ServiceExitCode::NO_ERROR)?;

            tracing::info!("Service has stopped!");

            Ok(())
        }
        Err(err) => {
            trace_err_chain!(err, "failed to setup vpn service");

            persistent_status.set_stopped(ServiceExitCode::from(
                ServiceSpecificExitCode::StartCommandInterface,
            ))?;

            Err(anyhow::Error::new(err))
        }
    }
}

fn register_service_event_handler(
    service_event_tx: mpsc::UnboundedSender<ServiceEvent>,
) -> windows_service::Result<ServiceStatusHandle> {
    service_control_handler::register(
        SERVICE_NAME,
        move |control_event| -> ServiceControlHandlerResult {
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
        },
    )
}

fn start_service_event_processor(
    mut service_event_rx: mpsc::UnboundedReceiver<ServiceEvent>,
    mut persistent_status: PersistentServiceStatus,
    shutdown_token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(service_event) = service_event_rx.recv().await {
            match service_event {
                ServiceEvent::Stop { completion_tx } => {
                    tracing::info!("Received stop.");

                    if !shutdown_token.is_cancelled() {
                        if let Err(e) = persistent_status.set_pending_stop(Duration::from_secs(20))
                        {
                            tracing::error!("Failed to set pending stop: {}", e);
                        }
                        shutdown_token.cancel();
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
    })
}
