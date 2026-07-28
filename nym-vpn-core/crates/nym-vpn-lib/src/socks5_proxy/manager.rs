// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    mem::replace,
    net::{Ipv4Addr, Ipv6Addr},
};

#[cfg(target_os = "android")]
use nym_socks5_proxy::SocketProtector;

#[cfg(not(target_os = "android"))]
use super::process;

#[cfg(target_os = "android")]
use super::task;

use super::Socks5ProxyEvent;
use nym_socks5_proxy_ipc::{InterfaceAddresses, ProxyConfig};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

enum Socks5ProxyState {
    Stopped,
    Starting(CancellationToken),
    #[cfg(not(target_os = "android"))]
    RunningProcess(process::RunningProcess),
    #[cfg(target_os = "android")]
    RunningTask(task::RunningTask),
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

    pub async fn start(
        &mut self,
        config: ProxyConfig,
        #[cfg(target_os = "android")] socket_protector: SocketProtector,
        shutdown_token: CancellationToken,
    ) {
        match self.state {
            Socks5ProxyState::Starting(_) => {
                tracing::debug!("nym-socks5-proxy is already starting");
                return;
            }
            #[cfg(not(target_os = "android"))]
            Socks5ProxyState::RunningProcess(_) => {
                tracing::debug!("nym-socks5-proxy is already running");
                return;
            }
            #[cfg(target_os = "android")]
            Socks5ProxyState::RunningTask(_) => {
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

        let (event_tx, event_rx) = mpsc::unbounded_channel::<Socks5ProxyEvent>();

        #[cfg(not(target_os = "android"))]
        let spawn_result = process::spawn(config, event_tx, child_token)
            .await
            .map_err(|e| e.to_string());

        #[cfg(target_os = "android")]
        let spawn_result = task::spawn(config, socket_protector, event_tx, child_token)
            .await
            .map_err(|e| e.to_string());

        match spawn_result {
            Ok(running) => {
                spawn_event_logger(event_rx);
                running.set_tunnel_addrs(self.tunnel_addrs.clone());
                #[cfg(not(target_os = "android"))]
                {
                    self.state = Socks5ProxyState::RunningProcess(running);
                }
                #[cfg(target_os = "android")]
                {
                    self.state = Socks5ProxyState::RunningTask(running);
                }
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
            #[cfg(not(target_os = "android"))]
            Socks5ProxyState::RunningProcess(running) => {
                running.shutdown();
                running.join().await;
                self.state = Socks5ProxyState::Stopped;
            }
            #[cfg(target_os = "android")]
            Socks5ProxyState::RunningTask(running) => {
                running.shutdown();
                running.join().await;
                self.state = Socks5ProxyState::Stopped;
            }
        }
    }

    /// Notify the running proxy of an updated excluded-countries list. A no-op if the proxy
    /// isn't currently running — a fresh start already picks up the latest list via
    /// `ProxyConfig`.
    pub fn set_excluded_countries(&self, excluded_countries: Vec<String>) {
        #[cfg(not(target_os = "android"))]
        if let Socks5ProxyState::RunningProcess(running) = &self.state {
            tracing::debug!(
                "Notifying nym-socks5-proxy of updated excluded countries: {excluded_countries:?}",
            );
            running.set_excluded_countries(excluded_countries);
        }

        #[cfg(target_os = "android")]
        if let Socks5ProxyState::RunningTask(running) = &self.state {
            tracing::debug!(
                "Notifying nym-socks5-proxy of updated excluded countries: {excluded_countries:?}",
            );
            running.set_excluded_countries(excluded_countries);
        }
    }

    pub fn set_tunnel_addrs(&mut self, v4_addr: Option<Ipv4Addr>, v6_addr: Option<Ipv6Addr>) {
        self.tunnel_addrs.v4_addr = v4_addr;
        self.tunnel_addrs.v6_addr = v6_addr;

        #[cfg(not(target_os = "android"))]
        if let Socks5ProxyState::RunningProcess(running) = &self.state {
            tracing::debug!(
                "Notifying nym-socks5-proxy of new tunnel addresses: {:?}",
                self.tunnel_addrs
            );
            running.set_tunnel_addrs(self.tunnel_addrs.clone());
        }

        #[cfg(target_os = "android")]
        if let Socks5ProxyState::RunningTask(running) = &self.state {
            tracing::debug!(
                "Notifying nym-socks5-proxy of new tunnel addresses: {:?}",
                self.tunnel_addrs
            );
            running.set_tunnel_addrs(self.tunnel_addrs.clone());
        }
    }
}

fn spawn_event_logger(mut event_rx: mpsc::UnboundedReceiver<Socks5ProxyEvent>) {
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                Socks5ProxyEvent::Ready => {
                    tracing::info!("nym-socks5-proxy ready");
                }
                Socks5ProxyEvent::Error { message } => {
                    tracing::error!("nym-socks5-proxy reported an error: {message}");
                }
                Socks5ProxyEvent::Exited { success } => {
                    tracing::info!("nym-socks5-proxy exited: success={success}");
                }
            }
        }
    });
}
