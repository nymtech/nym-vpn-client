// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::OsString,
    time::{Duration, Instant},
};

use anyhow::{Context, anyhow};
use windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
use windows_service::{
    Error as ServiceError,
    service::{
        ServiceAccess, ServiceAction, ServiceActionType, ServiceDependency, ServiceErrorControl,
        ServiceFailureActions, ServiceFailureResetPeriod, ServiceInfo, ServiceStartType,
        ServiceState,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use super::{SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME, SERVICE_TYPE};

pub fn install_service() -> anyhow::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    println!("Registering event logger {SERVICE_DISPLAY_NAME}...");
    eventlog::register(SERVICE_DISPLAY_NAME)?;

    println!("Registering {SERVICE_NAME} service...");

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
                .with_context(|| "Failed to create service")?
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

    println!("{SERVICE_NAME} service was installed successfully.");

    Ok(())
}

pub async fn uninstall_service() -> anyhow::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    {
        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;

        // If the service does not exist, then it's not an error.
        let service = match service_manager.open_service(SERVICE_NAME, service_access) {
            Ok(service) => service,
            Err(ServiceError::Winapi(ref e))
                if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST.0 as i32) =>
            {
                println!("{SERVICE_NAME} service is not installed.");
                return Ok(());
            }
            Err(e) => return Err(anyhow!("Failed to open service {SERVICE_NAME}: {e}")),
        };

        service.delete()?;

        if service.query_status()?.current_state != ServiceState::Stopped {
            service.stop()?;
        }
    }

    // Poll until service is deleted or timeout.
    let start = Instant::now();
    let timeout = Duration::from_secs(30);
    while start.elapsed() < timeout {
        if let Err(windows_service::Error::Winapi(e)) =
            service_manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
            && e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST.0 as i32)
        {
            println!("{SERVICE_NAME} service was uninstalled successfully.");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!(
        "{SERVICE_NAME} service failed to uninstall in time"
    ))
}

pub fn start_service() -> anyhow::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::START;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    if service.query_status()?.current_state != ServiceState::Running {
        service.start(&[] as &[&std::ffi::OsStr])?;
    }

    let start = Instant::now();
    let timeout = Duration::from_secs(30);
    while start.elapsed() < timeout {
        if service.query_status()?.current_state == ServiceState::Running {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(anyhow!(
        "timed out waiting for the service to reach running"
    ))
}

fn get_service_info() -> ServiceInfo {
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: std::env::current_exe().unwrap(),
        launch_arguments: vec![OsString::from("-v"), OsString::from("run-as-service")],
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
