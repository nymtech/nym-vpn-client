// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState},
    ErrorStateReason, NextTunnelState, SharedState, TunnelCommand, TunnelState, TunnelStateHandler,
};

pub struct ErrorState;

impl ErrorState {
    pub fn enter(reason: ErrorStateReason) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        (Box::new(Self), TunnelState::Error(reason))
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ErrorState {
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
                        NextTunnelState::NewState(ConnectingState::enter(shared_state).await)
                    },
                    TunnelCommand::Disconnect => NextTunnelState::NewState(DisconnectedState::enter()),
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
