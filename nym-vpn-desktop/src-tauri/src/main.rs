// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, sync::Arc};

use anyhow::{anyhow, Context, Result};
use tauri::api::path::{config_dir, data_dir};
use tokio::{fs::try_exists, sync::Mutex};
use tracing::{debug, error, info};

use commands::*;
use states::app::AppState;

use nym_vpn_lib::nym_config;

use crate::fs::{config::AppConfig, data::AppData, storage::AppStorage};

mod commands;
mod country;
mod error;
mod fs;
mod states;
mod vpn_client;

const APP_DIR: &str = "nym-vpn";
const APP_DATA_FILE: &str = "app-data.toml";
const APP_CONFIG_FILE: &str = "config.toml";

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
    dotenvy::dotenv().ok();
    setup_logging();

    let app_data_store = {
        let mut app_data_path =
            data_dir().ok_or(anyhow!("Failed to retrieve data directory path"))?;
        app_data_path.push(APP_DIR);
        AppStorage::<AppData>::new(app_data_path, APP_DATA_FILE, None)
    };
    debug!("app_data_store: {}", app_data_store.full_path.display());

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

    let app_data = app_data_store.read().await?;
    debug!("app_data: {app_data:?}");
    let app_config = app_config_store.read().await?;
    debug!("app_config: {app_config:?}");

    // check for the existence of the env_config_file if provided
    if let Some(env_config_file) = &app_config.env_config_file {
        debug!("provided env_config_file: {}", env_config_file.display());
        if !(try_exists(env_config_file)
            .await
            .context("an error happened while trying to read env_config_file `{}`")?)
        {
            let err_message = format!(
                "app config, env_config_file `{}`: file not found",
                env_config_file.display()
            );
            error!(err_message);
            return Err(anyhow!(err_message));
        }
    } else {
        std::env::set_var("CONFIGURED", "true");
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("NETWORK_NAME", "sandbox");
        std::env::set_var("BECH32_PREFIX", "n");
        std::env::set_var("MIX_DENOM", "unym");
        std::env::set_var("MIX_DENOM_DISPLAY", "nym");
        std::env::set_var("STAKE_DENOM", "unyx");
        std::env::set_var("STAKE_DENOM_DISPLAY", "nyx");
        std::env::set_var("DENOMS_EXPONENT", "6");
        std::env::set_var(
            "REWARDING_VALIDATOR_ADDRESS",
            "n1pefc2utwpy5w78p2kqdsfmpjxfwmn9d39k5mqa",
        );
        std::env::set_var(
            "MIXNET_CONTRACT_ADDRESS",
            "n1xr3rq8yvd7qplsw5yx90ftsr2zdhg4e9z60h5duusgxpv72hud3sjkxkav",
        );
        std::env::set_var(
            "VESTING_CONTRACT_ADDRESS",
            "n1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qackslz",
        );
        std::env::set_var(
            "COCONUT_BANDWIDTH_CONTRACT_ADDRESS",
            "n13902g92xfefeyzuyed49snlm5fxv5ms6mdq5kvrut27hasdw5a9q9vyw6c",
        );
        std::env::set_var(
            "GROUP_CONTRACT_ADDRESS",
            "n18nczmqw6adwxg2wnlef3hf0etf8anccafp2pjpul5rrtmv96umyq5mv7t5",
        );
        std::env::set_var(
            "MULTISIG_CONTRACT_ADDRESS",
            "n1q3zzxl78rlmxv3vn0uf4vkyz285lk8q2xzne299yt9x6mpfgk90qukuzmv",
        );
        std::env::set_var(
            "COCONUT_DKG_CONTRACT_ADDRESS",
            "n1jsz20ggp5a6v76j060erkzvxmeus8htlpl77yxp878f0gf95cyaq6p2pee",
        );
        std::env::set_var(
            "NAME_SERVICE_CONTRACT_ADDRESS",
            "n12ne7qtmdwd0j03t9t5es8md66wq4e5xg9neladrsag8fx3y89rcs36asfp",
        );
        std::env::set_var(
            "SERVICE_PROVIDER_DIRECTORY_CONTRACT_ADDRESS",
            "n1ps5yutd7sufwg058qd7ac7ldnlazsvmhzqwucsfxmm445d70u8asqxpur4",
        );
        std::env::set_var(
            "EPHEMERA_CONTRACT_ADDRESS",
            "n19lc9u84cz0yz3fww5283nucc9yvr8gsjmgeul0",
        );
        std::env::set_var("STATISTICS_SERVICE_DOMAIN_ADDRESS", "http://0.0.0.0");
        std::env::set_var("EXPLORER_API", "https://sandbox-explorer.nymtech.net/api");
        std::env::set_var("NYXD", "https://rpc.sandbox.nymtech.net");
        std::env::set_var("NYXD_WS", "wss://rpc.sandbox.nymtech.net/websocket");
        std::env::set_var("NYM_API", "https://sandbox-nym-api1.nymtech.net/api");
    }

    // Read the env variables in the provided file and export them all to the local environment.
    nym_config::defaults::setup_env(app_config.env_config_file);

    info!("Starting tauri app");

    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(AppState::from(&app_data))))
        .manage(Arc::new(Mutex::new(app_data_store)))
        .manage(Arc::new(Mutex::new(app_config_store)))
        .setup(|_app| {
            info!("app setup");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connection::set_vpn_mode,
            connection::get_connection_state,
            connection::connect,
            connection::disconnect,
            connection::get_connection_start_time,
            app_data::get_app_data,
            app_data::set_app_data,
            app_data::set_ui_theme,
            app_data::set_entry_location_selector,
            app_data::set_monitoring,
            app_data::set_auto_connect,
            app_data::get_node_countries,
            app_data::set_root_font_size,
            node_location::set_node_location,
            node_location::get_default_node_location,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
