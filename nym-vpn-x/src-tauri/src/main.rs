// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::cli::{db_command, Commands};
use crate::window::AppWindow;
use crate::{
    cli::{print_build_info, Cli},
    db::Db,
    fs::{config::AppConfig, storage::AppStorage},
    grpc::client::GrpcClient,
};

use anyhow::{anyhow, Result};
use clap::Parser;
use commands::daemon as cmd_daemon;
use commands::db as cmd_db;
use commands::fs as cmd_fs;
use commands::log as cmd_log;
use commands::window as cmd_window;
use commands::*;
use nym_config::defaults;
use states::app::AppState;
use tauri::api::path::config_dir;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, trace, warn};

mod cli;
mod commands;
mod country;
mod db;
mod envi;
mod error;
mod events;
mod fs;
mod grpc;
mod http;
mod log;
mod states;
mod system_tray;
mod vpn_status;
mod window;

pub const APP_DIR: &str = "nymvpn-x";
pub const MAIN_WINDOW_LABEL: &str = "main";
const APP_CONFIG_FILE: &str = "config.toml";
const ENV_APP_NOSPLASH: &str = "APP_NOSPLASH";
const VPND_RETRY_INTERVAL: Duration = Duration::from_secs(2);
const SYSTRAY_ID: &str = "main";

#[tokio::main]
async fn main() -> Result<()> {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    dotenvy::dotenv().ok();
    // parse the command line arguments
    let cli = Cli::parse();

    let _guard = log::setup_tracing(cli.log_file).await?;
    trace!("cli args: {:#?}", cli);

    #[cfg(windows)]
    if cli.console {
        use windows::Win32::System::Console::AllocConsole;
        let _ = unsafe { AllocConsole() };
    }

    let context = tauri::generate_context!();

    if cli.build_info {
        print_build_info(context.package_info());
        return Ok(());
    }

    info!("Creating k/v embedded db");
    let db = Db::new()?;

    if let Some(Commands::Db { command: Some(cmd) }) = &cli.command {
        return db_command(&db, cmd);
    }

    let app_config_store = {
        let mut app_config_path =
            config_dir().ok_or(anyhow!("Failed to retrieve config directory path"))?;
        app_config_path.push(APP_DIR);
        AppStorage::<AppConfig>::new(app_config_path, APP_CONFIG_FILE, None)
            .await
            .inspect_err(|e| error!("Failed to init app config store: {e}"))?
    };
    debug!(
        "app_config_store: {}",
        &app_config_store.full_path.display()
    );

    let app_config = match app_config_store.read().await {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("failed to read app config: {e}, falling back to default (empty) config");
            debug!("clearing the config file");
            app_config_store
                .clear()
                .await
                .inspect_err(|e| error!("failed to clear the config file: {e}"))
                .ok();
            AppConfig::default()
        }
    };
    debug!("app_config: {app_config:?}");

    info!("network environment: mainnet");
    defaults::setup_env::<PathBuf>(None);

    let app_state = AppState::try_from((&db, &app_config, &cli)).map_err(|e| {
        error!("failed to create app state from saved app data and config: {e}");
        e
    })?;

    let grpc = GrpcClient::new(&app_config, &cli);

    info!("Starting tauri app");
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(app_state)))
        .manage(Arc::new(app_config))
        .manage(Arc::new(cli.clone()))
        .manage(db.clone())
        .manage(Arc::new(grpc.clone()))
        .setup(move |app| {
            info!("app setup");

            let app_win = AppWindow::new(&app.handle(), MAIN_WINDOW_LABEL)?;
            app_win.restore_size(&db)?;
            app_win.restore_position(&db)?;

            let env_nosplash = envi::is_truthy(ENV_APP_NOSPLASH);
            trace!("env APP_NOSPLASH: {}", env_nosplash);

            // if splash-screen is disabled, remove it and show
            // the main window without waiting for frontend signal
            if cli.nosplash || env_nosplash {
                debug!("splash screen disabled, showing main window");
                app_win.no_splash();
            }

            debug!("building system tray");
            let handle = app.handle();
            system_tray::systray(SYSTRAY_ID)
                .on_event(move |event| {
                    let handle = handle.clone();
                    tokio::spawn(async move {
                        system_tray::on_tray_event(&handle, event).await;
                    });
                })
                .build(app)
                .inspect_err(|e| error!("error while building system tray: {e}"))?;

            let handle = app.handle();
            let c_grpc = grpc.clone();
            tokio::spawn(async move {
                info!("starting vpnd health spy");
                loop {
                    c_grpc.watch(&handle).await.ok();
                    sleep(VPND_RETRY_INTERVAL).await;
                    debug!("vpnd health spy retry");
                }
            });

            let handle = app.handle();
            let c_grpc = grpc.clone();
            tokio::spawn(async move {
                info!("starting vpn status spy");
                loop {
                    if c_grpc.refresh_vpn_status(&handle).await.is_ok() {
                        c_grpc.watch_vpn_state(&handle).await.ok();
                    }
                    sleep(VPND_RETRY_INTERVAL).await;
                    debug!("vpn status spy retry");
                }
            });

            let handle = app.handle();
            let c_grpc = grpc.clone();
            tokio::spawn(async move {
                info!("starting vpn connection updates spy");
                loop {
                    c_grpc.watch_vpn_connection_updates(&handle).await.ok();
                    sleep(VPND_RETRY_INTERVAL).await;
                    debug!("vpn connection updates spy retry");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connection::set_vpn_mode,
            connection::get_connection_state,
            connection::connect,
            connection::disconnect,
            connection::get_connection_start_time,
            cmd_db::db_set,
            cmd_db::db_get,
            cmd_db::db_flush,
            node_location::get_node_location,
            node_location::set_node_location,
            node_location::get_countries,
            cmd_window::show_main_window,
            commands::cli::cli_args,
            cmd_log::log_js,
            credential::add_credential,
            cmd_daemon::daemon_status,
            cmd_fs::log_dir,
        ])
        // keep the app running in the background on window close request
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                let win = event.window();
                if win.label() == MAIN_WINDOW_LABEL {
                    win.hide()
                        .inspect_err(|e| error!("failed to hide main window: {e}"))
                        .ok();
                    api.prevent_close();
                }
            }
        })
        .run(context)
        .expect("error while running tauri application");

    Ok(())
}
