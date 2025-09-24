// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::future::{BoxFuture, Fuse, FusedFuture, FutureExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
    states::{ConnectingState, DisconnectedState, ErrorState, OfflineState},
    tunnel::Tombstone,
    tunnel_monitor::TunnelMonitorHandle,
};

type WaitHandle = BoxFuture<'static, Tombstone>;

pub struct DisconnectingState {
    after_disconnect: PrivateActionAfterDisconnect,
    tunnel_wait_handle: Fuse<WaitHandle>,
}

impl DisconnectingState {
    pub fn enter(
        after_disconnect: PrivateActionAfterDisconnect,
        tunnel_monitor_handle: TunnelMonitorHandle,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        // It's safe to abort status listener as it's stateless.
        if let Some(status_listener_handle) = shared_state.status_listener_handle.take() {
            status_listener_handle.abort();
        }
        tunnel_monitor_handle.cancel();

        (
            Box::new(Self {
                after_disconnect: after_disconnect.clone(),
                tunnel_wait_handle: tunnel_monitor_handle.wait().boxed().fuse(),
            }),
            PrivateTunnelState::Disconnecting { after_disconnect },
        )
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for DisconnectingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        // Precautionary escape hatch, even though this is unlikely to ever evaluate to true
        if self.tunnel_wait_handle.is_terminated() {
            return NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await);
        }

        tokio::select! {
            tombstone = (&mut self.tunnel_wait_handle) => {
                match self.after_disconnect {
                    PrivateActionAfterDisconnect::Nothing => NextTunnelState::NewState(DisconnectedState::enter(Some(tombstone), shared_state).await),
                    PrivateActionAfterDisconnect::Error(reason) => {
                        NextTunnelState::NewState(ErrorState::enter(reason, shared_state).await)
                    },
                    PrivateActionAfterDisconnect::Reconnect => {
                        NextTunnelState::NewState(ConnectingState::enter(0, None, shared_state).await)
                    },
                    PrivateActionAfterDisconnect::Offline { reconnect, gateways } => {
                        NextTunnelState::NewState(OfflineState::enter(reconnect, gateways, shared_state).await)
                    }
                }
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        self.after_disconnect = match self.after_disconnect {
                            PrivateActionAfterDisconnect::Offline { gateways,  .. } => {
                                PrivateActionAfterDisconnect::Offline { reconnect: true, gateways }
                            }
                            _ => PrivateActionAfterDisconnect::Reconnect,
                        };
                        NextTunnelState::SameState(self)
                    },
                    TunnelCommand::Disconnect => {
                        self.after_disconnect = match self.after_disconnect {
                            PrivateActionAfterDisconnect::Offline { gateways, .. } => {
                                PrivateActionAfterDisconnect::Offline { reconnect: false, gateways }
                            }
                            _ => PrivateActionAfterDisconnect::Nothing
                        };
                        NextTunnelState::SameState(self)
                    }
                    TunnelCommand::SetAllowLan(allow_lan, complete_tx) => {
                        _ = shared_state.set_allow_lan(allow_lan);
                        _ = complete_tx.send(());
                        NextTunnelState::SameState(self)
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                let tombstone = if self.tunnel_wait_handle.is_terminated() {
                    None
                } else {
                    // Wait for tunnel to exit anyway because it's unsafe to drop the task manager.
                    Some(self.tunnel_wait_handle.await)
                };

                NextTunnelState::NewState(DisconnectedState::enter(tombstone, shared_state).await)
            }
        }
    }
}
