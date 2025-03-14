// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use crate::cli::{db_command, Commands};
use crate::startup_error::ErrorKey;
use crate::window::AppWindow;
use crate::{
    cli::{print_build_info, Cli},
    db::Db,
    fs::{config::AppConfig, storage::AppStorage},
    grpc::client::GrpcClient,
};

use crate::fs::path::APP_CONFIG_DIR;
use anyhow::{anyhow, Result};
use clap::Parser;
use commands::daemon as cmd_daemon;
use commands::db as cmd_db;
use commands::dev as cmd_dev;
use commands::env as cmd_env;
use commands::fs as cmd_fs;
use commands::gateway as cmd_gw;
use commands::log as cmd_log;
use commands::window as cmd_window;
use commands::*;
use state::app::AppState;
use tauri::Manager;
use tauri_plugin_window_state::StateFlags;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, trace, warn};

mod cli;
mod commands;
mod country;
mod db;
mod env;
mod error;
mod events;
mod fs;
mod grpc;
mod log;
mod misc;
mod startup_error;
mod state;
mod tray;
mod window;

pub const APP_NAME: &str = "NymVPN";
pub const APP_DIR: &str = "nym-vpn-app";
pub const MAIN_WINDOW_LABEL: &str = "main";
pub const ERROR_WINDOW_LABEL: &str = "error";
const APP_CONFIG_FILE: &str = "config.toml";
const ENV_APP_NOSPLASH: &str = "APP_NOSPLASH";
const VPND_RETRY_INTERVAL: Duration = Duration::from_secs(2);

// build time pkg data
build_info::build_info!(fn build_info);

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    #[cfg(all(not(debug_assertions), windows))]
    cli::attach_console();

    // parse the command line arguments
    let cli = Cli::parse();
    let _guard = log::setup_tracing(&cli).await?;
    trace!("cli args: {:#?}", cli);

    #[cfg(unix)]
    misc::nvidia_check();

    #[cfg(windows)]
    if cli.console {
        use windows::Win32::System::Console::AllocConsole;
        let _ = unsafe { AllocConsole() };
    }

    let context = tauri::generate_context!();
    let pkg_info = context.package_info();

    if cli.build_info {
        print_build_info(pkg_info);
        return Ok(());
    }

    if let Some(Commands::Db { command: Some(cmd) }) = &cli.command {
        return db_command(cmd);
    }

    info!("app version: {}", pkg_info.version);
    info!("Starting tauri app");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION)
                .with_denylist(&[ERROR_WINDOW_LABEL])
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            info!("an app instance is already running, focusing main window");
            window::focus_main_window(app)
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .setup(move |app| {
            info!("app setup");

            app.manage(cli.clone());

            info!("Creating k/v embedded db");
            let Ok(db) = Db::new().inspect_err(|e| {
                startup_error::set_error(ErrorKey::from(e), Some(&e.to_string()));
            }) else {
                startup_error::create_window(app.handle())?;
                return Ok(());
            };
            app.manage(db.clone());

            let app_window = AppWindow::create_main_window(app.handle())?;
            app_window.set_bg_color(&db).ok();
            app_window.set_max_size().ok();

            let app_config_store = {
                let path = APP_CONFIG_DIR
                    .clone()
                    .ok_or(anyhow!("failed to get app config dir"))?;
                AppStorage::<AppConfig>::new(path, APP_CONFIG_FILE, None)
                    .inspect_err(|e| error!("Failed to init app config store: {e}"))?
            };
            debug!(
                "app_config_store: {}",
                &app_config_store.full_path.display()
            );

            let app_config = match app_config_store.read() {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("failed to read app config: {e}, falling back to default (empty) config");
                    debug!("clearing the config file");
                    app_config_store
                        .clear()
                        .inspect_err(|e| error!("failed to clear the config file: {e}"))
                        .ok();
                    AppConfig::default()
                }
            };
            debug!("app_config: {app_config:?}");

            let app_state = AppState::new(&db, &app_config, &cli);
            app.manage(Mutex::new(app_state));

            let pkg_info = app.package_info().clone();
            let grpc = GrpcClient::new(&app_config, &cli, &pkg_info);
            let mut c_grpc = grpc.clone();
            let handle = app.handle().clone();
            tokio::spawn(async move {
                c_grpc.update_vpnd_info(&handle).await.ok();
            });

            app.manage(app_config);
            app.manage(grpc.clone());

            // if splash-screen is disabled, remove it and show
            // the main window without waiting for frontend signal
            if cli.nosplash || env::is_truthy(ENV_APP_NOSPLASH) {
                debug!("splash screen disabled, showing main window");
                app_window.no_splash();
            }

            tray::setup(app.handle())?;

            let handle = app.handle().clone();
            let mut c_grpc = grpc.clone();
            tokio::spawn(async move {
                info!("starting vpnd health spy");
                loop {
                    c_grpc.watch(&handle).await.ok();
                    sleep(VPND_RETRY_INTERVAL).await;
                    trace!("vpnd health spy retry");
                }
            });

            let handle = app.handle().clone();
            let c_grpc = grpc.clone();
            tokio::spawn(async move {
                info!("starting vpn tunnel spy");
                loop {
                    if c_grpc.tunnel_state(&handle).await.is_ok() {
                        c_grpc.watch_tunnel_events(&handle).await.ok();
                    }
                    sleep(VPND_RETRY_INTERVAL).await;
                    trace!("vpn tunnel spy retry");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tunnel::set_vpn_mode,
            tunnel::get_tunnel_state,
            tunnel::connect,
            tunnel::disconnect,
            cmd_dev::get_credentials_mode,
            cmd_dev::set_credentials_mode,
            cmd_db::db_set,
            cmd_db::db_get,
            cmd_db::db_del,
            cmd_db::db_flush,
            cmd_gw::get_gateways,
            cmd_window::show_main_window,
            cmd_window::set_background_color,
            commands::cli::cli_args,
            cmd_log::log_js,
            account::add_account,
            account::forget_account,
            account::is_account_stored,
            account::get_account_id,
            account::get_device_id,
            account::account_links,
            cmd_daemon::daemon_status,
            cmd_daemon::set_network,
            cmd_daemon::system_messages,
            cmd_daemon::feature_flags,
            cmd_daemon::network_compat,
            cmd_fs::log_dir,
            startup::startup_error,
            cmd_env::env,
        ])
        // keep the app running in the background on window close request
        .on_window_event(|win, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
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
