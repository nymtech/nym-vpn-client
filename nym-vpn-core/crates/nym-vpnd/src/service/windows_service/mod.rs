// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod persistent_service_status;

use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    sync::LazyLock,
    time::{Duration, Instant},
};

use anyhow::Context;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_util::sync::CancellationToken;
use windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
use windows_service::{
    service::{
        ServiceAccess, ServiceAction, ServiceActionType, ServiceControl, ServiceDependency,
        ServiceErrorControl, ServiceExitCode, ServiceFailureActions, ServiceFailureResetPeriod,
        ServiceInfo, ServiceStartType, ServiceState, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
    Error as ServiceError,
};

use crate::{
    command_interface,
    logging::{LogFileRemover, LoggingSetup},
    runtime,
    service::NymVpnService,
};
use persistent_service_status::PersistentServiceStatus;

windows_service::define_windows_service!(ffi_service_main, service_main);

pub static SERVICE_NAME: &str = "nym-vpnd";
pub static SERVICE_DISPLAY_NAME: &str = "NymVPN Service";

pub static SERVICE_DESCRIPTION: &str = "A service that creates and runs tunnels to the Nym network";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

pub const FETCH_NETWORK_EXIT_CODE: u32 = 1;

enum ServiceEvent {
    Stop { completion_tx: oneshot::Sender<()> },
    PreShutdown { completion_tx: oneshot::Sender<()> },
}

#[derive(Debug, Clone, Default)]
pub struct ServiceNetworkConfig {
    pub network: Option<String>,
    pub config_env_file: Option<PathBuf>,
}

/// Network configuration passed from `main()` and used later to fetch network environment.
static SERVICE_NETWORK_CONFIG: LazyLock<Mutex<ServiceNetworkConfig>> =
    LazyLock::new(|| Mutex::new(ServiceNetworkConfig::default()));

/// Logging setup passed from `main()` and used later to interact with logging.
static LOGGING_SETUP: LazyLock<Mutex<Option<LoggingSetup>>> = LazyLock::new(|| Mutex::new(None));

fn service_main(arguments: Vec<OsString>) {
    if let Err(err) = run_service(arguments) {
        tracing::error!("service_main: {:?}", err);
    }
}

fn run_service(_args: Vec<OsString>) -> anyhow::Result<()> {
    runtime::new_runtime().block_on(run_service_inner())
}

async fn run_service_inner() -> anyhow::Result<()> {
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

    let network_config = (*SERVICE_NETWORK_CONFIG.lock().await).clone();
    let logging_setup = (*LOGGING_SETUP.lock().await).take();
    let log_path = logging_setup.as_ref().map(|setup| setup.log_path.clone());
    let cloned_network_config = network_config.clone();
    let network_env_result = tokio::task::spawn(async move {
        let global_config_file =
            crate::setup_global_config(cloned_network_config.network.as_deref())?;
        crate::environment::setup_environment(
            &global_config_file,
            cloned_network_config.config_env_file.as_deref(),
        )
        .await
    })
    .await;
    let network_env = match network_env_result {
        Ok(Ok(network_env)) => {
            network_env.export_to_env();
            network_env
        }
        Ok(Err(err)) => {
            persistent_status
                .set_stopped(ServiceExitCode::ServiceSpecific(FETCH_NETWORK_EXIT_CODE))?;

            tracing::error!(
                "Failed to fetch network environment for '{}': {}",
                network_config.network.as_deref().unwrap_or("mainnet"),
                err
            );
            return Err(err).with_context(|| "Failed to fetch network environment");
        }
        Err(err) => {
            tracing::error!("Failed to join on network fetch task: {}", err);
            return Err(err).with_context(|| "Failed to join on network fetch task");
        }
    };

    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let (file_logging_event_tx, file_logging_event_rx) = mpsc::channel(1);

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

    // Start the command interface that listens for commands from the outside
    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        tunnel_event_rx,
        network_env.clone(),
        shutdown_token.child_token(),
    );

    let user_agent = crate::util::construct_user_agent();

    // Start the VPN service that wraps the actual VPN
    let vpn_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        file_logging_event_tx,
        shutdown_token.child_token(),
        network_env,
        user_agent,
        log_path,
    );

    tracing::info!("Service has started");
    persistent_status.set_running()?;

    if let Err(e) = vpn_handle.await {
        tracing::error!("Failed to join on vpn service: {}", e);
    }

    if let Err(e) = command_handle.await {
        tracing::error!("Failed to join on command interface: {}", e);
    }

    if let Some(file_logging_handle) = file_logging_handle {
        if let Err(e) = file_logging_handle.await {
            tracing::error!("Failed to join on file logging: {}", e);
        }
    }

    tracing::info!("Service is stopping!");
    persistent_status.set_stopped(ServiceExitCode::NO_ERROR)?;

    tracing::info!("Service has stopped!");

    Ok(())
}

pub(super) fn get_service_info() -> ServiceInfo {
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: env::current_exe().unwrap(),
        launch_arguments: vec![OsString::from("--run-as-service")],
        dependencies: vec![
            // Base Filter Engine
            ServiceDependency::Service(OsString::from("BFE")),
            // Network Store Interface Service
            // This service delivers network notifications (e.g. interface addition/deleting etc).
            ServiceDependency::Service(OsString::from("NSI")),
        ],
        account_name: None, // run as System
        account_password: None,
    }
}

pub fn start(
    service_network_config: ServiceNetworkConfig,
    logging_setup: Option<LoggingSetup>,
) -> Result<(), windows_service::Error> {
    // Important: release mutex lock before starting service dispatcher to avoid deadlock.
    *SERVICE_NETWORK_CONFIG.blocking_lock() = service_network_config;
    *LOGGING_SETUP.blocking_lock() = logging_setup;

    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

pub fn install_service() -> anyhow::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    println!("Registering event logger {}...", SERVICE_DISPLAY_NAME);
    eventlog::register(SERVICE_DISPLAY_NAME).unwrap();

    println!("Registering {} service...", SERVICE_NAME);

    let service_access = ServiceAccess::QUERY_CONFIG
        | ServiceAccess::QUERY_STATUS
        | ServiceAccess::CHANGE_CONFIG
        | ServiceAccess::START;
    let service_info = get_service_info();
    let service = match service_manager.open_service(SERVICE_NAME, service_access) {
        Ok(service) => {
            service
                .change_config(&service_info)
                .with_context(|| "Failed to change service config")?;
            service
        }
        Err(ServiceError::Winapi(io_error))
            // Safety: i32 cast cannot fail because `ERROR_SERVICE_DOES_NOT_EXIST` is within i32 boundaries
            if io_error.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST.0 as i32) =>
        {
            service_manager
                .create_service(&service_info, service_access)
                .with_context(|| "Failed to open service")?
        }
        Err(e) => Err(e).with_context(|| "Failed to open service")?,
    };

    let recovery_actions = vec![
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(3),
        },
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(30),
        },
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(60 * 10),
        },
    ];

    let failure_actions = ServiceFailureActions {
        reset_period: ServiceFailureResetPeriod::After(Duration::from_secs(60 * 15)),
        reboot_msg: None,
        command: None,
        actions: Some(recovery_actions),
    };

    service
        .update_failure_actions(failure_actions)
        .with_context(|| "Failed to update failure actions")?;
    service
        .set_failure_actions_on_non_crash_failures(true)
        .with_context(|| "Failed to set failure actions on non-crash failures")?;
    service
        .set_description(SERVICE_DESCRIPTION)
        .with_context(|| "Failed to set service description")?;

    println!("{} service has been registered.", SERVICE_NAME);

    Ok(())
}

pub fn uninstall_service() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // The service will be marked for deletion as long as this function call succeeds.
    // However, it will not be deleted from the database until it is stopped and all open handles to it are closed.
    service.delete()?;
    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }
    // Explicitly close our open handle to the service. This is automatically called when `service` goes out of scope.
    drop(service);

    // Win32 API does not give us a way to wait for service deletion.
    // To check if the service is deleted from the database, we have to poll it ourselves.
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    while start.elapsed() < timeout {
        if let Err(windows_service::Error::Winapi(e)) =
            service_manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        {
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST.0 as i32) {
                println!("{} is deleted.", SERVICE_NAME);
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    println!("{} is marked for deletion.", SERVICE_NAME);

    Ok(())
}

pub fn start_service() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::START;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    if service.query_status()?.current_state != ServiceState::Running {
        service.start(&[] as &[&std::ffi::OsStr])?;
    }
    Ok(())
}
