// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// mod account;
mod cli;
mod command_interface;
mod logging;
mod service;
mod types;
mod util;
#[cfg(windows)]
mod windows_service;

use clap::Parser;
use nym_vpn_lib::nym_config::defaults::setup_env;
use service::NymVpnService;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{
    cli::CliArgs,
    command_interface::{start_command_interface, CommandInterfaceOptions},
    logging::setup_logging,
};

fn run_inner(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .build()
        .unwrap()
        .block_on(run_inner_async(args))
}

async fn run_inner_async(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let state_changes_tx = broadcast::channel(10).0;
    let shutdown_token = CancellationToken::new();

    // Start the command interface that listens for commands from the outside
    let (command_handle, vpn_command_rx) = start_command_interface(
        state_changes_tx.subscribe(),
        Some(CommandInterfaceOptions {
            disable_socket_listener: args.disable_socket_listener,
            enable_http_listener: args.enable_http_listener,
        }),
        shutdown_token.child_token(),
    );

    // Start the VPN service that wraps the actual VPN
    let vpn_service_handle = NymVpnService::spawn(
        state_changes_tx,
        vpn_command_rx,
        shutdown_token.child_token(),
    );

    vpn_service_handle.await;
    command_handle.await;

    Ok(())
}

#[cfg(unix)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    setup_logging(args.command.run_as_service);
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
        setup_logging(false);
        run_inner(args)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}
