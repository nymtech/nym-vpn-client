// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Context;
use windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
use windows_service::{
    service::{
        ServiceAccess, ServiceAction, ServiceActionType, ServiceFailureActions,
        ServiceFailureResetPeriod, ServiceState,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
    Error as ServiceError,
};

use super::{service::get_service_info, SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};

// see https://github.com/mullvad/windows-service-rs/blob/main/examples/install_service.rs
pub(super) fn install_service() -> anyhow::Result<()> {
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

// see https://github.com/mullvad/windows-service-rs/blob/main/examples/uninstall_service.rs
pub(super) fn uninstall_service() -> windows_service::Result<()> {
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
        sleep(Duration::from_secs(1));
    }
    println!("{} is marked for deletion.", SERVICE_NAME);

    Ok(())
}

pub(super) fn start_service() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::START;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    if service.query_status()?.current_state != ServiceState::Running {
        service.start(&[] as &[&std::ffi::OsStr])?;
    }
    Ok(())
}
