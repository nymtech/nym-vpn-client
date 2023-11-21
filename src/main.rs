// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

mod commands;
mod error;
mod gateway_client;
mod mixnet_processor;
mod tunnel;

use crate::commands::CliArgs;
use crate::gateway_client::{Config as GatewayConfig, GatewayData};
use crate::mixnet_processor::start_processor;
use crate::tunnel::{start_tunnel, Tunnel};
use clap::Parser;
use default_net::get_default_interface;
use futures::channel::oneshot;
use gateway_client::GatewayClient;
use log::{debug, error, info};
use nym_config::defaults::{setup_env, NymNetworkDetails};
use nym_sdk::mixnet::{MixnetClientBuilder, Recipient, StoragePaths};
use nym_task::TaskManager;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::str::FromStr;
use talpid_routing::RouteManager;
use talpid_types::net::wireguard::{
    ConnectionConfig, PeerConfig, PrivateKey, PublicKey, TunnelConfig, TunnelOptions,
};
use talpid_types::net::GenericTunnelOptions;
#[cfg(target_os = "linux")]
use talpid_types::ErrorExt;
use talpid_wireguard::config::Config;

fn init_config(args: &CliArgs, gateway_data: GatewayData) -> Result<Config, error::Error> {
    let tunnel = TunnelConfig {
        private_key: PrivateKey::from(
            *PublicKey::from_base64(&args.private_key)
                .map_err(|_| error::Error::InvalidWireGuardKey)?
                .as_bytes(),
        ),
        addresses: vec![gateway_data.private_ip],
    };
    let peers = vec![PeerConfig {
        public_key: gateway_data.public_key,
        allowed_ips: vec!["10.1.0.1".parse().unwrap()],
        endpoint: gateway_data.endpoint,
        psk: None,
    }];
    let connection_config = ConnectionConfig {
        tunnel: tunnel.clone(),
        peer: peers[0].clone(),
        exit_peer: None,
        ipv4_gateway: Ipv4Addr::from_str(&gateway_data.private_ip.to_string())?,
        ipv6_gateway: None,
        #[cfg(target_os = "linux")]
        fwmark: None,
    };
    let generic_options = GenericTunnelOptions { enable_ipv6: true };
    let wg_options = TunnelOptions::default();
    let config = Config::new(
        tunnel,
        peers,
        &connection_config,
        &wg_options,
        &generic_options,
        None,
    )?;
    Ok(config)
}

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
async fn main() -> Result<(), error::Error> {
    setup_logging();
    let args = commands::CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    let gateway_config = GatewayConfig::override_from_env(&args, GatewayConfig::default());
    info!("nym-api: {}", gateway_config.api_url);
    let gateway_client = GatewayClient::new(gateway_config)?;
    let gateway_data = gateway_client.get_gateway_data(&args.entry_gateway).await?;
    debug!("wg gateway data: {:?}", gateway_data);
    info!("wg gateway endpoint: {}", gateway_data.endpoint);
    info!("wg gateway public key: {}", gateway_data.public_key);
    info!("wg gateway private ip: {}", gateway_data.private_ip);
    let default_node_address = get_default_interface()
        .map_err(|_| crate::error::Error::DefaultInterfaceGatewayError)?
        .gateway
        .map_or(
            Err(crate::error::Error::DefaultInterfaceGatewayError),
            |g| Ok(g.ip_addr),
        )?;

    let recipient_address = Recipient::try_from_base58_string(&args.recipient_address)
        .map_err(|_| error::Error::RecipientFormattingError)?;
    info!("ip-packet-router: {:?}", recipient_address);

    let config = init_config(&args, gateway_data.clone())?;
    info!("wg mtu: {}", config.mtu);
    #[cfg(target_os = "linux")]
    info!("wg fwmark: {:?}", config.fwmark);
    #[cfg(target_os = "linux")]
    info!("wg enable_ipv6: {}", config.enable_ipv6);
    info!("wg ipv4_gateway: {}", config.ipv4_gateway);
    info!("wg ipv6_gateway: {:?}", config.ipv6_gateway);
    info!("wg peers: {:?}", config.peers);

    let shutdown = TaskManager::new(10);

    info!("Setting up route manager");
    #[cfg(target_os = "linux")]
    let mut route_manager = {
        let fwmark = 0;
        let table_id = 0;
        RouteManager::new(HashSet::new(), fwmark, table_id).await?
    };

    #[cfg(not(target_os = "linux"))]
    let mut route_manager = RouteManager::new(HashSet::new()).await?;

    let route_manager_handle = route_manager.handle()?;
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    info!("Creating tunnel");
    let mut tunnel = Tunnel::new(config, route_manager_handle)?;

    let wireguard_waiting = if args.enable_wireguard {
        info!("Starting wireguard tunnel");
        let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
        let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;
        Some((finished_shutdown_rx, tunnel_handle))
    } else {
        info!("Wireguard is disabled");
        None
    };

    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    let mixnet_client = MixnetClientBuilder::new_with_default_storage(StoragePaths::new_from_dir(
        &args.mixnet_client_path,
    )?)
    .await?
    .request_gateway(args.entry_gateway.clone())
    .network_details(NymNetworkDetails::new_from_env())
    .debug_config(debug_config)
    .build()?
    .connect_to_mixnet()
    .await?;
    let processor_config = mixnet_processor::Config::new(
        default_node_address,
        gateway_data.endpoint.ip(),
        recipient_address,
        tunnel.config.ipv4_gateway.to_string(),
        tunnel.config.ipv6_gateway.map(|ip| ip.to_string()),
    );
    info!("Mixnet processor config: {:#?}", processor_config);

    start_processor(
        processor_config,
        mixnet_client,
        &mut route_manager,
        &shutdown,
        args.enable_wireguard,
    )
    .await?;

    if let Err(e) = shutdown.catch_interrupt().await {
        error!("Could not wait for interrupts anymore - {e}. Shutting down the tunnel.");
    }

    let sig_handle = tokio::task::spawn_blocking(move || -> Result<(), error::Error> {
        debug!("Received interrupt signal");
        route_manager.clear_routes()?;
        #[cfg(target_os = "linux")]
        if let Err(error) =
            tokio::runtime::Handle::current().block_on(route_manager.clear_routing_rules())
        {
            error!(
                "{}",
                error.display_chain_with_msg("Failed to clear routing rules")
            );
        }
        tunnel_close_tx
            .send(())
            .map_err(|_| error::Error::OneshotSendError)?;
        Ok(())
    });

    if let Some((finished_shutdown_rx, tunnel_handle)) = wireguard_waiting {
        tunnel_handle.await??;
        sig_handle.await??;
        finished_shutdown_rx.await?;
    } else {
        sig_handle.await??;
    }
    tunnel.dns_monitor.reset()?;
    tunnel.firewall.reset_policy()?;

    Ok(())
}
