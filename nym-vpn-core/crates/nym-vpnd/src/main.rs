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

static ENVIRONMENTS: std::sync::OnceLock<Environments> = std::sync::OnceLock::new();

static NYM_NETWORK: std::sync::OnceLock<nym_vpn_lib::nym_config::defaults::NymNetworkDetails> =
    std::sync::OnceLock::new();

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Environments {
    pub environments: Vec<String>,
}

pub struct Discovery {
    pub network: nym_vpn_lib::nym_config::defaults::NymNetworkDetails,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    write_default_available_environments();
    write_default_nym_network();
    fetch_latest_available_enviroments();
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

fn write_default_nym_network() {
    // TODO: handle the env variable override consistently
    let env_file = "discovery.json";
    let env_path = service::default_config_dir().join(env_file);
    println!("env_path: {:?}", env_path);
    let network = nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_mainnet();
    if !env_path.exists() {
        // let default_env = generated::get_environments();
        let network_json =
            serde_json::to_string_pretty(&network).expect("Failed to serialize network env");
        std::fs::write(env_path, network_json).expect("Failed to write env file");
    }
}

fn read_discovery_file() -> nym_vpn_lib::nym_config::defaults::NymNetworkDetails {
    let discovery_file = "discovery.json";
    let discovery_path = service::default_config_dir().join(discovery_file);
    tracing::info!("discovery_path: {:?}", discovery_path);

    // Read json file from discovery_path
    let file_str = std::fs::read_to_string(discovery_path).expect("Failed to read discovery file");
    let network: nym_vpn_lib::nym_config::defaults::NymNetworkDetails =
        serde_json::from_str(&file_str).expect("Failed to parse network file");
    dbg!(&network);
    network
}

// Fetch the latest available environments from the server
fn fetch_latest_available_enviroments() {
    //let latest = .. fetch remotely ..
    let latest = generated::get_environments();

    // Initialize the global variable with the latest available environments
    ENVIRONMENTS.get_or_init(|| latest);
}

#[cfg(unix)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    setup_logging(args.command.run_as_service);
    if let Some(ref env) = args.config_env_file {
        setup_env(Some(env));
    } else {
        // Read the local discovery file, that we synced on last time
        let network = read_discovery_file();
    }

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
