// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    env,
    ffi::OsString,
    io,
    time::{Duration, Instant},
};

use anyhow::Context;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
use windows_service::{
    service::{
        ServiceAccess, ServiceAction, ServiceActionType, ServiceControl, ServiceControlAccept,
        ServiceDependency, ServiceErrorControl, ServiceExitCode, ServiceFailureActions,
        ServiceFailureResetPeriod, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
    Error as ServiceError,
};

use crate::{command_interface, runtime, service::NymVpnService};

windows_service::define_windows_service!(ffi_service_main, service_main);

pub static SERVICE_NAME: &str = "nym-vpnd";
pub static SERVICE_DISPLAY_NAME: &str = "NymVPN Service";

pub static SERVICE_DESCRIPTION: &str = "A service that creates and runs tunnels to the Nym network";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

fn service_main(arguments: Vec<OsString>) {
    if let Err(err) = run_service(arguments) {
        println!("service_main {:?}", err);
        tracing::error!("service_main: {:?}", err);
    }
}

fn run_service(_args: Vec<OsString>) -> windows_service::Result<()> {
    // TODO: network selection is not yet implemented/supported
    let network_name = "mainnet";
    match nym_vpn_network_config::Network::fetch(network_name) {
        Ok(network_env) => {
            network_env.export_to_env();
            let rt = runtime::new_runtime();
            rt.block_on(run_service_inner(network_env))
        }
        Err(err) => {
            tracing::error!(
                "Failed to fetch network environment for '{}': {}",
                network_name,
                err
            );
            Err(windows_service::Error::Winapi(io::Error::new(
                io::ErrorKind::Other,
                "Failed to fetch network environment",
            )))
        }
    }
}

async fn run_service_inner(
    network_env: nym_vpn_network_config::Network,
) -> windows_service::Result<()> {
    tracing::info!("Setting up event handler");

    let shutdown_token = CancellationToken::new();
    let cloned_shutdown_token = shutdown_token.clone();
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // todo: check if this works without tokio runtime.
                cloned_shutdown_token.cancel();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    tracing::info!("Service is starting...");

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(20),
        process_id: None,
    })?;

    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);

    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    // Start the command interface that listens for commands from the outside
    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        tunnel_event_rx,
        None,
        network_env.clone(),
        shutdown_token.child_token(),
    );

    let user_agent = crate::util::construct_user_agent();

    // Start the VPN service that wraps the actual VPN
    let vpn_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        shutdown_token.child_token(),
        network_env,
        user_agent,
    );

    tracing::info!("Service has started");

    // Tell the system that the service is running now
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(20),
        process_id: None,
    })?;

    if let Err(e) = vpn_handle.await {
        tracing::error!("Failed to join on vpn service: {}", e);
    }

    if let Err(e) = command_handle.await {
        tracing::error!("Failed to join on command interface: {}", e);
    }

    tracing::info!("Service is stopping!");

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(20),
        process_id: None,
    })?;

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

pub fn start() -> Result<(), windows_service::Error> {
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
