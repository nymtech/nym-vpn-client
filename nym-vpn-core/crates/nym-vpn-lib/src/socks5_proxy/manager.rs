// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use super::process::{Socks5ProcessEvent, Socks5ProcessTask, Socks5ProxyProcess};

use nym_socks5_proxy_ipc::ProxyConfig;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

enum Socks5ProxyState {
    Stopped,
    Starting,
    Running(Socks5ProxyProcess),
    Stopping,
}

pub struct Socks5ProxyManager {
    state: Socks5ProxyState,
    pub tunnel_addr: Option<IpAddr>,
}

impl Socks5ProxyManager {
    pub fn new() -> Self {
        Self {
            state: Socks5ProxyState::Stopped,
            tunnel_addr: None,
        }
    }

    pub async fn start_or_stop(
        &mut self,
        enabled: bool,
        config: ProxyConfig,
        shutdown_token: CancellationToken,
    ) {
        if enabled {
            self.start(config, shutdown_token).await;
        } else {
            self.stop().await;
        }
    }

    pub async fn start(&mut self, config: ProxyConfig, shutdown_token: CancellationToken) {
        let previous = std::mem::replace(&mut self.state, Socks5ProxyState::Starting);

        match previous {
            Socks5ProxyState::Stopped => {}
            Socks5ProxyState::Starting => {
                tracing::debug!("nym-socks5-proxy is already starting");
                self.state = Socks5ProxyState::Starting;
                return;
            }
            Socks5ProxyState::Running(process) => {
                tracing::debug!("nym-socks5-proxy is already running");
                self.state = Socks5ProxyState::Running(process);
                return;
            }
            Socks5ProxyState::Stopping => {
                tracing::debug!("nym-socks5-proxy is already stopping");
                self.state = Socks5ProxyState::Stopping;
                return;
            }
        }

        let (event_tx, event_rx) = mpsc::unbounded_channel::<Socks5ProcessEvent>();

        match Socks5ProcessTask::spawn(config, event_tx, shutdown_token.child_token()).await {
            Ok((handle, join_handle)) => {
                spawn_event_logger(event_rx);

                if let Some(addr) = self.tunnel_addr {
                    handle.notify_vpn_connected(addr);
                }

                self.state = Socks5ProxyState::Running(Socks5ProxyProcess {
                    handle,
                    join_handle,
                });
            }
            Err(err) => {
                tracing::warn!("Failed to start nym-socks5-proxy (continuing without it): {err}");
                self.state = Socks5ProxyState::Stopped;
            }
        }
    }

    pub async fn stop(&mut self) {
        let previous = std::mem::replace(&mut self.state, Socks5ProxyState::Stopping);

        let Socks5ProxyState::Running(process) = previous else {
            match previous {
                Socks5ProxyState::Stopped => {
                    tracing::debug!("nym-socks5-proxy is already stopped");
                    self.state = Socks5ProxyState::Stopped;
                }
                Socks5ProxyState::Starting => {
                    tracing::debug!("nym-socks5-proxy is already starting");
                    self.state = Socks5ProxyState::Starting;
                }
                Socks5ProxyState::Stopping => {
                    tracing::debug!("nym-socks5-proxy is already stopping");
                    self.state = Socks5ProxyState::Stopping;
                }
                Socks5ProxyState::Running(_) => unreachable!(),
            }
            return;
        };

        process.handle.shutdown();

        if let Err(err) = process.join_handle.await {
            tracing::error!("Failed to join on socks5 process task: {err}");
        }

        self.state = Socks5ProxyState::Stopped;
    }

    pub fn notify_connected(&mut self, tunnel_addr: Option<IpAddr>) {
        self.tunnel_addr = tunnel_addr;

        if let (Some(addr), Socks5ProxyState::Running(process)) = (tunnel_addr, &self.state) {
            process.handle.notify_vpn_connected(addr);
        }
    }

    pub fn notify_disconnected(&self) {
        if let Socks5ProxyState::Running(process) = &self.state {
            process.handle.notify_vpn_disconnected();
        }
    }
}

fn spawn_event_logger(mut event_rx: mpsc::UnboundedReceiver<Socks5ProcessEvent>) {
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                Socks5ProcessEvent::Ready => {
                    tracing::info!("nym-socks5-proxy ready");
                }
                Socks5ProcessEvent::StatusUpdate { active_connections } => {
                    tracing::debug!(active_connections, "nym-socks5-proxy status update");
                }
                Socks5ProcessEvent::Error { message } => {
                    tracing::error!(%message, "nym-socks5-proxy reported an error");
                }
                Socks5ProcessEvent::Exited { success } => {
                    tracing::info!(success, "nym-socks5-proxy exited");
                    break;
                }
            }
        }
    });
}
