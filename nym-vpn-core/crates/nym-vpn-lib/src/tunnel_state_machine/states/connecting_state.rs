// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::{ConnectedState, DisconnectingState, OfflineState},
    tunnel::{SelectedGateways, Tombstone},
    tunnel_monitor::{
        TunnelMonitor, TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorHandle,
    },
    NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
};

pub struct ConnectingState {
    monitor_handle: TunnelMonitorHandle,
    monitor_event_receiver: TunnelMonitorEventReceiver,
    retry_attempt: u32,
    selected_gateways: Option<SelectedGateways>,
}

impl ConnectingState {
    pub async fn enter(
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        if shared_state
            .offline_monitor
            .connectivity()
            .await
            .is_offline()
        {
            // FIXME: Temporary: Nudge route manager to update the default interface
            #[cfg(target_os = "macos")]
            {
                log::debug!("Poking route manager to update default routes");
                shared_state.route_handler.refresh_routes().await;
            }
            return OfflineState::enter(true, retry_attempt, selected_gateways);
        }

        let (monitor_event_sender, monitor_event_receiver) = mpsc::unbounded_channel();
        let monitor_handle = TunnelMonitor::start(
            retry_attempt,
            selected_gateways.clone(),
            monitor_event_sender,
            shared_state.mixnet_event_sender.clone(),
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            shared_state.route_handler.clone(),
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            shared_state.dns_handler.clone(),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            shared_state.tun_provider.clone(),
            shared_state.nym_config.clone(),
            shared_state.tunnel_settings.clone(),
        );

        (
            Box::new(Self {
                monitor_handle,
                monitor_event_receiver,
                retry_attempt,
                selected_gateways,
            }),
            PrivateTunnelState::Connecting {
                connection_data: None,
            },
        )
    }

    async fn on_tunnel_exit(mut tombstone: Tombstone, _shared_state: &mut SharedState) {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            if let Err(e) = _shared_state
                .dns_handler
                .reset_before_interface_removal()
                .await
            {
                tracing::error!("Failed to reset dns before interface removal: {}", e);
            }
            _shared_state.route_handler.remove_routes().await;
        }
        #[cfg(windows)]
        tombstone.wg_instances.clear();
        tombstone.tun_devices.clear();
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ConnectingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
           Some(monitor_event) = self.monitor_event_receiver.recv() => {
            match monitor_event {
                TunnelMonitorEvent::InitializingClient => {
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::EstablishingTunnel(conn_data) => {
                    NextTunnelState::NewState((self, PrivateTunnelState::Connecting { connection_data: Some(*conn_data) }))
                }
                TunnelMonitorEvent::SelectedGateways(new_gateways) => {
                    self.selected_gateways = Some(*new_gateways);
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::Up(conn_data) => {
                    NextTunnelState::NewState(ConnectedState::enter(
                        *conn_data,
                        self.selected_gateways.expect("selected gateways must be set"),
                        self.monitor_handle,
                        self.monitor_event_receiver,
                    ))
                }
                TunnelMonitorEvent::Down(reason) => {
                    if let Some(reason) = reason {
                        NextTunnelState::NewState(DisconnectingState::enter(
                            PrivateActionAfterDisconnect::Error(reason),
                            self.monitor_handle,
                            shared_state
                        ))
                    } else {
                        let tombstone = self.monitor_handle.wait().await;
                        Self::on_tunnel_exit(tombstone, shared_state).await;

                        NextTunnelState::NewState(ConnectingState::enter(
                            self.retry_attempt.saturating_add(1),
                            self.selected_gateways,
                            shared_state
                        ).await)
                    }
                }
            }
           }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(
                            PrivateActionAfterDisconnect::Nothing,
                            self.monitor_handle,
                            shared_state,
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
                                shared_state,
                            ))
                        }
                    }
                }
            }
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(DisconnectingState::enter(
                        PrivateActionAfterDisconnect::Offline {
                            reconnect: true,
                            retry_attempt: self.retry_attempt,
                            gateways: self.selected_gateways
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
                    shared_state,
                ))
            }
            else => NextTunnelState::Finished
        }
    }
}
