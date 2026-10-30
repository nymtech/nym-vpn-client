// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! NetworkManager VPN plugin for nym-vpnd.
//!
//! Spawned by NetworkManager as root, owns `org.freedesktop.NetworkManager.nymvpn` on the
//! system bus and forwards `Connect`/`Disconnect` to the daemon over its gRPC socket. The daemon
//! keeps ownership of the tun devices, routing, firewall and DNS; NetworkManager only gets the
//! external gateway address and the service state.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("nm-nymvpn-service is only supported on Linux");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
mod nm;
#[cfg(target_os = "linux")]
mod plugin;
#[cfg(target_os = "linux")]
mod service;
#[cfg(target_os = "linux")]
mod state;

#[cfg(target_os = "linux")]
#[derive(clap::Parser)]
#[command(version, about)]
struct Cli {
    /// Enable debug logging.
    #[arg(short, long)]
    verbose: bool,

    /// Serve on the session bus instead of the system bus (manual testing without root).
    #[arg(long, hide = true)]
    session_bus: bool,
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::{Arc, atomic::AtomicU32};

    use anyhow::Context;
    use clap::Parser;

    let cli = Cli::parse();
    let level = if cli.verbose {
        "info,nm_nymvpn_service=debug"
    } else {
        "info"
    };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .init();
    tracing::info!("nm-nymvpn-service {}", env!("CARGO_PKG_VERSION"));

    let state = Arc::new(AtomicU32::new(nm::service_state::INIT));
    let (tx, rx) = tokio::sync::mpsc::channel(8);

    let builder = if cli.session_bus {
        zbus::connection::Builder::session()?
    } else {
        zbus::connection::Builder::system()?
    };
    let connection = builder
        .name(nm::BUS_NAME)?
        .serve_at(nm::OBJECT_PATH, plugin::VpnPlugin::new(tx, state.clone()))?
        .build()
        .await
        .with_context(|| format!("failed to own {} on the bus", nm::BUS_NAME))?;
    tracing::info!("serving {} at {}", nm::INTERFACE, nm::OBJECT_PATH);

    let iface = connection
        .object_server()
        .interface::<_, plugin::VpnPlugin>(nm::OBJECT_PATH)
        .await?;
    // No property cache: the Disconnect decision must see NetworkManager's state as it is now, not
    // as it was when a PropertiesChanged signal was last processed.
    let network_manager = match nm::NetworkManagerProxy::builder(&connection)
        .cache_properties(zbus::proxy::CacheProperties::No)
        .build()
        .await
    {
        Ok(proxy) => Some(proxy),
        Err(err) => {
            tracing::warn!(
                "NetworkManager proxy unavailable, treating every Disconnect as a user request: {err}"
            );
            None
        }
    };

    let service = service::Service::new(
        service::GrpcDaemon::default(),
        iface,
        network_manager,
        state,
    );
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        result = service.run(rx) => result?,
        _ = sigterm.recv() => tracing::info!("SIGTERM received, exiting"),
    }
    Ok(())
}
