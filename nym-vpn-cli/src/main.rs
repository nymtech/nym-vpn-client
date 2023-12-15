// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod commands;

use nym_vpn_lib::error::*;
use nym_vpn_lib::gateway_client::Config as GatewayConfig;
use nym_vpn_lib::NymVpn;

use crate::commands::override_from_env;
use clap::Parser;
use log::*;
use nym_vpn_lib::nym_config::defaults::setup_env;

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
    debug!("{:?}", nym_vpn_lib::nym_bin_common::bin_info!());
    setup_env(args.config_env_file.as_ref());

    // Setup gateway configuration
    let gateway_config = override_from_env(&args, GatewayConfig::default());
    info!("nym-api: {}", gateway_config.api_url());

    let nym_vpn = NymVpn {
        gateway_config,
        mixnet_client_path: args.mixnet_client_path,
        entry_gateway: args.entry_gateway,
        exit_router: args.exit_router,
        enable_wireguard: args.enable_wireguard,
        private_key: args.private_key,
        ip: args.ip,
        mtu: args.mtu,
        disable_routing: args.disable_routing,
        enable_two_hop: args.enable_two_hop,
        enable_poisson_rate: args.enable_poisson_rate,
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
