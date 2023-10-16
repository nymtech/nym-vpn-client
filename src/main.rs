// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

mod commands;
mod error;
mod mixnet_processor;
mod tunnel;

use crate::commands::CliArgs;
use crate::mixnet_processor::start_processor;
use crate::tunnel::{start_tunnel, Tunnel};
use clap::Parser;
use futures::channel::oneshot;
use log::{debug, error, warn};
use nym_bin_common::logging::setup_logging;
use nym_sdk::mixnet::Recipient;
use nym_task::TaskManager;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use talpid_routing::RouteManager;
use talpid_types::net::wireguard::{PeerConfig, PresharedKey, PrivateKey, PublicKey, TunnelConfig};
use talpid_wireguard::config::Config;

const DEFAULT_MTU: u16 = 1420;

fn init_config(args: CliArgs) -> Result<Config, error::Error> {
    Ok(Config {
        tunnel: TunnelConfig {
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
        },
        peers: vec![PeerConfig {
            public_key: PublicKey::from_base64(&args.public_key)
                .map_err(|_| error::Error::InvalidWireGuardKey)?,
            allowed_ips: args
                .allowed_ips
                .iter()
                .filter_map(|ip| ip.parse().ok())
                .collect(),
            endpoint: SocketAddr::from_str(&args.endpoint)?,
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
        }],
        ipv4_gateway: Ipv4Addr::from_str(&args.ipv4_gateway)?,
        ipv6_gateway: None,
        mtu: DEFAULT_MTU,
        obfuscator_config: None,
    })
}

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    setup_logging();
    let args = commands::CliArgs::parse();
    let recipient_address = Recipient::try_from_base58_string(&args.recipient_address)
        .map_err(|_| error::Error::RecipientFormattingError)?;
    let config = init_config(args)?;
    let shutdown = TaskManager::new(10);

    let mut route_manager = RouteManager::new(HashSet::new()).await?;
    let route_manager_handle = route_manager.handle()?;
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let mut tunnel = Tunnel::new(config, route_manager_handle)?;

    let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;
    let processor_config = mixnet_processor::Config::new(recipient_address);
    start_processor(processor_config, &shutdown).await?;

    if let Err(e) = shutdown.catch_interrupt().await {
        error!("Could not wait for interrupts anymore - {e}. Shutting down the tunnel.");
    }
    let sig_handle = tokio::task::spawn_blocking(move || -> Result<(), error::Error> {
        debug!("Received interrupt signal");
        route_manager.clear_routes()?;
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
