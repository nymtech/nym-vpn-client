// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod cli;
mod command_interface;
mod config;
mod environment;
mod logging;
mod runtime;
mod service;
mod shutdown_handler;
mod util;
#[cfg(windows)]
mod windows_service;

use clap::Parser;
use nym_vpn_network_config::Network;
use service::NymVpnService;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{cli::CliArgs, command_interface::CommandInterfaceOptions, config::GlobalConfigFile};

fn main() -> anyhow::Result<()> {
    run()
}

#[cfg(unix)]
fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let mut global_config_file = GlobalConfigFile::read_from_file()?;

    if let Some(ref network) = args.network {
        global_config_file.network_name = network.to_owned();
        global_config_file.write_to_file()?;
    }

    let options = logging::Options {
        enable_file_log: args.command.run_as_service,
        enable_stdout_log: true,
    };
    let _guard = logging::setup_logging(options);

    let network_env = environment::setup_environment(&global_config_file, &args)?;

    run_inner(args, network_env)
}

#[cfg(windows)]
fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let mut global_config_file = GlobalConfigFile::read_from_file()?;

    if let Some(ref network) = args.network {
        global_config_file.network_name = network.to_owned();
        global_config_file.write_to_file()?;
    }

    let network_env = environment::setup_environment(&global_config_file, &args)?;

    if args.command.is_any() {
        Ok(windows_service::start(args)?)
    } else {
        let options = logging::Options {
            enable_file_log: false,
            enable_stdout_log: true,
        };
        let _guard = logging::setup_logging(options);
        run_inner(args, network_env)
    }
}

fn run_inner(args: CliArgs, network_env: Network) -> anyhow::Result<()> {
    runtime::new_runtime().block_on(run_inner_async(args, network_env))
}

async fn run_inner_async(args: CliArgs, network_env: Network) -> anyhow::Result<()> {
    network_env.check_consistency().await?;

    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let shutdown_token = CancellationToken::new();

    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        tunnel_event_rx,
        Some(CommandInterfaceOptions {
            disable_socket_listener: args.disable_socket_listener,
            enable_http_listener: args.enable_http_listener,
        }),
        network_env.clone(),
        shutdown_token.child_token(),
    );

    // The user agent can be overridden by the user, but if it's not, we'll construct it
    // based on the current system information and it will be for "nym-vpnd". A number of the rpc
    // calls also provide a user-agent field so that the app can identity itself properly.
    let user_agent = args.user_agent.unwrap_or_else(util::construct_user_agent);

    let vpn_service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        shutdown_token.child_token(),
        network_env,
        user_agent,
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
