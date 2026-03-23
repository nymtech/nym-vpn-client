// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{OsStr, OsString},
    io,
    thread::sleep,
    time::{Duration, Instant},
};
use windows_service::{
    service::{
        Service, ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};
use windows_sys::Win32::Foundation::ERROR_SERVICE_ALREADY_RUNNING;

const SERVICE_NAME: &str = "nymvpn-split-tunnel";
const SERVICE_DISPLAY_NAME: &str = "NymVPN Split Tunnel Driver";
const DRIVER_FILE_NAME: &str = "nymvpn-split-tunnel.sys";
const START_WAIT_STATUS_TIMEOUT: Duration = Duration::from_secs(8);
const STOP_WAIT_STATUS_TIMEOUT: Duration = Duration::from_secs(8);

/// Install the service if it does not exist.
pub fn install_driver_service() -> Result<(), Error> {
    let scm = ServiceManager::local_computer(
        None::<OsString>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .map_err(Error::OpenServiceControlManager)?;

    if scm
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        .is_ok()
    {
        return Ok(());
    }

    // The driver file will be in the same directory as the executable
    let executable_path = std::env::current_exe()
        .map_err(|e| Error::General(format!("Failed to get current executable path: {e}")))?
        .with_file_name(DRIVER_FILE_NAME);
    if !executable_path.exists() {
        return Err(Error::General(format!(
            "Cannot install driver as the executable is missing at: {}",
            executable_path.display()
        )));
    }

    tracing::info!("Installing split tunnel service");

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::KERNEL_DRIVER,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    scm.create_service(
        &service_info,
        ServiceAccess::START | ServiceAccess::QUERY_STATUS,
    )
    .map_err(Error::InstallService)?;

    Ok(())
}

/// Uninstall the service if it exists.
pub fn uninstall_driver_service() -> Result<(), Error> {
    let _ = unsafe { stop_driver_service() };

    let scm = ServiceManager::local_computer(
        None::<OsString>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .map_err(Error::OpenServiceControlManager)?;

    if let Ok(service) = scm.open_service(SERVICE_NAME, ServiceAccess::DELETE) {
        tracing::info!("Uninstalling split tunnel service");

        service.delete().map_err(Error::InstallService)?;
    }

    Ok(())
}

/// Start the split tunnel driver service.
pub fn start_driver_service() -> Result<(), Error> {
    tracing::debug!("Starting split tunnel service");

    let service = connect_to_service()?;

    if let Err(error) = service.start::<&OsStr>(&[]) {
        if let windows_service::Error::Winapi(error) = &error
            && error.raw_os_error() == Some(ERROR_SERVICE_ALREADY_RUNNING as i32)
        {
            return Ok(());
        }
        return Err(Error::StartService(error));
    }

    wait_for_status(&service, ServiceState::Running, START_WAIT_STATUS_TIMEOUT)
}

/// Stop the split tunnel driver service if it is running.
///
/// # Safety
///
/// The driver must be reset before calling this function. Failing to do so prevents
/// the driver from freeing resources and unregistering its callbacks.
// TODO: This is due to a bug in the driver. `unsafe` may be removed when this is fixed.
pub unsafe fn stop_driver_service() -> Result<(), Error> {
    let service = connect_to_service()?;

    let _ = service.stop();

    wait_for_status(&service, ServiceState::Stopped, STOP_WAIT_STATUS_TIMEOUT)
}

fn connect_to_service() -> Result<Service, Error> {
    let scm = ServiceManager::local_computer(None::<OsString>, ServiceManagerAccess::CONNECT)
        .map_err(Error::OpenServiceControlManager)?;

    let service = scm
        .open_service(SERVICE_NAME, ServiceAccess::all())
        .map_err(Error::OpenServiceHandle)?;

    Ok(service)
}

fn wait_for_status(
    service: &Service,
    target_state: ServiceState,
    timeout: Duration,
) -> Result<(), Error> {
    let initial_time = Instant::now();
    loop {
        let status = service.query_status().map_err(Error::QueryServiceStatus)?;

        if status.current_state == target_state {
            break;
        }

        if initial_time.elapsed() >= timeout {
            return Err(Error::StatusTimeout);
        }

        sleep(Duration::from_millis(250));
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to open service control manager
    #[error("Failed to connect to service control manager")]
    OpenServiceControlManager(#[source] windows_service::Error),

    /// Failed to create a service handle
    #[error("Failed to open service")]
    OpenServiceHandle(#[source] windows_service::Error),

    /// Failed to install the split tunnel service
    #[error("Failed to install split tunnel device driver service")]
    InstallService(#[source] windows_service::Error),

    /// Failed to start split tunnel service
    #[error("Failed to start split tunnel device driver service")]
    StartService(#[source] windows_service::Error),

    /// Failed to check service status
    #[error("Failed to query service status")]
    QueryServiceStatus(#[source] windows_service::Error),

    /// Timed-out waiting for service status to change to expected state
    #[error("Timed out waiting for service to change to expected state")]
    StatusTimeout,

    /// Failed to connect to existing driver
    #[error("Failed to open service handle")]
    OpenHandle(#[source] super::driver::DeviceHandleError),

    /// Failed to reset existing driver
    #[error("Failed to reset driver state")]
    ResetDriver(#[source] io::Error),

    /// General error
    #[error("Split tunnel error: {0}")]
    General(String),
}
