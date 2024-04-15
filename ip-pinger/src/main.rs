use clap::{Args, Parser, Subcommand};
use ipnetwork::{Ipv4Network, Ipv6Network};
use nym_vpn_lib::gateway_directory::Config;
use nym_vpn_lib::gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint};
use nym_vpn_lib::nym_config::defaults::var_names::{EXPLORER_API, NYM_API};
use nym_vpn_lib::nym_config::OptionalSet;
use nym_vpn_lib::wg_gateway_client::WgConfig as WgGatewayConfig;
use nym_vpn_lib::{error::*, IpPair, NodeIdentity};
use nym_vpn_lib::{nym_bin_common::bin_info_local_vergen, wg_gateway_client::WgConfig};
use nym_vpn_lib::{NymVpn, Recipient};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
    sync::OnceLock,
    fs,
};

use log::*;
use nym_vpn_lib::nym_config::defaults::{setup_env, var_names};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(err) = run().await {
        error!("Exit with error: {err}");
        eprintln!("An error occurred: {err}");
        std::process::exit(1)
    }
    Ok(())
}

async fn run() -> Result<()> {
    setup_logging();
    debug!("{:?}", nym_vpn_lib::nym_bin_common::bin_info!());
    // setup_env(args.config_env_file.as_ref());
    setup_env::<PathBuf>(None);
    run_vpn().await
}

async fn run_vpn() -> Result<()> {
    let gateway_config = GatewayConfig::default()
        .with_optional_env(GatewayConfig::with_custom_api_url, None, NYM_API)
        .with_optional_env(GatewayConfig::with_custom_explorer_url, None, EXPLORER_API);
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "explorer-api: {}",
        gateway_config
            .explorer_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    let entry_point = EntryPoint::Random;
    let exit_point = EntryPoint::Random;

    // let mut nym_vpn = NymVpn::new(entry_point, exit_point);
    // nym_vpn.gateway_config = gateway_config;
    // nym_vpn.disable_background_cover_traffic = true;
    // nym_vpn.run().await?;
    Ok(())
}

fn setup_logging() {
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
