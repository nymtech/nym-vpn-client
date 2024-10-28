// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::DisconnectingState, tunnel::any_tunnel_handle::AnyTunnelHandle, ActionAfterDisconnect,
    ConnectionData, ErrorStateReason, NextTunnelState, SharedState, TunnelCommand, TunnelState,
    TunnelStateHandler,
};

pub struct ConnectedState {
    tunnel_handle: AnyTunnelHandle,
}

impl ConnectedState {
    pub fn enter(
        connection_data: ConnectionData,
        tunnel_handle: AnyTunnelHandle,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        (
            Box::new(Self { tunnel_handle }),
            TunnelState::Connected { connection_data },
        )
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ConnectedState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            _ = shutdown_token.cancelled() => {
                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, Some(self.tunnel_handle), shared_state))
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, Some(self.tunnel_handle) , shared_state))
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            shared_state.tunnel_settings = tunnel_settings;
                            NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Reconnect, Some(self.tunnel_handle), shared_state))
                        }
                    }
                }
            }
            maybe_error = self.tunnel_handle.recv_error() => {
                match maybe_error {
                    Some(error) => {
                        tracing::error!("Tunnel error: {}", error);
                        // todo: handle error
                        NextTunnelState::SameState(self)
                    }
                    None => {
                        tracing::info!("Tunnel went down unexpectedly.");
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Error(ErrorStateReason::TunnelDown), Some(self.tunnel_handle), shared_state))
                    }
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
