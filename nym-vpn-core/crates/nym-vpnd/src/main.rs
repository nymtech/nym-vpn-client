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

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use sentry::ClientInitGuard;
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib::UserAgent;
use nym_vpnd_types::log_path::LogPath;

use crate::{
    cli::{CliArgs, Command},
    config::GlobalConfigFile,
    logging::RemoveLogFileHandle,
};
use service::{NymVpnService, NymVpnServiceParameters};

fn main() -> anyhow::Result<()> {
    let rt = runtime::new_runtime();
    rt.block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let _sentry_guard = init_sentry();
    let sentry_enabled = _sentry_guard.is_some();

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
            windows_service::installation::uninstall_service()?;
            Ok(())
        }
        #[cfg(windows)]
        Command::StartService => {
            println!("Starting {} service...", windows_service::SERVICE_NAME);
            windows_service::installation::start_service()?;
            Ok(())
        }
        Command::RunAsService | Command::RunStandalone => {
            run_vpn_service(args, sentry_enabled).await
        }
    }
}

async fn run_vpn_service(args: CliArgs, sentry_enabled: bool) -> anyhow::Result<()> {
    let shutdown_token = CancellationToken::new();
    let run_as_service = args.is_run_as_service();
    let options = logging::Options {
        verbosity_level: args.verbosity_level(),
        enable_file_log: run_as_service,
        enable_stdout_log: !run_as_service,
        sentry: sentry_enabled,
    };
    let logging_setup_with_remover =
        logging::setup_logging_with_file_remover(options, shutdown_token.child_token());
    let log_path = logging_setup_with_remover
        .as_ref()
        .map(|s| s.log_path.clone());
    let remove_log_file_handle = logging_setup_with_remover
        .as_ref()
        .map(|s| s.remove_log_file_handle.clone());
    let run_parameters = RunParameters::new_with_cli_args(args, log_path, sentry_enabled);

    if sentry_enabled {
        tracing::info!("Sentry monitoring enabled");
    }

    #[cfg(windows)]
    if run_as_service {
        windows_service::start(run_parameters, remove_log_file_handle, shutdown_token).await?;
    } else {
        run_standalone(run_parameters, remove_log_file_handle, shutdown_token).await?;
    }

    #[cfg(not(windows))]
    run_standalone(run_parameters, remove_log_file_handle, shutdown_token).await?;

    let _worker_guard = if let Some(setup) = logging_setup_with_remover {
        if setup.file_remover_handle.await.is_err() {
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

async fn run_standalone(
    parameters: RunParameters,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    let global_config_file = setup_global_config(parameters.network)?;
    let network_env =
        environment::setup_environment(&global_config_file, parameters.config_env_file.as_deref())
            .await?;

    let vpn_service_params = NymVpnServiceParameters {
        log_path: parameters.log_path,
        network_env,
        sentry_enabled: parameters.sentry_enabled,
        netstats_enabled: global_config_file.collect_network_statistics,
        stats_id_seed: parameters.stats_id_seed,
        user_agent: parameters.user_agent,
    };

    let shutdown_child_token = shutdown_token.child_token();
    let mut shutdown_join_set = shutdown_handler::install(shutdown_token);
    let vpn_service_handle = setup_vpn_service(
        vpn_service_params,
        remove_log_file_handle,
        shutdown_child_token,
    )
    .await?;

    vpn_service_handle.wait_until_shutdown().await;
    shutdown_join_set.shutdown().await;

    Ok(())
}

/// Provides a way to wait for vpn service and command interface termination.
struct VpnServiceHandle {
    vpn_service_handle: JoinHandle<()>,
    command_handle: JoinHandle<()>,
}

impl VpnServiceHandle {
    pub fn new(vpn_service_handle: JoinHandle<()>, command_handle: JoinHandle<()>) -> Self {
        Self {
            vpn_service_handle,
            command_handle,
        }
    }

    pub async fn wait_until_shutdown(self) {
        if let Err(e) = self.vpn_service_handle.await {
            tracing::error!("Failed to join on vpn service: {}", e);
        }

        if let Err(e) = self.command_handle.await {
            tracing::error!("Failed to join on command interface: {}", e);
        }
    }
}

async fn setup_vpn_service(
    parameters: NymVpnServiceParameters,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<VpnServiceHandle> {
    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);

    let (command_handle, vpn_command_rx) =
        command_interface::start_command_interface(tunnel_event_rx, shutdown_token.child_token())
            .await?;

    let vpn_service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        remove_log_file_handle,
        parameters,
        shutdown_token.child_token(),
    );

    Ok(VpnServiceHandle::new(vpn_service_handle, command_handle))
}

fn setup_global_config(network: Option<String>) -> anyhow::Result<GlobalConfigFile> {
    let mut global_config_file = GlobalConfigFile::read_from_file()?;
    if let Some(network) = network {
        global_config_file.network_name = network;
        global_config_file.write_to_file()?;
    }
    Ok(global_config_file)
}

fn init_sentry() -> Option<ClientInitGuard> {
    if !GlobalConfigFile::sentry_enabled() {
        return None;
    }

    let Some(dsn) = environment::sentry_dsn() else {
        eprintln!("failed to init sentry: SENTRY_DSN is not set");
        return None;
    };

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
}
