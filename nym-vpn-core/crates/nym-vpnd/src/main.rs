// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod cli;
mod command_interface;
mod discovery;
mod logging;
mod runtime;
mod service;
mod shutdown_handler;
mod types;
mod util;
#[cfg(windows)]
mod windows_service;

use std::sync::OnceLock;

use clap::Parser;
use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;
use service::NymVpnService;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{cli::CliArgs, command_interface::CommandInterfaceOptions};

// Lazy initialized global NymNetworkDetails
static GLOBAL_NETWORK_DETAILS: OnceLock<NymNetworkDetails> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    run()
}

fn set_global_network_details(network_details: NymNetworkDetails) -> anyhow::Result<()> {
    GLOBAL_NETWORK_DETAILS
        .set(network_details)
        .map_err(|_| anyhow::anyhow!("Failed to set network details"))
}

#[cfg(unix)]
fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let global_config_file = discovery::read_global_config_file()?;

    logging::setup_logging(args.command.run_as_service);

    if let Some(ref env) = args.config_env_file {
        nym_vpn_lib::nym_config::defaults::setup_env(Some(env));
        let network_details = NymNetworkDetails::new_from_env();
        set_global_network_details(network_details)?;
    } else {
        let network_name = global_config_file.network_name.clone();
        tracing::info!("Setting up environment by discovering the network: {network_name}");
        discovery::discover_env(&network_name)?;
    }

    run_inner(args)
}

#[cfg(windows)]
fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let global_config_file = discovery::read_global_config_file()?;

    if let Some(ref env) = args.config_env_file {
        nym_vpn_lib::nym_config::defaults::setup_env(Some(env));
        let network_details = NymNetworkDetails::new_from_env();
        GLOBAL_NETWORK_DETAILS
            .set(network_details)
            .map_err(|_| anyhow::anyhow!("Failed to set network details"))?;
    } else {
        let network_name = global_config_file.network_name.clone();
        tracing::info!("Setting up environment from discovery file: {network_name}");
        discovery::discover_env(&network_name)?;
    }

    if args.command.is_any() {
        Ok(windows_service::start(args)?)
    } else {
        setup_logging(false);
        run_inner(args)
    }
}

fn run_inner(args: CliArgs) -> anyhow::Result<()> {
    runtime::new_runtime().block_on(run_inner_async(args))
}

async fn run_inner_async(args: CliArgs) -> anyhow::Result<()> {
    let (state_changes_tx, state_changes_rx) = broadcast::channel(10);
    let (status_tx, status_rx) = broadcast::channel(10);
    let shutdown_token = CancellationToken::new();

    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        state_changes_rx,
        status_rx,
        Some(CommandInterfaceOptions {
            disable_socket_listener: args.disable_socket_listener,
            enable_http_listener: args.enable_http_listener,
        }),
        shutdown_token.child_token(),
    );

    let vpn_service_handle = NymVpnService::spawn(
        state_changes_tx,
        vpn_command_rx,
        status_tx,
        shutdown_token.child_token(),
    );

    let mut shutdown_join_set = shutdown_handler::install(shutdown_token);

    if let Err(e) = vpn_service_handle.await {
        tracing::error!("Failed to join on vpn service: {}", e);
    }

    if let Err(e) = command_handle.await {
        tracing::error!("Failed to join on command interface: {}", e);
    }

    shutdown_join_set.shutdown().await;

    Ok(())
}
