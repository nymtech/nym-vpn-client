// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod installation;
mod persistent_service_status;

use std::{ffi::OsString, sync::LazyLock, time::Duration};

use anyhow::Context;
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

use nym_vpn_lib::{config::GlobalConfig, logging::LogFileRemoverHandle};

use crate::RunParameters;
use persistent_service_status::PersistentServiceStatus;

windows_service::define_windows_service!(ffi_service_main, service_main);

pub static SERVICE_NAME: &str = "nym-vpnd";
pub static SERVICE_DISPLAY_NAME: &str = "NymVPN Service";
pub static SERVICE_DESCRIPTION: &str = "A service that creates and runs tunnels to the Nym network";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

static SHARED_SERVICE_STATE: LazyLock<Mutex<Option<SharedServiceState>>> =
    LazyLock::new(|| Mutex::new(None));

/// Exit codes used by Nym windows service.
#[repr(u32)]
pub enum ServiceSpecificExitCode {
    /// Failure to fetch network environment
    FetchNetworkEnvironment = 1,
    /// Failure to setup vpn service
    SetupVpnService = 2,
}

impl From<ServiceSpecificExitCode> for ServiceExitCode {
    fn from(value: ServiceSpecificExitCode) -> Self {
        ServiceExitCode::ServiceSpecific(value as u32)
    }
}

enum ServiceEvent {
    Stop { completion_tx: oneshot::Sender<()> },
    PreShutdown { completion_tx: oneshot::Sender<()> },
}

#[derive(Clone)]
struct SharedServiceState {
    runtime_handle: tokio::runtime::Handle,
    run_parameters: RunParameters,
    global_config: GlobalConfig,
    log_file_remover_handle: Option<LogFileRemoverHandle>,
    shutdown_token: CancellationToken,
}

pub async fn start(
    run_parameters: RunParameters,
    global_config: GlobalConfig,
    log_file_remover_handle: Option<LogFileRemoverHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    let initial_state = SharedServiceState {
        runtime_handle: tokio::runtime::Handle::current(),
        run_parameters,
        global_config,
        log_file_remover_handle,
        shutdown_token,
    };

    *SHARED_SERVICE_STATE.lock().await = Some(initial_state);

    // Start service dispatcher blocking this thread until the service is stopped.
    tokio::task::spawn_blocking(|| service_dispatcher::start(SERVICE_NAME, ffi_service_main))
        .await
        .with_context(|| "failed to join on service dispatcher task")??;
    Ok(())
}

fn service_main(_arguments: Vec<OsString>) {
    let rt = SHARED_SERVICE_STATE
        .blocking_lock()
        .as_ref()
        .expect("logical error: shared service state is not initialized")
        .runtime_handle
        .clone();

    if let Err(err) = rt.block_on(run_service()) {
        tracing::error!("service_main: {:?}", err);
    }
}

async fn run_service() -> anyhow::Result<()> {
    tracing::info!("Setting up event handler");

    let Some(service_state) = SHARED_SERVICE_STATE.lock().await.clone() else {
        return Err(anyhow::anyhow!(
            "logical error: shared service state is not initialized"
        ));
    };
    let run_params = service_state.run_parameters;

    let shutdown_token = service_state.shutdown_token;
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

    let endpoint_health = std::sync::Arc::new(nym_endpoint_health::EndpointHealthTracker::new());

    let network_cache = crate::environment::setup_environment(
        &run_params.paths.config_dir,
        &service_state.global_config.network_name,
        run_params.user_agent.clone(),
        endpoint_health.clone(),
    )
    .await
    .or_else(|err| {
        tracing::error!(
            "Failed to fetch network environment for {}: {}",
            service_state.global_config.network_name,
            err
        );

        persistent_status.set_stopped(ServiceExitCode::from(
            ServiceSpecificExitCode::FetchNetworkEnvironment,
        ))?;

        Err(err).with_context(|| "Failed to fetch network environment")
    })?;

    let join_handle = crate::spawn_vpn_service(
        run_params,
        network_cache,
        service_state.log_file_remover_handle,
        endpoint_health,
        shutdown_token.child_token(),
    )
    .await
    .or_else(|err| {
        tracing::error!("Failed to setup vpn service: {err}");

        persistent_status.set_stopped(ServiceExitCode::from(
            ServiceSpecificExitCode::SetupVpnService,
        ))?;

        Err(err)
    })?;

    tracing::info!("Service has started");
    persistent_status.set_running()?;

    if let Err(err) = join_handle.await {
        tracing::error!("Failed to join on vpn service task: {err}");
    }

    persistent_status.set_stopped(ServiceExitCode::NO_ERROR)?;
    tracing::info!("Service has stopped!");

    Ok(())
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
                        tracing::info!("Service is stopping");
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
