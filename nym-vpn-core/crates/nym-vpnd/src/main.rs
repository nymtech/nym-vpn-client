// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod cli;
mod command_interface;
mod environment;
mod shutdown_handler;
#[cfg(windows)]
mod windows_service;

use anyhow::Context;
use clap::Parser;
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_platform_metadata::new_user_agent;
use nym_vpn_lib::{
    UserAgent,
    config::GlobalConfig,
    logging::LogFileRemoverHandle,
    paths::Paths,
    service::{NymVpnService, NymVpnServiceParameters, ServiceConfigStorageType},
};
#[cfg(target_os = "windows")]
use nym_vpn_lib::{install_split_tunnel_driver_service, uninstall_split_tunnel_driver_service};
use nym_vpn_network_config::NetworkCache;
#[cfg(target_os = "windows")]
use windows_service::{
    SERVICE_NAME,
    installation::{install_service, start_service, uninstall_service},
};

use crate::cli::{CliArgs, Command, RunArgs};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();

    match cli_args.command.unwrap_or_default() {
        #[cfg(windows)]
        Command::InstallService => {
            install_service().context("Failed to install Windows service")?;
            install_split_tunnel_driver_service()
                .context("Failed to install split tunnel driver service")?;
            Ok(())
        }
        #[cfg(windows)]
        Command::UninstallService => {
            uninstall_split_tunnel_driver_service()
                .context("Failed to uninstall split tunnel driver service")
                .or(uninstall_service()
                    .await
                    .context("Failed to uninstall Windows service"))?;
            Ok(())
        }
        #[cfg(windows)]
        Command::StartService => {
            println!("Starting {SERVICE_NAME} service...");
            start_service()
                .await
                .context("Failed to start Windows service")?;
            Ok(())
        }
        Command::RunStandalone => run_vpn_service(cli_args, RunArgs::default()).await,
        Command::RunAsService(run_args) | Command::RunWithArgs(run_args) => {
            run_vpn_service(cli_args, run_args).await
        }
    }
}

async fn run_vpn_service(cli_args: CliArgs, run_args: RunArgs) -> anyhow::Result<()> {
    let mut paths = Paths::new();

    let global_config = GlobalConfig::read_from_config_dir(&paths.config_dir)
        .await
        .unwrap_or_default();

    let _sentry_guard = if global_config.sentry_monitoring {
        nym_vpn_lib::sentry::init_sentry()
    } else {
        None
    };
    let sentry_enabled = _sentry_guard.is_some();

    let shutdown_token = CancellationToken::new();
    let run_as_service = cli_args.is_run_as_service();
    let options = nym_vpn_lib::logging::Options {
        verbosity_level: cli_args.verbosity_level(),
        enable_stdout_log: !run_as_service,
        enable_json_log: cli_args.json_output,
        log_dir: if run_as_service {
            Some(paths.log_dir.clone())
        } else {
            None
        },
        sentry: sentry_enabled,
    };
    let logging_setup = nym_vpn_lib::logging::setup_logging_with_file_remover(
        options,
        shutdown_token.child_token(),
    );
    paths.log_path = logging_setup.as_ref().map(|s| s.log_path.clone());
    let remove_log_file_signal = logging_setup
        .as_ref()
        .map(|s| s.log_file_remover_handle.clone());
    let run_parameters = RunParameters {
        paths,
        user_agent: cli_args.user_agent.unwrap_or_else(|| new_user_agent!()),
        sentry_enabled,
        disable_client_verification: run_args.disable_client_verification,
    };

    nym_vpn_lib::log_software_and_os_version();
    if sentry_enabled {
        tracing::info!("Sentry monitoring enabled");
    }

    #[cfg(windows)]
    if run_as_service {
        windows_service::start(
            run_parameters,
            global_config,
            remove_log_file_signal,
            shutdown_token,
        )
        .await?;
    } else {
        run_standalone(
            run_parameters,
            global_config,
            remove_log_file_signal,
            shutdown_token,
        )
        .await?;
    }

    #[cfg(not(windows))]
    run_standalone(
        run_parameters,
        global_config,
        remove_log_file_signal,
        shutdown_token,
    )
    .await?;

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
    paths: Paths,
    sentry_enabled: bool,
    user_agent: UserAgent,
    disable_client_verification: bool,
}

/// Run vpn service as a standalone process.
async fn run_standalone(
    parameters: RunParameters,
    global_config_file: GlobalConfig,
    log_file_remover_handle: Option<LogFileRemoverHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    parameters
        .paths
        .create_directories()
        .await
        .context("failed to create base directories")?;

    let network_cache = environment::setup_environment(
        &parameters.paths.config_dir,
        &global_config_file.network_name,
        parameters.user_agent.clone(),
    )
    .await?;

    let mut shutdown_join_set = shutdown_handler::install(shutdown_token.clone());

    let join_handle = spawn_vpn_service(
        parameters,
        network_cache,
        log_file_remover_handle,
        shutdown_token,
    )
    .await?;

    if let Err(err) = join_handle.await {
        tracing::error!("Failed to join on vpn service: {err}");
    }

    shutdown_join_set.shutdown().await;

    Ok(())
}

async fn spawn_vpn_service(
    parameters: RunParameters,
    network_cache: NetworkCache,
    log_file_remover_handle: Option<LogFileRemoverHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<JoinHandle<()>> {
    let vpn_service_params = NymVpnServiceParameters {
        paths: parameters.paths,
        network_cache,
        sentry_enabled: parameters.sentry_enabled,
        user_agent: parameters.user_agent,
        service_storage_type: ServiceConfigStorageType::Persistent,
    };

    let command_shutdown_token = CancellationToken::new();
    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
    let (command_handle, vpn_command_rx) = command_interface::start_command_interface(
        parameters.disable_client_verification,
        tunnel_event_rx,
        command_shutdown_token.child_token(),
    )
    .await
    .with_context(|| "failed to start command interface")?;

    let vpn_service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        log_file_remover_handle,
        vpn_service_params,
        shutdown_token.child_token(),
    );

    let join_handle = tokio::spawn(async move {
        if let Err(e) = vpn_service_handle.await {
            tracing::error!("Failed to join on vpn service: {}", e);
        }

        tracing::info!("Cancelling command token");
        command_shutdown_token.cancel();
        if let Err(e) = command_handle.await {
            tracing::error!("Failed to join on command interface: {}", e);
        }
    });

    Ok(join_handle)
}
