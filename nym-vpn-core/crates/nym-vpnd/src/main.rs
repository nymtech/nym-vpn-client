// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// mod account;
mod cli;
mod command_interface;
mod logging;
mod runtime;
mod service;
mod shutdown_handler;
mod types;
mod util;
#[cfg(windows)]
mod windows_service;

use clap::Parser;
use nym_vpn_lib::nym_config::defaults::setup_env;
use service::{default_log_dir, NymVpnService};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{
    cli::CliArgs,
    command_interface::{start_command_interface, CommandInterfaceOptions},
    logging::setup_logging,
};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/env.rs"));
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Environments {
    pub environments: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    write_default_available_environments();
    run()
}

// In the configuration directory. If the env.json file doesn't already exists, we write the
// default one that we get from the build script.
fn write_default_available_environments() {
    // TODO: handle the env variable override consistently
    let env_file = "env.json";
    let env_path = service::default_config_dir().join(env_file);
    if !env_path.exists() {
        let default_env = generated::get_environments();
        let env_json =
            serde_json::to_string_pretty(&default_env).expect("Failed to serialize default env");
        std::fs::write(env_path, env_json).expect("Failed to write env file");
    }
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

fn run_inner(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    runtime::new_runtime().block_on(run_inner_async(args))
}

async fn run_inner_async(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (state_changes_tx, state_changes_rx) = broadcast::channel(10);
    let (status_tx, status_rx) = broadcast::channel(10);
    let shutdown_token = CancellationToken::new();

    let (command_handle, vpn_command_rx) = start_command_interface(
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
