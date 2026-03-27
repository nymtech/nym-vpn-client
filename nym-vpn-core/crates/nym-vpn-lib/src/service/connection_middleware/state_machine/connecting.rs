// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Connecting state happens when the user clicks connect button.
//! It expresses the user’s desired target state is to be connected, which is
//! not achieved yet.

use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::service::connection_middleware::state_machine::{
    ConnectionMiddlewareCommand, ConnectionMiddlewareStateHandler, NextConnectionMiddlewareState,
    PrivateConnectionMiddlewareState, SharedState, connected::ConnectedState,
    disconnected::DisconnectedState,
};

pub struct ConnectingState {}

impl ConnectingState {
    pub fn enter() -> (
        Box<dyn ConnectionMiddlewareStateHandler>,
        PrivateConnectionMiddlewareState,
    ) {
        (
            Box::new(Self {}),
            PrivateConnectionMiddlewareState::Connecting,
        )
    }
}

#[async_trait::async_trait]
impl ConnectionMiddlewareStateHandler for ConnectingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<ConnectionMiddlewareCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextConnectionMiddlewareState {
        tokio::select! {
            biased;
            _ = shutdown_token.cancelled() => {
                NextConnectionMiddlewareState::Finished
            }
            Some(command) = command_rx.recv() => {
                match command {
                    ConnectionMiddlewareCommand::Start => NextConnectionMiddlewareState::SameState(self),
                    ConnectionMiddlewareCommand::Stop => NextConnectionMiddlewareState::NewState(DisconnectedState::enter()),
                }
            }
            Ok(event) = shared_state.tunnel_event_rx.recv() => {
                match event {
                    TunnelEvent::NewState(tunnel_state) => {
                        match tunnel_state {
                            TunnelState::Disconnected | TunnelState::Disconnecting {..} | TunnelState::Connecting { .. } |  TunnelState::Error(_) | TunnelState::Offline {..} => NextConnectionMiddlewareState::SameState(self),
                            TunnelState::Connected { .. } => NextConnectionMiddlewareState::NewState(ConnectedState::enter()),
                        }
                    },
                    _ => NextConnectionMiddlewareState::SameState(self),
                }
            }
        }
    }
}
