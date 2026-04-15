// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod proxy;
mod routing;

#[cfg(target_os = "windows")]
mod windows_bind;

use std::{
    fs::{File, create_dir_all},
    io::{Write, stdout},
    mem::discriminant,
    net::IpAddr,
    path::Path,
};

use anyhow::{Context, Result};
use nym_socks5_proxy_ipc::{DaemonMessage, ErrorData, ProxyConfig, ProxyMessage};
use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    sync::watch,
};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let shutdown_token = CancellationToken::new();
    install_signal_handlers(shutdown_token.clone());

    // Shared VPN tunnel address
    let (tunnel_addr_tx, tunnel_addr_rx) = watch::channel::<Option<IpAddr>>(None);

    // Messages from the daemon are passed via stdin.
    // Responses are written via stdout.
    let mut lines = BufReader::new(stdin()).lines();

    let config = match read_initial_config(&mut lines, &shutdown_token).await {
        Ok(cfg) => cfg,
        Err(err) => {
            send_error_message(&format!("{err:#}"));
            return Err(err);
        }
    };

    let proxy_dir = config.data_dir.join("nym-socks5-proxy");
    if let Err(err) = create_dir_all(&proxy_dir).with_context(|| {
        format!(
            "Failed to create proxy data directory '{}'",
            proxy_dir.display()
        )
    }) {
        send_error_message(&format!("{err:#}"));
        return Err(err);
    }

    if let Err(err) = init_tracing(&proxy_dir, &config.log_level) {
        send_error_message(&format!("{err:#}"));
        return Err(err);
    }

    tracing::info!("nym-socks5-proxy starting. config={config:#?}");

    // Start the SOCKS5 proxy listener.
    if let Err(err) = proxy::run(config, &proxy_dir, tunnel_addr_rx, shutdown_token.clone()).await {
        let msg = format!("{err:#}");
        tracing::error!("SOCKS5 proxy failed to start: {msg}");
        send_error_message(&msg);
        return Err(err);
    }

    // Notify the daemon that the proxy is bound and ready.
    send_message(&ProxyMessage::Ack);
    tracing::info!("SOCKS5 proxy ready");

    // Continue reading daemon messages until stdin EOF or signal.
    loop {
        tokio::select! {
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) if !line.trim().is_empty() => {
                        handle_daemon_message(&line, &tunnel_addr_tx, &shutdown_token);
                    }
                    Ok(Some(_)) => {} // blank line — ignore
                    Ok(None) => {
                        tracing::info!("Stdin end-of-file; shutting down");
                        shutdown_token.cancel();
                        break;
                    }
                    Err(err) => {
                        tracing::warn!("Error reading stdin: {err}; shutting down");
                        shutdown_token.cancel();
                        break;
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                tracing::info!("Shutdown requested");
                break;
            }
        }
    }

    tracing::info!("nym-socks5-proxy exiting");
    Ok(())
}

// Don't add tracing in this function as it's not initialized yet!
async fn read_initial_config(
    lines: &mut tokio::io::Lines<BufReader<tokio::io::Stdin>>,
    shutdown_token: &CancellationToken,
) -> Result<ProxyConfig> {
    let line = tokio::select! {
        result = lines.next_line() => {
            result
                .context("Error reading from stdin")?
                .context("Stdin closed before configuration was received")?
        }
        _ = shutdown_token.cancelled() => {
            anyhow::bail!("Shutdown requested before configuration was received");
        }
    };

    match line
        .parse::<DaemonMessage>()
        .context("Failed to decode daemon message")?
    {
        DaemonMessage::Configure(cfg) => Ok(cfg),
        other => anyhow::bail!(
            "Expected Configure as first message, got variant {:?}",
            discriminant(&other)
        ),
    }
}

fn handle_daemon_message(
    line: &str,
    tunnel_addr_tx: &watch::Sender<Option<IpAddr>>,
    shutdown_token: &CancellationToken,
) {
    match line.parse::<DaemonMessage>() {
        Ok(DaemonMessage::Configure(_)) => {
            tracing::warn!("Received unexpected duplicate Configure message; ignoring");
            send_error_message("Unexpected Configure message");
        }
        Ok(DaemonMessage::VpnConnected(data)) => {
            tracing::info!("VPN tunnel connected with address {}", data.tunnel_addr);
            let _ = tunnel_addr_tx.send(Some(data.tunnel_addr));
            send_message(&ProxyMessage::Ack);
        }
        Ok(DaemonMessage::VpnDisconnected) => {
            tracing::info!("VPN tunnel disconnected; reverting to default routing");
            let _ = tunnel_addr_tx.send(None);
            send_message(&ProxyMessage::Ack);
        }
        Ok(DaemonMessage::Terminate) => {
            tracing::info!("Received Terminate from daemon — shutting down cleanly");
            shutdown_token.cancel();
        }
        Err(err) => {
            tracing::warn!("Failed to parse daemon message: {err}; raw: {line}");
        }
    }
}

fn send_message(msg: &ProxyMessage) {
    println!("{msg}");
    let _ = stdout().flush();
}

fn send_error_message(msg: &str) {
    send_message(&ProxyMessage::Error(ErrorData {
        message: msg.to_string(),
    }));
}

fn install_signal_handlers(shutdown_token: CancellationToken) {
    #[cfg(unix)]
    tokio::spawn(async move {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
        tokio::select! {
            _ = sigterm.recv() => tracing::info!("Received SIGTERM"),
            _ = sigint.recv() => tracing::info!("Received SIGINT"),
        }
        shutdown_token.cancel();
    });

    #[cfg(windows)]
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
        tracing::info!("Received Ctrl-C");
        shutdown_token.cancel();
    });
}
fn init_tracing(proxy_dir: &Path, log_level: &str) -> Result<()> {
    let log_path = proxy_dir.join("nym-socks5-proxy.log");
    let file = File::create(&log_path)
        .with_context(|| format!("Failed to open log file '{}'", log_path.display()))?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::Layer::new()
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false),
        )
        .try_init()
        .context("Failed to initialize tracing")?;

    Ok(())
}
