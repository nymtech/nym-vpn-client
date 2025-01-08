// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    tunnel::SelectedGateways, tunnel_monitor::TunnelMonitorHandle, NextTunnelState,
    PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
};

pub struct OfflineState {
    // todo: store last used gateway and reconnect to it!
    selected_gateways: Option<SelectedGateways>,
    /// Whether to connect the tunnel upon gaining the network connectivity.
    reconnect: bool,
    // todo: wait for handle before reconnecting
    monitor_handle: TunnelMonitorHandle,
}

impl OfflineState {
    pub fn enter(
        monitor_handle: TunnelMonitorHandle,
        selected_gateways: Option<SelectedGateways>,
        reconnect: bool,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        // It's safe to abort status listener as it's stateless.
        if let Some(status_listener_handle) = shared_state.status_listener_handle.take() {
            status_listener_handle.abort();
        }
        monitor_handle.cancel();

        (
            Box::new(Self {
                reconnect,
                monitor_handle,
                selected_gateways,
            }),
            PrivateTunnelState::Offline { reconnect },
        )
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for OfflineState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            _ = shutdown_token.cancelled() => {
                NextTunnelState::Finished
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        if self.reconnect {
                            NextTunnelState::SameState(self)
                        } else {
                            self.reconnect = true;
                            let new_state = PrivateTunnelState::Offline { reconnect: self.reconnect };
                            NextTunnelState::NewState((self, new_state))
                        }
                    },
                    TunnelCommand::Disconnect => {
                        if self.reconnect {
                            self.reconnect = false;
                            let new_state = PrivateTunnelState::Offline { reconnect: self.reconnect };
                            NextTunnelState::NewState((self, new_state))
                        } else {
                            NextTunnelState::SameState(self)
                        }
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
