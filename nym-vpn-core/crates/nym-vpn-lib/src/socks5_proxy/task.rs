// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use nym_file_updater::FileUpdater;
#[cfg(target_os = "android")]
use nym_socks5_proxy::SocketProtector;
use nym_socks5_proxy::default_interface;
use nym_socks5_proxy_ipc::{DaemonMessage, InterfaceAddresses, ProxyConfig};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::Socks5ProxyEvent;

pub struct RunningTask {
    msg_tx: mpsc::UnboundedSender<DaemonMessage>,
    shutdown_token: CancellationToken,
    join_handle: JoinHandle<()>,
}

impl RunningTask {
    pub fn set_tunnel_addrs(&self, tunnel_addrs: InterfaceAddresses) {
        let msg = DaemonMessage::SetTunnelAddresses(tunnel_addrs);
        if self.msg_tx.send(msg).is_err() {
            tracing::warn!("could not send SetTunnelAddresses to proxy task: channel closed");
        }
    }

    pub fn set_excluded_countries(&self, excluded_countries: Vec<String>) {
        let msg = DaemonMessage::SetExcludedCountries(excluded_countries);
        if self.msg_tx.send(msg).is_err() {
            tracing::warn!("could not send SetExcludedCountries to proxy task: channel closed");
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }

    pub async fn join(self) {
        if let Err(err) = self.join_handle.await {
            tracing::error!("Failed to join socks5 proxy task: {err}");
        }
    }
}

pub async fn spawn(
    config: ProxyConfig,
    #[cfg(target_os = "android")] socket_protector: SocketProtector,
    event_tx: mpsc::UnboundedSender<Socks5ProxyEvent>,
    shutdown_token: CancellationToken,
) -> Result<RunningTask> {
    let (msg_tx, msg_rx) = mpsc::unbounded_channel::<DaemonMessage>();
    let (ready_tx, ready_rx) = oneshot::channel::<Result<(), String>>();

    let join_handle = tokio::spawn(supervisor(
        config,
        #[cfg(target_os = "android")]
        socket_protector,
        msg_rx,
        ready_tx,
        event_tx,
        shutdown_token.clone(),
    ));

    match ready_rx.await {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => anyhow::bail!("Proxy error during startup: {msg}"),
        Err(_) => anyhow::bail!("Proxy task exited before reporting ready"),
    }

    tracing::info!("nym-socks5-proxy task is ready");

    Ok(RunningTask {
        msg_tx,
        shutdown_token,
        join_handle,
    })
}

async fn supervisor(
    config: ProxyConfig,
    #[cfg(target_os = "android")] socket_protector: SocketProtector,
    mut msg_rx: mpsc::UnboundedReceiver<DaemonMessage>,
    ready_tx: oneshot::Sender<Result<(), String>>,
    event_tx: mpsc::UnboundedSender<Socks5ProxyEvent>,
    shutdown_token: CancellationToken,
) {
    if let Err(err) = tokio::fs::create_dir_all(&config.data_dir).await {
        let msg = format!(
            "Failed to create proxy data directory '{}': {err:#}",
            config.data_dir.display()
        );
        tracing::error!("{msg}");
        let _ = ready_tx.send(Err(msg));
        return;
    }

    let (tunnel_addrs_tx, tunnel_addrs_rx) = watch::channel(InterfaceAddresses::default());
    let (excluded_countries_tx, excluded_countries_rx) =
        watch::channel(config.excluded_countries.clone());
    let default_interface_rx = default_interface::start_monitor(shutdown_token.child_token()).await;

    let file_updater_handle = match FileUpdater::new() {
        Ok((file_updater, handle)) => {
            tokio::spawn(file_updater.run(shutdown_token.child_token()));
            handle
        }
        Err(err) => {
            let msg = format!("Failed to create file updater: {err:#}");
            tracing::error!("{msg}");
            let _ = ready_tx.send(Err(msg));
            return;
        }
    };

    match nym_socks5_proxy::run(
        config,
        default_interface_rx,
        tunnel_addrs_rx,
        excluded_countries_rx,
        shutdown_token.clone(),
        file_updater_handle,
        #[cfg(target_os = "android")]
        socket_protector,
    )
    .await
    {
        Ok(()) => {
            let _ = ready_tx.send(Ok(()));
            let _ = event_tx.send(Socks5ProxyEvent::Ready);
        }
        Err(err) => {
            let msg = format!("{err:#}");
            tracing::error!("SOCKS5 proxy task failed to start: {msg}");
            let _ = ready_tx.send(Err(msg.clone()));
            let _ = event_tx.send(Socks5ProxyEvent::Error { message: msg });
            return;
        }
    }

    // Forward DaemonMessages from the handle to the proxy's tunnel-address watch channel.
    loop {
        tokio::select! {
            msg = msg_rx.recv() => {
                match msg {
                    Some(DaemonMessage::SetTunnelAddresses(addrs)) => {
                        tracing::debug!("SOCKS5 proxy task: updating tunnel addresses");
                        let _ = tunnel_addrs_tx.send(addrs);
                    }
                    Some(DaemonMessage::SetExcludedCountries(countries)) => {
                        tracing::debug!("SOCKS5 proxy task: updating excluded countries");
                        let _ = excluded_countries_tx.send(countries);
                    }
                    Some(DaemonMessage::Terminate) | None => {
                        tracing::debug!("SOCKS5 proxy task: shutting down");
                        shutdown_token.cancel();
                        break;
                    }
                    Some(DaemonMessage::Configure(_)) => {
                        tracing::warn!("SOCKS5 proxy task: unexpected Configure message");
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                break;
            }
        }
    }

    let _ = event_tx.send(Socks5ProxyEvent::Exited { success: true });
}
