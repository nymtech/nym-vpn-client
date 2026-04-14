// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::collections::HashSet;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
    states::{ConnectingState, OfflineState},
    tunnel::Tombstone,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_common::trace_err_chain;
use nym_http_api_client::HickoryDnsResolver;

pub struct DisconnectedState;

impl DisconnectedState {
    pub async fn enter(
        tombstone: Option<Tombstone>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Err(err) = shared_state.split_tunnel.reset_tunnel().await {
            trace_err_chain!(err, "failed to reset split tunnel");
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_dns(shared_state).await;

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_firewall_policy(shared_state);

        // Drop tombstone to close tunnel devices.
        drop(tombstone);

        // Clear addresses from the pre-resolve table in the (shared) DNS resolver.
        HickoryDnsResolver::shared().clear_preresolve();

        shared_state.allow_networking().await;

        (Box::new(Self), PrivateTunnelState::Disconnected)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn reset_firewall_policy(shared_state: &mut SharedState) {
        if let Err(e) = shared_state.firewall.reset_policy() {
            trace_err_chain!(e, "Failed to reset firewall policy");
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(error) = shared_state.dns_handler.reset().await {
            trace_err_chain!(error, "Failed to reset DNS");
        }
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for DisconnectedState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                tracing::debug!("DisconnectedState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => {
                        NextTunnelState::NewState(ConnectingState::enter(0, None, shared_state).await)
                    },
                    TunnelCommand::Disconnect => NextTunnelState::SameState(self),
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        if shared_state.tunnel_settings.diff(&tunnel_settings).is_some_and(|diff| diff.split_tunnel_changed()) {
                            let _ = shared_state.set_exclude_paths(tunnel_settings.split_tunnel.effective_app_paths(), HashSet::new()).await;
                        }

                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                    TunnelCommand::Block(_reason) => {
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(OfflineState::enter(false, None, shared_state).await)
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                Self::reset_dns(shared_state).await;
                NextTunnelState::Finished
            }
        }
    }
}
