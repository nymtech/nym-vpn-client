// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, sync::Arc};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use tauri::{api::path::config_dir, Manager};
use tokio::{fs::try_exists, sync::Mutex};
use tracing::{debug, error, info, trace};

use commands::db as cmd_db;
use commands::*;
use states::app::AppState;

use nym_vpn_lib::nym_config;

use crate::{
    cli::{print_build_info, Cli},
    db::Db,
    fs::{config::AppConfig, storage::AppStorage},
};

mod cli;
mod commands;
mod country;
mod db;
mod error;
mod events;
mod fs;
mod http;
mod network;
mod states;
mod vpn_client;

pub const APP_DIR: &str = "nym-vpn";
const APP_CONFIG_FILE: &str = "config.toml";
const ENV_APP_NOSPLASH: &str = "APP_NOSPLASH";

pub fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    dotenvy::dotenv().ok();
    setup_logging();

    // parse the command line arguments
    let cli = Cli::parse();
    trace!("cli args: {:#?}", cli);

    let context = tauri::generate_context!();

    if cli.build_info {
        print_build_info(context.package_info());
        return Ok(());
    }

    let app_config_store = {
        let mut app_config_path =
            config_dir().ok_or(anyhow!("Failed to retrieve config directory path"))?;
        app_config_path.push(APP_DIR);
        AppStorage::<AppConfig>::new(app_config_path, APP_CONFIG_FILE, None)
    };
    debug!(
        "app_config_store: {}",
        &app_config_store.full_path.display()
    );

    let app_config = app_config_store.read().await?;
    debug!("app_config: {app_config:?}");

    // check for the existence of the env_config_file if provided
    if let Some(env_config_file) = &app_config.env_config_file {
        debug!("provided env_config_file: {}", env_config_file.display());
        if !(try_exists(env_config_file)
            .await
            .context("an error happened while reading env_config_file `{}`")?)
        {
            let err_message = format!(
                "app config, env_config_file `{}`: file not found",
                env_config_file.display()
            );
            error!(err_message);
            return Err(anyhow!(err_message));
        }
    } else if cli.sandbox {
        // If no env_config_file is provided, setup the sandbox environment
        // This is tempory until we switch to mainnet
        network::setup_sandbox_environment();
    }

    // Read the env variables in the provided file and export them all to the local environment.
    nym_config::defaults::setup_env(app_config.env_config_file.clone());

    info!("Creating k/v embedded db");
    let db = Db::new()?;

    let app_state = AppState::try_from((&db, &app_config)).map_err(|e| {
        error!("failed to create app state from saved app data and config: {e}");
        e
    })?;

    info!("Starting tauri app");

    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(app_state)))
        .manage(Arc::new(app_config))
        .manage(Arc::new(cli))
        .manage(db)
        .setup(move |app| {
            info!("app setup");
            let env_nosplash = env::var(ENV_APP_NOSPLASH).map(|_| true).unwrap_or(false);
            trace!("env APP_NOSPLASH: {}", env_nosplash);

            // if splash-screen is disabled, remove it and show
            // the main window without waiting for frontend signal
            if cli.nosplash || env_nosplash {
                debug!("splash screen disabled, showing main window");
                let main_win = app.get_window("main").expect("failed to get main window");
                main_win
                    .eval("document.getElementById('splash').remove();")
                    .expect("failed to remove splash screen");

                main_win.show().expect("failed to show main window");
            }
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
            node_location::get_fastest_node_location,
            node_location::get_countries,
            window::show_main_window,
            commands::cli::cli_args,
            log::log_js,
        ])
        .run(context)
        .expect("error while running tauri application");

    Ok(())
}
