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

use clap::Parser;
use logging::{LogFileRemover, LoggingSetup};
use nym_vpn_network_config::Network;
use service::NymVpnService;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{cli::CliArgs, config::GlobalConfigFile};

fn main() -> anyhow::Result<()> {
    run()
}

#[cfg(unix)]
fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let options = logging::Options {
        verbosity_level: args.verbosity_level(),
        enable_file_log: args.command.run_as_service,
        enable_stdout_log: true,
    };
    let logging_setup = logging::setup_logging(options);

    let global_config_file = setup_global_config(args.network.as_deref())?;
    let network_env =
        environment::setup_environment(&global_config_file, args.config_env_file.as_deref())?;

    run_inner(args, network_env, logging_setup)
}

#[cfg(windows)]
fn run() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    if args.command.install {
        println!(
            "Processing request to install {} as a service...",
            service::windows_service::SERVICE_NAME
        );
        service::windows_service::install_service()?;
        Ok(())
    } else if args.command.uninstall {
        println!(
            "Processing request to uninstall {} as a service...",
            service::windows_service::SERVICE_NAME
        );
        service::windows_service::uninstall_service()?;
        Ok(())
    } else if args.command.start {
        println!(
            "Processing request to start service {}...",
            service::windows_service::SERVICE_NAME
        );
        service::windows_service::start_service()?;
        Ok(())
    } else if args.command.run_as_service {
        // TODO: enable this through setting or flag
        // println!("Configuring logging source...");
        // eventlog::init(SERVICE_DISPLAY_NAME, log::Level::Info).unwrap();
        let logging_setup = logging::setup_logging(logging::Options {
            verbosity_level: args.verbosity_level(),
            enable_file_log: true,
            enable_stdout_log: false,
        });
        service::windows_service::start(
            service::windows_service::ServiceNetworkConfig {
                network: args.network.to_owned(),
                config_env_file: args.config_env_file.to_owned(),
            },
            logging_setup,
        )?;
        Ok(())
    } else {
        let options = logging::Options {
            verbosity_level: args.verbosity_level(),
            enable_file_log: false,
            enable_stdout_log: true,
        };
        let logging_setup = logging::setup_logging(options);

        let global_config_file = setup_global_config(args.network.as_deref())?;
        let network_env =
            environment::setup_environment(&global_config_file, args.config_env_file.as_deref())?;

        run_inner(args, network_env, logging_setup)
    }
}

fn setup_global_config(network: Option<&str>) -> anyhow::Result<GlobalConfigFile> {
    let mut global_config_file = GlobalConfigFile::read_from_file()?;
    if let Some(network) = network {
        global_config_file.network_name = network.to_owned();
        global_config_file.write_to_file()?;
    }
    Ok(global_config_file)
}

fn run_inner(
    args: CliArgs,
    network_env: Network,
    logging_setup: Option<LoggingSetup>,
) -> anyhow::Result<()> {
    runtime::new_runtime().block_on(run_inner_async(args, network_env, logging_setup))
}

async fn run_inner_async(
    args: CliArgs,
    network_env: Network,
    logging_setup: Option<LoggingSetup>,
) -> anyhow::Result<()> {
    network_env.check_consistency().await?;

    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let (file_logging_event_tx, file_logging_event_rx) = mpsc::channel(1);
    let shutdown_token = CancellationToken::new();

    let file_logging_handle = logging_setup.map(|logging_setup| {
        tokio::spawn(
            LogFileRemover::new(
                file_logging_event_rx,
                logging_setup,
                shutdown_token.child_token(),
            )
            .run(),
        )
    });

    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        tunnel_event_rx,
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
        file_logging_event_tx,
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

    if let Some(file_logging_handle) = file_logging_handle {
        if let Err(e) = file_logging_handle.await {
            tracing::error!("Failed to join on file logging: {}", e);
        }
    }

    shutdown_join_set.shutdown().await;

    Ok(())
}
