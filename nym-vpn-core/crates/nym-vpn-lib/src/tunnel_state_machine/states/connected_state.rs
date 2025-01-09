// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::DisconnectingState,
    tunnel::SelectedGateways,
    tunnel_monitor::{TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorHandle},
    ConnectionData, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState,
    TunnelCommand, TunnelStateHandler,
};

pub struct ConnectedState {
    monitor_handle: TunnelMonitorHandle,
    monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: SelectedGateways,
}

impl ConnectedState {
    pub fn enter(
        connection_data: ConnectionData,
        selected_gateways: SelectedGateways,
        monitor_handle: TunnelMonitorHandle,
        monitor_event_receiver: TunnelMonitorEventReceiver,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        (
            Box::new(Self {
                monitor_handle,
                monitor_event_receiver,
                selected_gateways,
            }),
            PrivateTunnelState::Connected { connection_data },
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
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(
                            PrivateActionAfterDisconnect::Nothing,
                            self.monitor_handle,
                            shared_state
                        ))
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            shared_state.tunnel_settings = tunnel_settings;
                            NextTunnelState::NewState(DisconnectingState::enter(
                                PrivateActionAfterDisconnect::Reconnect { retry_attempt: 0 },
                                self.monitor_handle,
                                shared_state
                            ))
                        }
                    }
                }
            }
            Some(monitor_event) = self.monitor_event_receiver.recv() => {
                match monitor_event {
                    TunnelMonitorEvent::Down(reason) => {
                        let after_disconnect = reason.map(PrivateActionAfterDisconnect::Error)
                            .unwrap_or(PrivateActionAfterDisconnect::Reconnect { retry_attempt: 0 });

                        NextTunnelState::NewState(DisconnectingState::enter(after_disconnect, self.monitor_handle, shared_state))
                    }
                    _ => {
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(DisconnectingState::enter(
                        PrivateActionAfterDisconnect::Offline {
                            reconnect: true,
                            retry_attempt: 0,
                            gateways: Some(self.selected_gateways)
                        },
                        self.monitor_handle,
                        shared_state
                    ))
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextTunnelState::NewState(DisconnectingState::enter(
                    PrivateActionAfterDisconnect::Nothing,
                    self.monitor_handle,
                    shared_state
                ))
            }
            else => NextTunnelState::Finished
        }
    }
}
