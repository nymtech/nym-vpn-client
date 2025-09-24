// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod cli;
mod command_interface;
mod config;
mod environment;
mod logging;
mod sentry;
mod service;
mod shutdown_handler;
mod user_agent;
#[cfg(windows)]
mod windows_service;

use std::path::PathBuf;

use clap::Parser;
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib::UserAgent;
use nym_vpnd_types::log_path::LogPath;

use crate::{
    cli::{CliArgs, Command},
    config::GlobalConfig,
    logging::LogFileRemoverHandle,
};
use service::{NymVpnService, NymVpnServiceParameters};

// Are we sure we need 10 worker threads?
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    match args.command.unwrap_or_default() {
        #[cfg(windows)]
        Command::InstallService => {
            println!(
                "Installing {} as a service...",
                windows_service::SERVICE_NAME
            );
            windows_service::installation::install_service()
        }
        #[cfg(windows)]
        Command::UninstallService => {
            println!("Uninstalling {} service...", windows_service::SERVICE_NAME);
            windows_service::installation::uninstall_service().await?;
            Ok(())
        }
        #[cfg(windows)]
        Command::StartService => {
            println!("Starting {} service...", windows_service::SERVICE_NAME);
            windows_service::installation::start_service()?;
            Ok(())
        }
        Command::RunAsService | Command::RunStandalone => run_vpn_service(args).await,
    }
}

async fn run_vpn_service(args: CliArgs) -> anyhow::Result<()> {
    // It would be better to call `init_sentry()` much later, as it forces a double-read
    // of the global configuration file, however there is a chicken-and-egg problem WRT
    // logging setup and reading the config file.
    let _sentry_guard = sentry::init_sentry().await;
    let sentry_enabled = _sentry_guard.is_some();

    let shutdown_token = CancellationToken::new();
    let run_as_service = args.is_run_as_service();
    let options = logging::Options {
        verbosity_level: args.verbosity_level(),
        enable_file_log: run_as_service,
        enable_stdout_log: !run_as_service,
        sentry: sentry_enabled,
    };
    let logging_setup =
        logging::setup_logging_with_file_remover(options, shutdown_token.child_token());
    let log_path = logging_setup.as_ref().map(|s| s.log_path.clone());
    let remove_log_file_signal = logging_setup
        .as_ref()
        .map(|s| s.log_file_remover_handle.clone());
    let run_parameters = RunParameters::new_with_cli_args(args, log_path, sentry_enabled);

    log_software_and_os_version();
    if sentry_enabled {
        tracing::info!("Sentry monitoring enabled");
    }

    #[cfg(windows)]
    if run_as_service {
        windows_service::start(run_parameters, remove_log_file_signal, shutdown_token).await?;
    } else {
        run_standalone(run_parameters, remove_log_file_signal, shutdown_token).await?;
    }

    #[cfg(not(windows))]
    run_standalone(run_parameters, remove_log_file_signal, shutdown_token).await?;

    let _worker_guard = if let Some(setup) = logging_setup {
        if setup.log_file_remover_join_handle.await.is_err() {
            tracing::error!("Failed to join on file logging handle");
        }
        Some(setup.worker_guard)
    } else {
        None
    };

    Ok(())
}

#[derive(Debug, Clone)]
struct RunParameters {
    log_path: Option<LogPath>,
    network: Option<String>,
    config_env_file: Option<PathBuf>,
    sentry_enabled: bool,
    stats_id_seed: Option<String>,
    user_agent: UserAgent,
}

impl RunParameters {
    fn new_with_cli_args(args: CliArgs, log_path: Option<LogPath>, sentry_enabled: bool) -> Self {
        let user_agent = args
            .user_agent
            .unwrap_or_else(user_agent::construct_user_agent);

        Self {
            log_path,
            network: args.network,
            config_env_file: args.config_env_file,
            sentry_enabled,
            stats_id_seed: args.stats_id_seed,
            user_agent,
        }
    }
}

/// Run vpn service as a standalone process.
async fn run_standalone(
    parameters: RunParameters,
    log_file_remover_handle: Option<LogFileRemoverHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    let global_config_file = setup_global_config(parameters.network).await?;

    // Migrate global configuration here, where we will have more information about the environment.

    let network_env =
        environment::setup_environment(&global_config_file, parameters.config_env_file.as_deref())
            .await?;

    let vpn_service_params = NymVpnServiceParameters {
        log_path: parameters.log_path,
        network_env: Box::new(network_env),
        sentry_enabled: parameters.sentry_enabled,
        netstats_enabled: global_config_file.collect_network_statistics,
        stats_id_seed: parameters.stats_id_seed,
        user_agent: parameters.user_agent,
    };

    let vpn_service_handle = setup_vpn_service(
        vpn_service_params,
        log_file_remover_handle,
        shutdown_token.child_token(),
    )
    .await?;

    let mut shutdown_join_set = shutdown_handler::install(shutdown_token);
    vpn_service_handle.wait_until_shutdown().await;
    shutdown_join_set.shutdown().await;

    Ok(())
}

/// Provides a way to wait for vpn service and command interface termination.
struct VpnServiceHandle {
    vpn_service_handle: JoinHandle<()>,
    command_handle: JoinHandle<()>,
    command_shutdown_token: CancellationToken,
}

impl VpnServiceHandle {
    /// Initialize with vpn service handle and command handle.
    /// `command_shutdown_token` must propagate cancellation to `command_handle`.
    pub fn new(
        vpn_service_handle: JoinHandle<()>,
        command_handle: JoinHandle<()>,
        command_shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            vpn_service_handle,
            command_handle,
            command_shutdown_token,
        }
    }

    pub async fn wait_until_shutdown(self) {
        if let Err(e) = self.vpn_service_handle.await {
            tracing::error!("Failed to join on vpn service: {}", e);
        }

        self.command_shutdown_token.cancel();

        if let Err(e) = self.command_handle.await {
            tracing::error!("Failed to join on command interface: {}", e);
        }
    }
}

async fn setup_vpn_service(
    parameters: NymVpnServiceParameters,
    log_file_remover_handle: Option<LogFileRemoverHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<VpnServiceHandle> {
    let command_shutdown_token = CancellationToken::new();
    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        tunnel_event_rx,
        command_shutdown_token.child_token(),
    )
    .await?;

    let vpn_service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        log_file_remover_handle,
        parameters,
        shutdown_token.child_token(),
    );

    Ok(VpnServiceHandle::new(
        vpn_service_handle,
        command_handle,
        command_shutdown_token,
    ))
}

async fn setup_global_config(network: Option<String>) -> anyhow::Result<GlobalConfig> {
    let mut global_config_file = GlobalConfig::read_from_default_config_dir().await?;
    if let Some(network) = network {
        global_config_file.network_name = network;
        global_config_file.write_to_default_config_dir().await?;
    }
    Ok(global_config_file)
}

fn log_software_and_os_version() {
    let build_info = nym_bin_common::bin_info_local_vergen!();
    tracing::info!(
        "{} {} ({})",
        build_info.binary_name,
        build_info.build_version,
        build_info.commit_sha
    );

    let os = nym_platform_metadata::SysInfo::new();
    tracing::info!("OS information: {}", os);
}
