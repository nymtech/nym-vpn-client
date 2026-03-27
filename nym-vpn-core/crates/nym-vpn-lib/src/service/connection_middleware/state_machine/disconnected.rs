// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Disconnected state is the default state.
//! It expresses the user’s desired state is to not be connected.

use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::service::connection_middleware::state_machine::{
    ConnectionMiddlewareCommand, ConnectionMiddlewareStateHandler, NextConnectionMiddlewareState,
    PrivateConnectionMiddlewareState, SharedState, connected::ConnectedState,
    connecting::ConnectingState,
};

pub struct DisconnectedState {}

impl DisconnectedState {
    pub fn enter() -> (
        Box<dyn ConnectionMiddlewareStateHandler>,
        PrivateConnectionMiddlewareState,
    ) {
        (
            Box::new(Self {}),
            PrivateConnectionMiddlewareState::Disconnected,
        )
    }
}

#[async_trait::async_trait]
impl ConnectionMiddlewareStateHandler for DisconnectedState {
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
                    ConnectionMiddlewareCommand::Start => NextConnectionMiddlewareState::NewState(ConnectingState::enter()),
                    ConnectionMiddlewareCommand::Stop => NextConnectionMiddlewareState::SameState(self),
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
