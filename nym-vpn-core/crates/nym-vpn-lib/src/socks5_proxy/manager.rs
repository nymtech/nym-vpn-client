// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    mem::replace,
    net::{Ipv4Addr, Ipv6Addr},
};

use super::process::{Socks5ProcessEvent, Socks5ProcessTask, Socks5ProxyProcess};

use nym_socks5_proxy_ipc::{InterfaceAddresses, ProxyConfig};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

enum Socks5ProxyState {
    Stopped,
    Starting(CancellationToken),
    Running(Socks5ProxyProcess),
    Stopping,
}

pub struct Socks5ProxyManager {
    state: Socks5ProxyState,
    tunnel_addrs: InterfaceAddresses,
}

impl Socks5ProxyManager {
    pub fn new() -> Self {
        Self {
            state: Socks5ProxyState::Stopped,
            tunnel_addrs: InterfaceAddresses::default(),
        }
    }

    pub async fn start(&mut self, config: ProxyConfig, shutdown_token: CancellationToken) {
        match self.state {
            Socks5ProxyState::Starting(_) => {
                tracing::debug!("nym-socks5-proxy is already starting");
                return;
            }
            Socks5ProxyState::Running(_) => {
                tracing::debug!("nym-socks5-proxy is already running");
                return;
            }
            Socks5ProxyState::Stopping => {
                tracing::debug!("nym-socks5-proxy is already stopping");
                return;
            }
            Socks5ProxyState::Stopped => {}
        }

        let child_token = shutdown_token.child_token();
        self.state = Socks5ProxyState::Starting(child_token.clone());

        let (event_tx, event_rx) = mpsc::unbounded_channel::<Socks5ProcessEvent>();

        match Socks5ProcessTask::spawn(config, event_tx, child_token).await {
            Ok((handle, join_handle)) => {
                spawn_event_logger(event_rx);

                handle.set_tunnel_addrs(self.tunnel_addrs.clone());

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
        match replace(&mut self.state, Socks5ProxyState::Stopping) {
            Socks5ProxyState::Stopped => {
                tracing::debug!("nym-socks5-proxy is already stopped");
                self.state = Socks5ProxyState::Stopped;
            }
            Socks5ProxyState::Starting(token) => {
                tracing::warn!(
                    "stop() called while nym-socks5-proxy is still starting; cancelling startup"
                );
                token.cancel();
                self.state = Socks5ProxyState::Stopped;
            }
            Socks5ProxyState::Stopping => {
                tracing::debug!("nym-socks5-proxy is already stopping");
                self.state = Socks5ProxyState::Stopping;
            }
            Socks5ProxyState::Running(process) => {
                process.handle.shutdown();
                if let Err(err) = process.join_handle.await {
                    tracing::error!("Failed to join on socks5 process task: {err}");
                }
                self.state = Socks5ProxyState::Stopped;
            }
        }
    }

    pub fn set_tunnel_addrs(&mut self, v4_addr: Option<Ipv4Addr>, v6_addr: Option<Ipv6Addr>) {
        self.tunnel_addrs.v4_addr = v4_addr;
        self.tunnel_addrs.v6_addr = v6_addr;

        if let Socks5ProxyState::Running(process) = &self.state {
            tracing::debug!(
                "Notifying nym-socks5-proxy of new tunnel addresses: {:?}",
                self.tunnel_addrs
            );
            process.handle.set_tunnel_addrs(self.tunnel_addrs.clone());
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
                    tracing::info!("nym-socks5-proxy active connections: {active_connections}");
                }
                Socks5ProcessEvent::Error { message } => {
                    tracing::error!("nym-socks5-proxy reported an error: {message}");
                }
                Socks5ProcessEvent::Exited { success } => {
                    tracing::info!("nym-socks5-proxy exited: success={success}");
                }
            }
        }
    });
}
