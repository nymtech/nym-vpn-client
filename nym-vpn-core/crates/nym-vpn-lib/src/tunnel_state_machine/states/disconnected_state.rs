// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
use nym_common::ErrorExt;

use crate::tunnel_state_machine::{
    states::{ConnectingState, OfflineState},
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
};

pub struct DisconnectedState;

impl DisconnectedState {
    pub async fn enter(
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(target_os = "macos")]
        if let Err(error) = _shared_state.dns_handler.reset().await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Unable to disable filtering resolver")
            );
        }

        (Box::new(Self), PrivateTunnelState::Disconnected)
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
                match command {
                    TunnelCommand::Connect => {
                        NextTunnelState::NewState(ConnectingState::enter(0, None, shared_state).await)
                    },
                    TunnelCommand::Disconnect => NextTunnelState::SameState(self),
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(OfflineState::enter(false, 0, None))
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextTunnelState::Finished
            }
            else => NextTunnelState::Finished
        }
    }
}
