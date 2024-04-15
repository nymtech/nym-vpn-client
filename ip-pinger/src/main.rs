use clap::{Args, Parser, Subcommand};
use futures::channel::{mpsc, oneshot};
use ipnetwork::{Ipv4Network, Ipv6Network};
use log::{debug, error, info};
use nym_connection_monitor::ConnectionMonitorTask;
use nym_gateway_directory::{DescribedGatewayWithLocation, GatewayClient, IpPacketRouterAddress};
use nym_sdk::mixnet::{
    MixnetClient, MixnetClientBuilder, MixnetClientSender, MixnetMessageSender, NodeIdentity,
    Recipient, StoragePaths,
};
use nym_task::TaskManager;
use nym_vpn_lib::gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint};
use nym_vpn_lib::nym_config::defaults::var_names::{EXPLORER_API, NYM_API};
use nym_vpn_lib::nym_config::OptionalSet;
use nym_vpn_lib::wg_gateway_client::WgConfig as WgGatewayConfig;
use nym_vpn_lib::{error::*, IpPair};
use nym_vpn_lib::{nym_bin_common::bin_info_local_vergen, wg_gateway_client::WgConfig};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{
    fs,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
    sync::OnceLock,
};
use tokio::time::timeout;
use tracing::warn;

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

    let (vpn_ctrl_tx, vpn_ctrl_rx) =
        futures::channel::mpsc::unbounded::<nym_vpn_lib::NymVpnCtrlMessage>();
    let (vpn_status_tx, vpn_status_rx) =
        futures::channel::mpsc::channel::<nym_vpn_lib::SentStatus>(128);
    let (vpn_exit_tx, vpn_exit_rx) =
        futures::channel::oneshot::channel::<nym_vpn_lib::NymVpnExitStatusMessage>();

    let gateway_client = GatewayClient::new(gateway_config.clone())?;
    let gateways = gateway_client
        .lookup_described_gateways_with_location()
        .await?;
    let working_exit_gateways: Vec<DescribedGatewayWithLocation> = gateways
        .clone()
        .into_iter()
        .filter(|gateway| gateway.is_current_build())
        .collect();

    let entry_point = EntryPoint::Random;
    let exit_point = ExitPoint::Random;

    let (entry_gateway_id, entry_location) = entry_point.lookup_gateway_identity(&gateways).await?;
    let (exit_router_address, exit_location) =
        exit_point.lookup_router_address(&working_exit_gateways)?;

    let task_manager = TaskManager::new(10).named("ip_pinger");

    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = true;

    let mixnet_client = MixnetClientBuilder::new_ephemeral()
        .request_gateway(entry_gateway_id.to_string())
        .network_details(nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env())
        .debug_config(debug_config)
        .custom_shutdown(task_manager.subscribe_named("mixnet_client_main"))
        // .credentials_mode(enable_credentials_mode)
        .build()?
        .connect_to_mixnet()
        .await?;

    let nym_address = mixnet_client.nym_address().clone();
    let entry_gateway = nym_address.gateway().to_base58_string();

    info!("Successfully connected to entry gateway: {entry_gateway}");

    info!("Sending mixnet ping to ourselves to verify mixnet connection");
    let shared_mixnet_client = nym_vpn_lib::mixnet_connect::SharedMixnetClient::new(mixnet_client);
    nym_connection_monitor::self_ping_and_wait(nym_address, shared_mixnet_client.inner()).await?;
    info!("Successfully mixnet pinged ourselves");

    let exit_gateway = exit_router_address.gateway().to_base58_string();
    info!("Connecting to exit gateway: {exit_gateway}");
    let enable_two_hop = false;
    let our_ips = nym_vpn_lib::mixnet_connect::connect_to_ip_packet_router(
        shared_mixnet_client,
        &exit_router_address,
        None,
        enable_two_hop,
    )
    .await?;
    info!("Successfully connected to exit gateway");
    info!("Using mixnet VPN IP addresses: {our_ips}");

    // Perform ICMP connectivity check for exit gateway

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
