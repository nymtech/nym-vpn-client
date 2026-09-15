// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_file_updater::FileUpdater;
use nym_socks5_proxy::{default_interface, proxy};

use std::{
    fs::File,
    io::{Write, stdout},
    mem::discriminant,
    path::Path,
};

use anyhow::{Context, Result, bail};
use nym_socks5_proxy_ipc::{
    DaemonMessage, ErrorData, InterfaceAddresses, ProxyConfig, ProxyMessage, validate_country_codes,
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

    // ProxyConfig::validate() will ensure the data and log directories exist, amongst other things.
    if let Err(err) = config.validate() {
        send_error_message(&format!("Invalid configuration: {err}"));
        bail!("Invalid configuration");
    }

    // Get the default interface addresses and monitor for changes in the routing.
    let default_interface_rx = default_interface::start_monitor(shutdown_token.child_token()).await;

    // Shared VPN tunnel addressese
    let (tunnel_addrs_tx, tunnel_addrs_rx) = watch::channel(InterfaceAddresses::default());

    // Shared geo-exclusion excluded countries list, updatable at runtime without a restart.
    let (excluded_countries_tx, excluded_countries_rx) =
        watch::channel(config.excluded_countries.clone());

    if let Err(err) = init_tracing(&config.log_dir, &config.log_level) {
        send_error_message(&format!("{err:#}"));
        return Err(err);
    }

    tracing::info!("nym-socks5-proxy starting. config={config:#?}");

    let (file_updater, file_updater_handle) = FileUpdater::new();
    tokio::spawn(file_updater.run(shutdown_token.child_token()));

    // Start the SOCKS5 proxy listener.
    if let Err(err) = proxy::run(
        config,
        default_interface_rx,
        tunnel_addrs_rx,
        excluded_countries_rx,
        shutdown_token.clone(),
        file_updater_handle,
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
                        handle_daemon_message(&line, &tunnel_addrs_tx, &excluded_countries_tx, &shutdown_token);
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
    excluded_countries_tx: &watch::Sender<Vec<String>>,
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
        Ok(DaemonMessage::SetExcludedCountries(countries)) => {
            if let Err(err) = validate_country_codes(&countries) {
                tracing::warn!("Rejected SetExcludedCountries: {err}");
                send_error_message(&format!("Invalid excluded countries: {err}"));
                return;
            }
            tracing::info!("Geo-exclusion excluded countries changed: {countries:?}");
            let _ = excluded_countries_tx.send(countries);
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

fn init_tracing(log_dir: &Path, log_level: &str) -> Result<()> {
    let log_path = log_dir.join("nym-socks5-proxy.log");
    let file = File::create(&log_path)
        .with_context(|| format!("Failed to open log file '{}'", log_path.display()))?;

    // Only log nym_socks5_proxy and nym_file_updater at the configured level;
    // everything else (hyper, reqwest, tower, …) is silenced to warn.
    let filter = EnvFilter::new(format!(
        "warn,nym_socks5_proxy={log_level},nym_file_updater={log_level}"
    ));

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
