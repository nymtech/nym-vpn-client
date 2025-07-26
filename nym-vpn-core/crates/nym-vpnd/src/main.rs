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
use nym_vpnd_types::log_path::LogPath;
use sentry::ClientInitGuard;
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib::UserAgent;
use nym_vpn_network_config::Network;

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
        Command::RunAsService => {
            let shutdown_token = CancellationToken::new();
            let options = logging::Options {
                verbosity_level: args.verbosity_level(),
                enable_file_log: true,
                enable_stdout_log: false,
                sentry: sentry_enabled,
            };
            let logging_setup_with_remover =
                logging::setup_logging_with_file_remover(options, shutdown_token.child_token());

            log_sentry_enabled(sentry_enabled);

            let log_path = logging_setup_with_remover
                .as_ref()
                .map(|s| s.log_path.clone());
            let remove_log_file_handle = logging_setup_with_remover
                .as_ref()
                .map(|s| s.remove_log_file_handle.clone());

            #[cfg(windows)]
            run_windows_service(
                log_path,
                args.network,
                args.config_env_file,
                sentry_enabled,
                remove_log_file_handle,
                shutdown_token,
            )
            .await?;

            #[cfg(not(windows))]
            run_standalone(
                log_path,
                args.network,
                args.config_env_file,
                sentry_enabled,
                remove_log_file_handle,
                shutdown_token,
            )
            .await?;

            if let Some(setup) = logging_setup_with_remover {
                if setup.file_remover_handle.await.is_err() {
                    tracing::error!("Failed to join on file logging handle");
                }
            }

            Ok(())
        }
        Command::RunStandalone => {
            let shutdown_token = CancellationToken::new();
            let options = logging::Options {
                verbosity_level: args.verbosity_level(),
                enable_file_log: false,
                enable_stdout_log: true,
                sentry: sentry_enabled,
            };
            let logging_setup_with_remover =
                logging::setup_logging_with_file_remover(options, shutdown_token.child_token());

            log_sentry_enabled(sentry_enabled);

            let log_path = logging_setup_with_remover
                .as_ref()
                .map(|s| s.log_path.clone());
            let remove_log_file_handle = logging_setup_with_remover
                .as_ref()
                .map(|s| s.remove_log_file_handle.clone());

            run_standalone(
                log_path,
                args.network,
                args.config_env_file,
                sentry_enabled,
                remove_log_file_handle,
                shutdown_token,
            )
            .await?;

            if let Some(setup) = logging_setup_with_remover {
                if setup.file_remover_handle.await.is_err() {
                    tracing::error!("Failed to join on file logging handle");
                }
            }

            Ok(())
        }
    }
}

#[cfg(windows)]
async fn run_windows_service(
    log_path: Option<LogPath>,
    network: Option<String>,
    config_env_file: Option<PathBuf>,
    sentry_enabled: bool,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    windows_service::start(
        log_path,
        network,
        config_env_file,
        sentry_enabled,
        remove_log_file_handle,
        shutdown_token,
        tokio::runtime::Handle::current(),
    )
    .await
}

async fn run_standalone(
    log_path: Option<LogPath>,
    network: Option<String>,
    config_env_file: Option<PathBuf>,
    sentry_enabled: bool,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    let global_config_file = setup_global_config(network)?;
    let network_env =
        environment::setup_environment(&global_config_file, config_env_file.as_deref()).await?;

    let parameters = VpnServiceSetupParameters {
        log_path,
        network_env,
        sentry_enabled,
        netstats_enabled: global_config_file.collect_network_statistics,
        stats_id_seed: None,
        user_agent: None,
    };

    let shutdown_child_token = shutdown_token.child_token();
    let mut shutdown_join_set = shutdown_handler::install(shutdown_token);
    let vpn_service_runtime =
        setup_vpn_service(parameters, remove_log_file_handle, shutdown_child_token).await?;

    vpn_service_runtime.wait_until_shutdown().await;
    shutdown_join_set.shutdown().await;

    Ok(())
}

struct VpnServiceSetupParameters {
    pub log_path: Option<LogPath>,
    pub network_env: Network,
    pub sentry_enabled: bool,
    pub netstats_enabled: bool,
    pub stats_id_seed: Option<String>,
    pub user_agent: Option<UserAgent>,
}

struct VpnServiceRuntime {
    vpn_service_handle: JoinHandle<()>,
    command_handle: JoinHandle<()>,
}

impl VpnServiceRuntime {
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

#[derive(thiserror::Error, Debug)]
enum SetupServiceError {
    #[error("failed to start command")]
    StartCommandInterface(#[source] std::io::Error),
}

async fn setup_vpn_service(
    parameters: VpnServiceSetupParameters,
    remove_log_file_handle: Option<RemoveLogFileHandle>,
    shutdown_token: CancellationToken,
) -> Result<VpnServiceRuntime, SetupServiceError> {
    let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);

    let (command_handle, vpn_command_rx) =
        command_interface::start_command_interface(tunnel_event_rx, shutdown_token.child_token())
            .await
            .map_err(SetupServiceError::StartCommandInterface)?;

    // The user agent can be overridden by the user, but if it's not, we'll construct it
    // based on the current system information and it will be for "nym-vpnd". A number of the rpc
    // calls also provide a user-agent field so that the app can identity itself properly.
    let user_agent = parameters
        .user_agent
        .unwrap_or_else(user_agent::construct_user_agent);

    let parameters = NymVpnServiceParameters {
        log_path: parameters.log_path,
        network_env: parameters.network_env,
        stats_id_seed: parameters.stats_id_seed,
        sentry_enabled: parameters.sentry_enabled,
        netstats_enabled: parameters.netstats_enabled,
        user_agent,
    };

    let vpn_service_handle = NymVpnService::spawn(
        vpn_command_rx,
        tunnel_event_tx,
        remove_log_file_handle,
        parameters,
        shutdown_token.child_token(),
    );

    Ok(VpnServiceRuntime::new(vpn_service_handle, command_handle))
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

fn log_sentry_enabled(sentry_enabled: bool) {
    if sentry_enabled {
        tracing::info!("Sentry monitoring enabled");
    }
}
