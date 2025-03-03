// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState},
    tunnel::SelectedGateways,
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
};

pub struct OfflineState {
    /// Whether to connect the tunnel once online
    reconnect: bool,

    /// Last known retry attempt before entering offline state.
    retry_attempt: u32,

    /// Gateways to which the tunnel will reconnect to once online
    selected_gateways: Option<SelectedGateways>,
}

impl OfflineState {
    pub fn enter(
        reconnect: bool,
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        (
            Box::new(Self {
                reconnect,
                retry_attempt,
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
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::SameState(self)
                } else if self.reconnect {
                    NextTunnelState::NewState(ConnectingState::enter(
                        self.retry_attempt,
                        self.selected_gateways,
                        shared_state
                    ).await)
                } else {
                    NextTunnelState::NewState(DisconnectedState::enter(shared_state).await)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextTunnelState::Finished
            }
            else => NextTunnelState::Finished
        }
    }
}
