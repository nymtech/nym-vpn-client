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
use futures::channel::oneshot;
use gateway_client::GatewayClient;
use log::{debug, error, warn};
use nym_bin_common::logging::setup_logging;
use nym_config::defaults::setup_env;
use nym_sdk::mixnet::Recipient;
use nym_task::TaskManager;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use talpid_routing::RouteManager;
use talpid_types::net::wireguard::{
    ConnectionConfig, PeerConfig, PresharedKey, PrivateKey, PublicKey, TunnelConfig, TunnelOptions,
};
use talpid_types::net::GenericTunnelOptions;
use talpid_wireguard::config::Config;

fn init_config(args: CliArgs, gateway_data: GatewayData) -> Result<Config, error::Error> {
    let tunnel = TunnelConfig {
        private_key: PrivateKey::from(
            PublicKey::from_base64(&args.private_key)
                .map_err(|_| error::Error::InvalidWireGuardKey)?
                .as_bytes()
                .clone(),
        ),
        addresses: args
            .addresses
            .iter()
            .filter_map(|ip| {
                if let Ok(parsed) = Ipv4Addr::from_str(ip) {
                    Some(IpAddr::V4(parsed))
                } else if let Ok(parsed) = Ipv6Addr::from_str(ip) {
                    Some(IpAddr::V6(parsed))
                } else {
                    None
                }
            })
            .collect(),
    };
    let peers = vec![PeerConfig {
        public_key: gateway_data.public_key,
        allowed_ips: vec![gateway_data.endpoint.ip().into()],
        endpoint: gateway_data.endpoint,
        psk: args
            .psk
            .map(|psk| match PublicKey::from_base64(&psk) {
                Ok(key) => Some(PresharedKey::from(Box::new(key.as_bytes().clone()))),
                Err(e) => {
                    warn!("Could not decode pre-shared key, not using one: {e:?}");
                    None
                }
            })
            .flatten(),
    }];
    let connection_config = ConnectionConfig {
        tunnel: tunnel.clone(),
        peer: peers[0].clone(),
        exit_peer: None,
        ipv4_gateway: Ipv4Addr::from_str(&args.ipv4_gateway)?,
        ipv6_gateway: None,
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

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    setup_logging();
    let args = commands::CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    let gateway_config = GatewayConfig::override_from_env(&args, GatewayConfig::default());
    let gateway_client = GatewayClient::new(gateway_config)?;
    let gateway_data = gateway_client.get_gateway_data(&args.entry_gateway).await?;
    let gateway_pub_key = gateway_data.public_key.to_string();

    let recipient_address = Recipient::try_from_base58_string(&args.recipient_address)
        .map_err(|_| error::Error::RecipientFormattingError)?;
    let config = init_config(args, gateway_data)?;
    let shutdown = TaskManager::new(10);

    let mut route_manager = RouteManager::new(HashSet::new()).await?;
    let route_manager_handle = route_manager.handle()?;
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let mut tunnel = Tunnel::new(config, route_manager_handle)?;

    let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;
    let processor_config = mixnet_processor::Config::new(
        gateway_pub_key,
        recipient_address,
        tunnel.config.ipv4_gateway.to_string(),
        tunnel.config.ipv6_gateway.map(|ip| ip.to_string()),
    );
    start_processor(processor_config, &mut route_manager, &shutdown).await?;

    if let Err(e) = shutdown.catch_interrupt().await {
        error!("Could not wait for interrupts anymore - {e}. Shutting down the tunnel.");
    }
    let sig_handle = tokio::task::spawn_blocking(move || -> Result<(), error::Error> {
        debug!("Received interrupt signal");
        route_manager.clear_routes()?;
        #[cfg(target_os = "linux")]
        if let Err(error) = tokio::runtime::Handle::current()
            .block_on(shared_values.route_manager.clear_routing_rules())
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

    tunnel_handle.await??;
    sig_handle.await??;
    finished_shutdown_rx.await?;
    tunnel.dns_monitor.reset()?;
    tunnel.firewall.reset_policy()?;

    Ok(())
}
