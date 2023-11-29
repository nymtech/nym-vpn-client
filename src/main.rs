// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

mod commands;

use nym_vpn_cli::error::*;
use nym_vpn_cli::gateway_client::Config as GatewayConfig;
use nym_vpn_cli::NymVPN;

use crate::commands::override_from_env;
use clap::Parser;
use log::*;
use nym_config::defaults::setup_env;
#[cfg(target_os = "linux")]
use talpid_types::ErrorExt;

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

async fn run() -> Result<()> {
    setup_logging();
    let args = commands::CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    // Setup gateway configuration
    let gateway_config = override_from_env(&args, GatewayConfig::default());
    info!("nym-api: {}", gateway_config.api_url());

    let nym_vpn = NymVPN {
        gateway_config,
        enable_wireguard: args.enable_wireguard,
        mixnet_client_path: args.mixnet_client_path,
        entry_gateway: args.entry_gateway,
        exit_router: args.exit_router,
        private_key: args.private_key,
        ip: args.ip,
        disable_routing: args.disable_routing,
        mtu: args.mtu,
    };
    nym_vpn.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = run().await {
        error!("Exit with error: {err}");
        eprintln!("An error occurred: {err}");
        std::process::exit(1)
    }
    Ok(())
}
