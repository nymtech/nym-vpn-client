// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use crate::cli::{Commands, db_command};
use crate::favorites::FavoritesState;
use crate::fs::path::{APP_CONFIG_DIR, APP_DATA_DIR};
use crate::startup_error::{ErrorKey, StartupError};
use crate::tray::TrayManager;
#[cfg(windows)]
use crate::updater::PendingUpdate;
use crate::window::AppWindow;
use crate::{
    cli::Cli,
    db::Db,
    fs::{app::AppFs, config::AppConfig},
    vpnd::client::VpndClient,
    vpnd::error::VpndError,
};

use anyhow::{Result, anyhow};
use clap::Parser;
use commands::daemon as cmd_daemon;
use commands::db as cmd_db;
use commands::diagnostic as cmd_diag;
use commands::favorites as cmd_favorites;
use commands::fs as cmd_fs;
use commands::gateway as cmd_gw;
use commands::log as cmd_log;
use commands::sentry as cmd_sentry;
use commands::socks5 as cmd_socks5;
use commands::sys as cmd_sys;
use commands::tray as cmd_tray;
#[cfg(windows)]
use commands::updater as cmd_updater;
use commands::window as cmd_window;
use commands::*;
use nym_favorites::FavoritesManager;
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
mod favorites;
mod fs;
#[cfg(windows)]
mod icon_extractor;
#[cfg(target_os = "linux")]
mod linux_update_watcher;
mod log;
mod sentry;
mod startup_error;
mod state;
mod sys;
mod tray;
#[cfg(windows)]
mod updater;
mod vpnd;
mod vpnd_check;
mod window;

pub const APP_NAME: &str = "NymVPN";
pub const APP_DIR: &str = "nym-vpn-app";
pub const MAIN_WINDOW_LABEL: &str = "main";
pub const ERROR_WINDOW_LABEL: &str = "error";
const APP_CONFIG_FILE: &str = "config.toml";
const ENV_APP_NOSPLASH: &str = "APP_NOSPLASH";
const VPND_RETRY_INTERVAL: Duration = Duration::from_secs(2);
const DEFAULT_SENTRY_ENABLED: bool = false;
const DEFAULT_NETSTATS_ENABLED: bool = true;
const DEFAULT_QUIC: bool = false;
const DEFAULT_DEBUG_LOGGING: bool = true;

// build time pkg data
build_info::build_info!(fn build_info);

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    #[cfg(all(not(debug_assertions), windows))]
    cli::attach_console();

    // parse the command line arguments
    let cli = Cli::parse();
    if cli.clean_local_files {
        fs::util::clean_local_files();
        return Ok(());
    }
    let app_config = AppConfig::read().ok();
    let sentry_enabled = app_config
        .as_ref()
        .map(|cfg| cfg.sentry_monitoring)
        .unwrap_or(DEFAULT_SENTRY_ENABLED);
    let debug_logging = app_config
        .as_ref()
        .map(|cfg| cfg.debug_logging)
        .unwrap_or(DEFAULT_DEBUG_LOGGING);
    let debug_logging_control = log::setup_tracing(&cli, sentry_enabled, debug_logging).await?;
    trace!("cli args: {:#?}", cli);

    let os = sys::OsInfo::new();
    info!("os: {}", os);
    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    {
        os.print_linux_info();
        os.linux_check();
    }

    #[cfg(windows)]
    if cli.console {
        use windows::Win32::System::Console::AllocConsole;
        let _ = unsafe { AllocConsole() };
    }

    let context = tauri::generate_context!();
    let pkg_info = context.package_info();

    if cli.build_info {
        cli::print_build_info(pkg_info);
        return Ok(());
    }

    if let Some(Commands::Db { command: Some(cmd) }) = &cli.command {
        return db_command(cmd);
    }

    let sentry_guard = if sentry_enabled {
        sentry::init(&os)
    } else {
        None
    };

    // Built here rather than in `setup`: `FavoritesManager::new` is async while the
    // setup hook is synchronous, so awaiting it on main avoids blocking on the
    // async runtime from inside the hook, and guarantees the store is loaded
    // before any command can read it.
    let favorites_manager = match APP_DATA_DIR.clone() {
        Some(dir) => {
            info!("favorites store dir: {}", dir.display());
            Some(FavoritesManager::new(dir).await)
        }
        None => {
            error!("failed to get app data dir, favorites will be unavailable");
            None
        }
    };

    let c_os = os.clone();
    info!("app version: {}", pkg_info.version);
    info!("Starting tauri app");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(if cfg!(target_os = "windows") {
                    StateFlags::POSITION
                } else {
                    StateFlags::SIZE | StateFlags::POSITION
                })
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
        .plugin(tauri_plugin_os::init())
        .on_window_event(move |win, event| {
            window::handle_event(&c_os, win, event);
        })
        .setup(move |app| {
            info!("app setup");

            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register_all() {
                    error!("Failed to register deep link schemes: {e}");
                } else {
                    info!("Deep link schemes registered successfully");
                }
            }

            #[cfg(windows)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())
                    .inspect_err(|e| {
                        error!("failed to init updater plugin: {e}");
                    })
                    .ok();
                app.manage(PendingUpdate(Mutex::new(None)));
            }

            #[cfg(target_os = "linux")]
            linux_update_watcher::spawn(app.handle().clone());

            app.manage(cli.clone());
            app.manage(Mutex::new(debug_logging_control));
            app.manage(FavoritesState::new(favorites_manager));

            info!("Creating k/v embedded db");
            let db = match Db::new() {
                Ok(db) => db,
                Err(e) => {
                    startup_error::create_window(
                        app.handle(),
                        StartupError::new(ErrorKey::from(&e), Some(e.to_string())),
                    )?;
                    return Ok(());
                }
            };
            db.set_defaults()
                .inspect_err(|_| error!("failed to set defaults"))
                .ok();
            app.manage(db.clone());

            let app_window = AppWindow::create_main_window(app.handle(), &cli)?;
            app_window.set_bg_color(&db).ok();
            #[cfg(target_os = "linux")]
            app_window.set_max_size(os.display_server.clone()).ok();
            #[cfg(not(target_os = "linux"))]
            app_window.set_max_size().ok();

            let fs_config = {
                let path = APP_CONFIG_DIR
                    .clone()
                    .ok_or(anyhow!("failed to get app config dir"))?;
                AppFs::<AppConfig>::new(path, APP_CONFIG_FILE, None)
                    .inspect_err(|e| error!("Failed to init app config store: {e}"))?
            };
            debug!("app_config_store: {}", &fs_config.full_path.display());

            let app_config = match fs_config.read() {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("failed to read app config: {e}, falling back to default (empty) config");
                    debug!("clearing the config file");
                    fs_config
                        .clear()
                        .inspect_err(|e| error!("failed to clear the config file: {e}"))
                        .ok();
                    AppConfig::default()
                }
            };
            debug!("app_config: {app_config:?}");

            let app_state = AppState::new(os, sentry_guard);
            app.manage(Mutex::new(app_state));

            let pkg_info = app.package_info();
            let vpnd = VpndClient::new(pkg_info);

            app.manage(Mutex::new(fs_config));
            app.manage(vpnd.clone());

            let tray_manager = TrayManager::new(app.handle())?;
            app.manage(tray_manager);

            let handle = app.handle().clone();
            let mut c_vpnd = vpnd.clone();
            tokio::spawn(async move {
                info!("starting vpnd spy");

                loop {
                    match c_vpnd.vpnd_info().await {
                        Ok(info) => {
                            info!("vpnd info: {info:?}");
                            c_vpnd.update_vpnd_state(info, &handle).await.ok();
                            c_vpnd.update_config(&handle).await.ok();
                            c_vpnd.tunnel_state(&handle).await.ok();

                            vpnd_check::sentry_check(sentry_enabled, &c_vpnd).await.ok();
                            vpnd_check::netstats_check(&db, &c_vpnd).await.ok();

                            info!("watching vpnd events");
                            // start watching vpnd events, this is a blocking call
                            // and will keep the task alive as long as the grpc connection
                            // with vpnd is UP
                            c_vpnd.watch_events(&handle).await.ok();
                            // if the events stream cuts off, that means vpnd is down
                            AppState::vpnd_down(&handle).await;
                        }
                        Err(VpndError::AuthenticationRequired) => {
                            info!("authentication denied, waiting for user to authenticate");
                            AppState::vpnd_auth_denied(&handle).await;
                            c_vpnd.wait_for_auth_retry().await;
                            continue;
                        }
                        Err(_) => {
                            info!("vpnd error, downing");
                            AppState::vpnd_down(&handle).await;
                        }
                    }
                    sleep(VPND_RETRY_INTERVAL).await;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_tray::update_tray_mode,
            cmd_tray::update_tray_show_hide,
            cmd_tray::update_tray_quit,
            cmd_tray::update_tray_state,
            cmd_tray::update_tray_entry,
            cmd_tray::update_tray_exit,
            cmd_tray::update_tray_entry_visible,
            tunnel::get_vpn_config,
            tunnel::set_vpn_mode,
            tunnel::get_tunnel_state,
            tunnel::connect,
            tunnel::disconnect,
            tunnel::reconnect,
            tunnel::set_node,
            tunnel::set_quic,
            tunnel::set_fronting_mode,
            tunnel::get_default_dns,
            tunnel::set_custom_dns,
            tunnel::set_custom_dns_enabled,
            tunnel::set_no_ipv6,
            tunnel::set_allow_lan,
            tunnel::set_ad_block,
            tunnel::get_privy_derivation_message,
            tunnel::set_mixnet_traffic_config,
            tunnel::calculate_traffic_latency,
            tunnel::get_mixnet_traffic_defaults,
            tunnel::set_enable_split_tunnel,
            tunnel::get_app_list,
            tunnel::add_app_to_split_tunnel,
            tunnel::remove_app_from_split_tunnel,
            tunnel::is_split_tunnel_supported,
            tunnel::add_custom_split_tunnel_app,
            tunnel::remove_custom_split_tunnel_app,
            tunnel::set_enable_geo_location,
            tunnel::set_geo_exclusion_enabled,
            tunnel::set_geo_exclusion_listen_port,
            tunnel::set_geo_exclusion_excluded_countries,
            gateway_independence::get_tentative_gateways,
            gateway_independence::set_gateway_independence,
            gateway_independence::set_gateway_independence_notifications,
            cmd_db::db_set,
            cmd_db::db_get,
            cmd_db::db_del,
            cmd_db::db_flush,
            cmd_gw::get_gateways,
            cmd_favorites::get_favorites,
            cmd_favorites::add_favorite,
            cmd_favorites::remove_favorite,
            cmd_window::show_main_window,
            cmd_window::set_background_color,
            commands::cli::cli_args,
            cmd_log::log_js,
            cmd_log::set_debug_logging,
            cmd_log::debug_logging_enabled,
            account::get_account_state,
            account::add_account,
            account::get_account_mode,
            account::forget_account,
            account::is_account_stored,
            account::get_account_id,
            account::get_canonical_account_id,
            account::get_device_id,
            account::account_links,
            account::get_deep_link,
            account::store_deeplink_account,
            account::get_autologin_deeplink,
            account::get_account_summary,
            account::refresh_account_state,
            account::handle_subscription_payment,
            cmd_daemon::daemon_status,
            cmd_daemon::set_network,
            cmd_daemon::system_messages,
            cmd_daemon::feature_flags,
            cmd_daemon::network_compat,
            cmd_daemon::vpnd_log_dir,
            cmd_daemon::delete_logs,
            cmd_daemon::retry_authentication,
            cmd_fs::log_dir,
            cmd_fs::delete_app_logs,
            cmd_fs::zip_logs,
            cmd_sys::os_info,
            cmd_sentry::enable_sentry,
            cmd_sentry::disable_sentry,
            cmd_sentry::sentry_enabled,
            commands::network_stats::enable_netstats,
            commands::network_stats::disable_netstats,
            cmd_socks5::enable_socks5,
            cmd_socks5::disable_socks5,
            cmd_socks5::get_socks5_status,
            cmd_diag::run_diagnostic,
            cmd_diag::share_diagnostic,
            #[cfg(windows)]
            cmd_updater::fetch_update,
            #[cfg(windows)]
            cmd_updater::install_update,
        ])
        .run(context)
        .expect("error while running tauri application");

    Ok(())
}
