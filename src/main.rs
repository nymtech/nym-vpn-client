// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

mod commands;
mod error;
mod tunnel;

use std::collections::HashSet;
use crate::commands::CliArgs;
use clap::Parser;
use futures::channel::{mpsc, oneshot};
use log::warn;
use nym_bin_common::logging::setup_logging;
use nym_task::TaskManager;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::RouteManager;
use talpid_tunnel::{tun_provider::TunProvider, TunnelArgs};
use talpid_types::net::wireguard::{PeerConfig, PresharedKey, PrivateKey, PublicKey, TunnelConfig};
use talpid_wireguard::{config::Config, WireguardMonitor};

const DEFAULT_MTU: u16 = 1420;

fn init_config(args: CliArgs) -> Result<Config, error::Error> {
    Ok(Config {
        tunnel: TunnelConfig {
            private_key: PrivateKey::from(PublicKey::from_base64(&args.private_key).map_err(|_| error::Error::InvalidWireGuardKey)?.as_bytes().clone()),
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
            public_key: PublicKey::from_base64(&args.public_key).map_err(|_| error::Error::InvalidWireGuardKey)?,
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
    let config = init_config(args)?;

    let (event_tx, _) = mpsc::unbounded();
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();
    let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
    let (command_tx, _) = mpsc::unbounded();
    let command_tx = Arc::new(command_tx);
    let weak_command_tx = Arc::downgrade(&command_tx);

    let on_tunnel_event =
        move |event| -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
            let (tx, rx) = oneshot::channel::<()>();
            let _ = event_tx.unbounded_send((event, tx));
            Box::pin(async move {
                let _ = rx.await;
            })
        };
    let mut route_manager = RouteManager::new(HashSet::new()).await.expect("create route manager");
    let tun_provider = TunProvider::new();
    let shutdown = TaskManager::new(10);

    let mut firewall = Firewall::new().expect("create firewall instance");
    let mut dns_monitor = DnsMonitor::new(weak_command_tx).expect("create dns monitor");
    let route_manager_handle = route_manager.handle().expect("route handle manager");

    let tunnel_handle = tokio::task::spawn_blocking(move || {
        let args = TunnelArgs {
            runtime: tokio::runtime::Handle::current(),
            resource_dir: &PathBuf::from("/tmp/wg-cli"),
            on_event: on_tunnel_event,
            tunnel_close_rx,
            tun_provider: Arc::new(Mutex::new(tun_provider)),
            retry_attempt: 3,
            route_manager: route_manager_handle,
        };
        let monitor = WireguardMonitor::start(config, None, None, args).expect("start wg monitor");
        println!("Starting wireguard monitor");
        if let Err(e) = monitor.wait() {
            println!("Tunnel disconnected with error {:?}", e);
        } else {
            finished_shutdown_tx
                .send(())
                .expect("send finished shutdown");
            println!("Sent shutdown message");
        }
    });

    shutdown.catch_interrupt().await.expect("catch interrupt");
    let sig_handle = tokio::task::spawn_blocking(move || {
        println!("Received interrupt signal");
        route_manager.clear_routes().expect("routes clear");
        tunnel_close_tx.send(()).expect("send tunnel close");
    });

    tunnel_handle.await.expect("tunnel error");
    sig_handle.await.expect("signal error");
    finished_shutdown_rx
        .await
        .expect("received finished shutdown");
    dns_monitor.reset().expect("dns reset");
    firewall.reset_policy().expect("firewall policy reset");

    println!("Finished");

    Ok(())
}
