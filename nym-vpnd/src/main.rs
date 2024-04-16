// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(unix), allow(dead_code))]

#[cfg(unix)]
mod cli;
#[cfg(unix)]
mod command_interface;
#[cfg(unix)]
mod logging;
#[cfg(unix)]
mod service;
mod grpc;

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    use nym_task::TaskManager;
    use nym_vpn_lib::nym_config::defaults::setup_env;

    use crate::{
        cli::CliArgs, command_interface::start_command_interface, logging::setup_logging,
        service::start_vpn_service,
    };

    setup_logging();
    let args = CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    let task_manager = TaskManager::new(10).named("nym_vpnd");
    let service_task_client = task_manager.subscribe_named("vpn_service");

    // The idea here for explicly starting two separate runtimes is to make sure they are properly
    // separated. Looking ahead a little ideally it would be nice to be able for the command
    // interface to be able to forcefully terminate the vpn if needed.

    // Start the command interface that listens for commands from the outside
    let (command_handle, vpn_command_rx) = start_command_interface(task_manager, &args);

    // Start the VPN service that wraps the actual VPN
    let vpn_handle = start_vpn_service(vpn_command_rx, service_task_client);

    vpn_handle.join().unwrap();
    command_handle.join().unwrap();

    Ok(())
}

#[cfg(not(unix))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unimplemented!("Daemon not implemented for non-unix platforms");
}
