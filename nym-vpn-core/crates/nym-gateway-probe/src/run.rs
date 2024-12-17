// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use clap::Parser;
use nym_bin_common::bin_info;
use nym_config::defaults::setup_env;
use nym_gateway_directory::{EntryPoint, GatewayMinPerformance};
use nym_gateway_probe::{ProbeResult, TestedNode};
use nym_sdk::mixnet::NodeIdentity;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::*;

fn pretty_build_info_static() -> &'static str {
    static PRETTY_BUILD_INFORMATION: OnceLock<String> = OnceLock::new();
    PRETTY_BUILD_INFORMATION.get_or_init(|| bin_info!().pretty_print())
}

fn validate_node_identity(s: &str) -> Result<NodeIdentity, String> {
    match s.parse() {
        Ok(cg) => Ok(cg),
        Err(_) => Err(format!("failed to parse country group: {}", s)),
    }
}

#[derive(Parser)]
#[clap(author = "Nymtech", version, long_version = pretty_build_info_static(), about)]
struct CliArgs {
    /// Path pointing to an env file describing the network.
    #[arg(short, long)]
    config_env_file: Option<PathBuf>,

    #[arg(long, short = 'g', alias = "gateway")]
    entry_gateway: Option<String>,

    /// Identity of the node to test
    #[arg(long, short, value_parser = validate_node_identity)]
    node: Option<NodeIdentity>,

    #[arg(long)]
    min_gateway_mixnet_performance: Option<u8>,

    #[arg(long)]
    min_gateway_vpn_performance: Option<u8>,

    #[arg(long)]
    only_wireguard: bool,

    #[arg(long)]
    ignore_egress_epoch_role: bool,

    #[arg(long)]
    no_log: bool,
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

pub(crate) async fn run() -> anyhow::Result<ProbeResult> {
    let args = CliArgs::parse();
    if !args.no_log {
        setup_logging();
    }
    debug!("{:?}", nym_bin_common::bin_info_local_vergen!());
    setup_env(args.config_env_file.as_ref());

    let min_gateway_performance = GatewayMinPerformance::from_percentage_values(
        args.min_gateway_mixnet_performance.map(u64::from),
        args.min_gateway_vpn_performance.map(u64::from),
    )?;

    let entry = if let Some(gateway) = args.entry_gateway {
        EntryPoint::from_base58_string(&gateway)?
    } else {
        fetch_random_gateway_with_ipr(min_gateway_performance).await?
    };

    let test_point = if let Some(node) = args.node {
        TestedNode::Custom { identity: node }
    } else {
        TestedNode::SameAsEntry
    };

    nym_gateway_probe::probe(
        entry,
        test_point,
        min_gateway_performance,
        args.only_wireguard,
        args.ignore_egress_epoch_role,
    )
    .await
}

async fn fetch_random_gateway_with_ipr(
    min_gateway_performance: GatewayMinPerformance,
) -> anyhow::Result<EntryPoint> {
    // We're fetching gateways with IPR, since they are more interesting to ping, but we can probe
    // gateways without an IPR as well
    tracing::info!("Selecting random gateway with IPR enabled");
    let gateways = nym_gateway_probe::fetch_gateways_with_ipr(min_gateway_performance).await?;
    let gateway = gateways
        .random_gateway()
        .ok_or(anyhow!("No gateways returned by nym-api"))?;
    Ok(EntryPoint::Gateway {
        identity: gateway.identity(),
    })
}
