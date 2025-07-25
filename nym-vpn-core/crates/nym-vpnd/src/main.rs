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
mod user_agent;
#[cfg(windows)]
mod windows_service;

#[cfg(windows)]
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use sentry::ClientInitGuard;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib::{SysInfo, UserAgent};
use nym_vpn_network_config::Network;

use crate::{
    cli::{CliArgs, Command},
    config::GlobalConfigFile,
};
use logging::{LogFileRemover, LoggingSetup};
use service::{NymVpnService, NymVpnServiceParameters};

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let _sentry_guard = init_sentry();
    let sentry_enabled = _sentry_guard.is_some();

    match args.command.unwrap_or_default() {
        #[cfg(windows)]
        Command::Install => {
            println!(
                "Processing request to install {} as a service...",
                windows_service::SERVICE_NAME
            );
            windows_service::installation::install_service()?;
            Ok(())
        }
        #[cfg(windows)]
        Command::Uninstall => {
            println!(
                "Processing request to uninstall {} as a service...",
                windows_service::SERVICE_NAME
            );
            windows_service::installation::uninstall_service()?;
            Ok(())
        }
        #[cfg(windows)]
        Command::Start => {
            println!(
                "Processing request to start service {}...",
                windows_service::SERVICE_NAME
            );
            windows_service::installation::start_service()?;
            Ok(())
        }
        Command::RunAsService => {
            let options = logging::Options {
                verbosity_level: args.verbosity_level(),
                enable_file_log: true,
                enable_stdout_log: false,
                sentry: sentry_enabled,
            };

            #[cfg(windows)]
            {
                run_windows_service(args.network, args.config_env_file, options)
            }

            #[cfg(not(windows))]
            {
                run_standalone(args.network, args.config_env_file, options)
            }
        }
        Command::RunStandalone => {
            let options = logging::Options {
                verbosity_level: args.verbosity_level(),
                enable_file_log: false,
                enable_stdout_log: true,
                sentry: sentry_enabled,
            };
            run_standalone(args.network, args.config_env_file, options)
        }
    }
}

#[cfg(windows)]
fn run_windows_service(
    network: Option<String>,
    config_env_file: Option<PathBuf>,
    options: logging::Options,
) -> anyhow::Result<()> {
    let sentry_enabled = options.sentry;
    let logging_setup = logging::setup_logging(options);
    if sentry_enabled {
        tracing::info!("Sentry monitoring enabled");
    }

    let os = SysInfo::new();
    os.display(true);

    windows_service::start(
        windows_service::ServiceNetworkConfig {
            network,
            config_env_file,
        },
        logging_setup,
        sentry_enabled,
    )?;

    Ok(())
}

fn run_standalone(
    network: Option<String>,
    config_env_file: Option<PathBuf>,
    options: logging::Options,
) -> anyhow::Result<()> {
    let sentry_enabled = options.sentry;
    let logging_setup = logging::setup_logging(options);
    let global_config_file = setup_global_config(network)?;

    if sentry_enabled {
        tracing::info!("Sentry monitoring enabled");
    }

    let os = SysInfo::new();
    os.display(true);

    runtime::new_runtime().block_on(async {
        let network_env =
            environment::setup_environment(&global_config_file, config_env_file.as_deref()).await?;
        let shutdown_token = CancellationToken::new();

        let standalone_config = RunStandaloneParameters {
            network_env,
            sentry_enabled,
            netstats_enabled: global_config_file.collect_network_statistics,
            stats_id_seed: None,
            user_agent: None,
        };

        run_standalone_async(standalone_config, logging_setup, shutdown_token).await
    })
}

struct RunStandaloneParameters {
    pub network_env: Network,
    pub sentry_enabled: bool,
    pub netstats_enabled: bool,
    pub stats_id_seed: Option<String>,
    pub user_agent: Option<UserAgent>,
}

async fn run_standalone_async(
    config: RunStandaloneParameters,
    logging_setup: Option<LoggingSetup>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    let log_path = logging_setup
        .as_ref()
        .map(|logging_setup| logging_setup.log_path().clone());
    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let (file_logging_event_tx, file_logging_event_rx) = mpsc::unbounded_channel();

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

    let (command_handle, vpn_command_rx) =
        command_interface::start_command_interface(tunnel_event_rx, shutdown_token.child_token())
            .await?;

    // The user agent can be overridden by the user, but if it's not, we'll construct it
    // based on the current system information and it will be for "nym-vpnd". A number of the rpc
    // calls also provide a user-agent field so that the app can identity itself properly.
    let user_agent = config
        .user_agent
        .unwrap_or_else(user_agent::construct_user_agent);

    let parameters = NymVpnServiceParameters {
        network_env: config.network_env,
        stats_id_seed: config.stats_id_seed,
        log_path,
        sentry_enabled: config.sentry_enabled,
        netstats_enabled: config.netstats_enabled,
        user_agent,
    };

    let vpn_service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        file_logging_event_tx,
        parameters,
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

    if let Some(file_logging_handle) = file_logging_handle {
        let _worker_guard = file_logging_handle
            .await
            .inspect_err(|e| tracing::error!("Failed to join on file logging: {}", e))
            .ok();
    }

    Ok(())
}

fn init_sentry() -> Option<ClientInitGuard> {
    if !GlobalConfigFile::sentry_enabled() {
        return None;
    }
    if let Some(dsn) = environment::sentry_dsn() {
        println!("Sentry monitoring enabled");
        let guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                send_default_pii: false,
                sample_rate: 1.0,
                traces_sample_rate: 1.0,
                enable_logs: true,
                shutdown_timeout: Duration::from_secs(2),
                ..Default::default()
            },
        ));
        Some(guard)
    } else {
        eprintln!("failed to init sentry: SENTRY_DSN is not set");
        None
    }
}

fn setup_global_config(network: Option<String>) -> anyhow::Result<GlobalConfigFile> {
    let mut global_config_file = GlobalConfigFile::read_from_file()?;
    if let Some(network) = network {
        global_config_file.network_name = network;
        global_config_file.write_to_file()?;
    }
    Ok(global_config_file)
}
