// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser;
use nym_task::TaskManager;
use nym_vpn_lib::{nym_config::defaults::setup_env, SHUTDOWN_TIMER_SECS};
use tokio::sync::broadcast;

use crate::{
    cli::CliArgs,
    command_interface::{start_command_interface, CommandInterfaceOptions},
    logging::setup_logging,
    service::start_vpn_service,
};

mod cli;
mod command_interface;
mod logging;
mod service;
#[cfg(windows)]
mod windows_service;

fn run_inner(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let task_manager = TaskManager::new(SHUTDOWN_TIMER_SECS).named("nym_vpnd");
    let service_task_client = task_manager.subscribe_named("vpn_service");

    let state_changes_tx = broadcast::channel(10).0;

    // Channels used to send events from the OS system handler (windows service, dbus etc)
    let (_event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    // Start the command interface that listens for commands from the outside
    let (command_handle, vpn_command_rx) = start_command_interface(
        state_changes_tx.subscribe(),
        task_manager,
        Some(CommandInterfaceOptions {
            disable_socket_listener: args.disable_socket_listener,
            enable_http_listener: args.enable_http_listener,
        }),
        event_rx,
    );

    // Start the VPN service that wraps the actual VPN
    let vpn_handle = start_vpn_service(state_changes_tx, vpn_command_rx, service_task_client);

    vpn_handle.join().unwrap();
    command_handle.join().unwrap();

    Ok(())
}

#[cfg(unix)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let args = CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    run_inner(args)
}

#[cfg(windows)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    if args.command.is_any() {
        Ok(windows_service::start(args)?)
    } else {
        setup_logging();
        run_inner(args)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}
