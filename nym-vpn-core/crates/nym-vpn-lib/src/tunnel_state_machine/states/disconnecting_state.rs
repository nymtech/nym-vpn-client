// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::future::{BoxFuture, Fuse, FutureExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState, ErrorState, OfflineState},
    tunnel::Tombstone,
    tunnel_monitor::TunnelMonitorHandle,
    NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
};

type WaitHandle = BoxFuture<'static, Tombstone>;

pub struct DisconnectingState {
    after_disconnect: PrivateActionAfterDisconnect,
    retry_attempt: u32,
    wait_handle: Fuse<WaitHandle>,
}

impl DisconnectingState {
    pub fn enter(
        after_disconnect: PrivateActionAfterDisconnect,
        monitor_handle: TunnelMonitorHandle,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        // It's safe to abort status listener as it's stateless.
        if let Some(status_listener_handle) = shared_state.status_listener_handle.take() {
            status_listener_handle.abort();
        }
        monitor_handle.cancel();

        let retry_attempt =
            if let PrivateActionAfterDisconnect::Reconnect { retry_attempt } = &after_disconnect {
                *retry_attempt
            } else {
                0
            };

        (
            Box::new(Self {
                after_disconnect: after_disconnect.clone(),
                retry_attempt,
                wait_handle: monitor_handle.wait().boxed().fuse(),
            }),
            PrivateTunnelState::Disconnecting { after_disconnect },
        )
    }

    async fn on_tunnel_exit(mut tombstone: Tombstone, _shared_state: &mut SharedState) {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        if let Err(e) = _shared_state
            .dns_handler
            .reset_before_interface_removal()
            .await
        {
            tracing::error!("Failed to reset dns before interface removal: {}", e);
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        _shared_state.route_handler.remove_routes().await;

        tracing::info!("Closing {} tunnel device(s).", tombstone.tun_devices.len());
        #[cfg(windows)]
        tombstone.wg_instances.clear();
        tombstone.tun_devices.clear();

        // todo: reset firewall
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
        tokio::select! {
            result = (&mut self.wait_handle) => {
                Self::on_tunnel_exit(result, shared_state).await;

                match self.after_disconnect {
                    PrivateActionAfterDisconnect::Nothing => NextTunnelState::NewState(DisconnectedState::enter(shared_state).await),
                    PrivateActionAfterDisconnect::Error(reason) => {
                        NextTunnelState::NewState(ErrorState::enter(reason, shared_state).await)
                    },
                    PrivateActionAfterDisconnect::Reconnect { retry_attempt } => {
                        NextTunnelState::NewState(ConnectingState::enter(retry_attempt, None, shared_state).await)
                    },
                    PrivateActionAfterDisconnect::Offline { reconnect, retry_attempt, gateways } => {
                        NextTunnelState::NewState(OfflineState::enter(reconnect, retry_attempt, gateways))
                    }
                }
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        self.after_disconnect = match self.after_disconnect {
                            PrivateActionAfterDisconnect::Offline { retry_attempt, gateways,  .. } => {
                                PrivateActionAfterDisconnect::Offline { reconnect: true, retry_attempt, gateways }
                            }
                            _ => PrivateActionAfterDisconnect::Reconnect { retry_attempt: self.retry_attempt },
                        };
                    },
                    TunnelCommand::Disconnect => {
                        self.after_disconnect = match self.after_disconnect {
                            PrivateActionAfterDisconnect::Offline { retry_attempt, gateways, .. } => {
                                PrivateActionAfterDisconnect::Offline { reconnect: false,retry_attempt,  gateways }
                            }
                            _ => PrivateActionAfterDisconnect::Nothing
                        };
                    }
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                    }
                }
                NextTunnelState::SameState(self)
            }
            _ = shutdown_token.cancelled() => {
                // Wait for tunnel to exit anyway because it's unsafe to drop the task manager.
                let result = self.wait_handle.await;
                Self::on_tunnel_exit(result, shared_state).await;

                NextTunnelState::NewState(DisconnectedState::enter(shared_state).await)
            }
            else => NextTunnelState::Finished
        }
    }
}
