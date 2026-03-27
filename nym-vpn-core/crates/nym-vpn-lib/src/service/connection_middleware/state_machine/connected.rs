// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Connected state happens when the user clicked the connect button and the
//! tunnel state machine managed to obtain a connection.
//! It expresses the user’s desired target state is to be connected, which is
//! currently achieved.

use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::service::connection_middleware::state_machine::{
    ConnectionMiddlewareCommand, ConnectionMiddlewareStateHandler, NextConnectionMiddlewareState,
    PrivateConnectionMiddlewareState, SharedState, connecting::ConnectingState,
    disconnected::DisconnectedState,
};

pub struct ConnectedState {}

impl ConnectedState {
    pub fn enter() -> (
        Box<dyn ConnectionMiddlewareStateHandler>,
        PrivateConnectionMiddlewareState,
    ) {
        (
            Box::new(Self {}),
            PrivateConnectionMiddlewareState::Connected,
        )
    }
}

#[async_trait::async_trait]
impl ConnectionMiddlewareStateHandler for ConnectedState {
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
                            TunnelState::Disconnected | TunnelState::Disconnecting {..} | TunnelState::Connecting { .. } |  TunnelState::Error(_) | TunnelState::Offline {..} => NextConnectionMiddlewareState::NewState(ConnectingState::enter()),
                            TunnelState::Connected { .. } => NextConnectionMiddlewareState::SameState(self),
                        }
                    },
                    _ => NextConnectionMiddlewareState::SameState(self),
                }
            }
        }
    }
}
