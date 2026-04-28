// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_socks5_proxy::{default_interface, proxy};

use std::{
    fs::{File, create_dir_all},
    io::{Write, stdout},
    mem::discriminant,
    path::Path,
};

use anyhow::{Context, Result, bail};
use nym_socks5_proxy_ipc::{
    DaemonMessage, ErrorData, InterfaceAddresses, ProxyConfig, ProxyMessage,
};
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

    // Get the default interface addresses and monitor for changes in the routing.
    let default_interface_rx = default_interface::start_monitor(shutdown_token.child_token()).await;

    // Shared VPN tunnel addressese
    let (tunnel_addrs_tx, tunnel_addrs_rx) = watch::channel(InterfaceAddresses::default());

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
    if let Err(err) = proxy::run(
        config,
        &proxy_dir,
        default_interface_rx,
        tunnel_addrs_rx,
        shutdown_token.clone(),
        #[cfg(target_os = "android")]
        std::sync::Arc::new(|_: i32| {}),
    )
    .await
    {
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
                        handle_daemon_message(&line, &tunnel_addrs_tx, &shutdown_token);
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
            bail!("Shutdown requested before configuration was received");
        }
    };

    match line
        .parse::<DaemonMessage>()
        .context("Failed to decode daemon message")?
    {
        DaemonMessage::Configure(cfg) => Ok(cfg),
        other => bail!(
            "Expected Configure as first message, got variant {:?}",
            discriminant(&other)
        ),
    }
}

fn handle_daemon_message(
    line: &str,
    tunnel_addrs_tx: &watch::Sender<InterfaceAddresses>,
    shutdown_token: &CancellationToken,
) {
    match line.parse::<DaemonMessage>() {
        Ok(DaemonMessage::Configure(_)) => {
            tracing::warn!("Received unexpected duplicate Configure message; ignoring");
            send_error_message("Unexpected Configure message");
        }
        Ok(DaemonMessage::SetTunnelAddresses(tunnel_addrs)) => {
            tracing::info!("VPN tunnel addresses changed: {tunnel_addrs:?}");
            let _ = tunnel_addrs_tx.send(tunnel_addrs);
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
