use std::{env, ffi::OsString, time::Duration};

use nym_task::TaskManager;
use tokio::sync::broadcast;
use tracing::{error, info};
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceDependency, ServiceErrorControl,
        ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

use crate::{
    cli::CliArgs, command_interface::start_command_interface, logging, service::start_vpn_service,
};

use super::install;

windows_service::define_windows_service!(ffi_service_main, service_main);

pub(crate) static SERVICE_NAME: &str = "nym-vpnd";
pub(crate) static SERVICE_DISPLAY_NAME: &str = "NymVPN Service";

pub(crate) static SERVICE_DESCRIPTION: &str =
    "A service that creates and runs tunnels to the Nym network";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

fn service_main(arguments: Vec<OsString>) {
    if let Err(err) = run_service(arguments) {
        // Handle error in some way.
        println!("service_main: {:?}", err);
        error!("service_main: {:?}", err);
    }
}

fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    info!("Setting up event handler");

    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                event_tx.send(()).unwrap();

                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    info!("Service is starting...");

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(20),
        process_id: None,
    })?;

    let task_manager = TaskManager::new(10).named("nym_vpnd");
    let service_task_client = task_manager.subscribe_named("vpn_service");

    let state_changes_tx = broadcast::channel(10).0;

    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    // Start the command interface that listens for commands from the outside
    let (command_handle, vpn_command_rx) =
        start_command_interface(state_changes_tx.subscribe(), task_manager, None, event_rx);

    // Start the VPN service that wraps the actual VPN
    let vpn_handle = start_vpn_service(state_changes_tx, vpn_command_rx, service_task_client);

    info!("Service has started");

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

    vpn_handle.join().unwrap();
    command_handle.join().unwrap();

    info!("Service is stopping!");

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

    info!("Service has stopped!");

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

pub(crate) fn start(args: CliArgs) -> Result<(), windows_service::Error> {
    if args.command.install {
        println!(
            "Processing request to install {} as a service...",
            SERVICE_NAME
        );
        install::install_service()?;
        return Ok(());
    }

    if args.command.uninstall {
        println!(
            "Processing request to uninstall {} as a service...",
            SERVICE_NAME
        );
        install::uninstall_service()?;
        return Ok(());
    }

    if args.command.start {
        println!("Processing request to start service {}...", SERVICE_NAME);
        install::start_service()?;
        return Ok(());
    }

    if args.command.run_as_service {
        // TODO: enable this through setting or flag
        // println!("Configuring logging source...");
        // eventlog::init(SERVICE_DISPLAY_NAME, log::Level::Info).unwrap();

        println!("Configuring logging to file...");
        let _guard = logging::setup_logging_to_file();

        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    }
    Ok(())
}
