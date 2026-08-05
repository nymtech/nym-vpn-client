// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::future::{BoxFuture, Fuse, FusedFuture, FutureExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "android")]
use nym_vpn_lib_types::ErrorStateReason;

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
    pub async fn enter(
        after_disconnect: PrivateActionAfterDisconnect,
        tunnel_monitor_handle: Option<TunnelMonitorHandle>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        // Disallow networking when disconnecting to prevent new connections during tunnel shutdown
        shared_state.disallow_networking().await;

        // It's safe to abort status listener as it's stateless.
        if let Some(status_listener_handle) = shared_state.status_listener_handle.take() {
            status_listener_handle.abort();
        }

        if let Some(tunnel_monitor_handle) = tunnel_monitor_handle {
            tunnel_monitor_handle.cancel();

            (
                Box::new(Self {
                    after_disconnect: after_disconnect.clone(),
                    tunnel_wait_handle: tunnel_monitor_handle.wait().boxed().fuse(),
                }) as _,
                PrivateTunnelState::Disconnecting { after_disconnect },
            )
        } else {
            Self::on_disconnect(after_disconnect, None, shared_state).await
        }
    }

    async fn on_disconnect(
        after_disconnect: PrivateActionAfterDisconnect,
        tombstone: Option<Tombstone>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        match after_disconnect {
            PrivateActionAfterDisconnect::Nothing => {
                DisconnectedState::enter(tombstone, shared_state).await
            }
            PrivateActionAfterDisconnect::Error(reason) => {
                #[cfg(target_os = "android")]
                if let Err(err) = shared_state.prepare_blocking_cover_before_release(tombstone) {
                    tracing::error!(
                        "Failed to install Android blocking TUN before error state: {err}"
                    );
                }
                #[cfg(not(target_os = "android"))]
                {
                    let _ = tombstone;
                }
                ErrorState::enter(reason, shared_state).await
            }
            PrivateActionAfterDisconnect::Reconnect => {
                #[cfg(target_os = "android")]
                if let Err(err) = shared_state.prepare_blocking_cover_before_release(tombstone) {
                    tracing::error!(
                        "Failed to install Android blocking TUN before reconnect: {err}"
                    );
                    return ErrorState::enter(ErrorStateReason::TunnelProvider, shared_state).await;
                }
                #[cfg(not(target_os = "android"))]
                {
                    let _ = tombstone;
                }
                ConnectingState::enter(0, None, shared_state).await
            }
            PrivateActionAfterDisconnect::Offline {
                reconnect,
                gateways,
            } => OfflineState::enter(reconnect, gateways, shared_state).await,
        }
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
                NextTunnelState::NewState(Self::on_disconnect(self.after_disconnect, Some(tombstone), shared_state).await)
            }
            Some(command) = command_rx.recv() => {
                tracing::debug!("DisconnectingState received command: {command:?}");
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
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        let diff = shared_state.tunnel_settings.diff(&tunnel_settings);
                        if diff.is_empty() {
                            return NextTunnelState::SameState(self);
                        }

                        shared_state.set_tunnel_settings(tunnel_settings).await;

                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        if diff.split_tunnel_changed() || diff.geo_exclusion_enabled_changed() {
                            let _ = shared_state.set_split_tunnel_exclude_paths().await;
                        }

                        #[cfg(not(target_os = "ios"))]
                        if diff.geo_exclusion_enabled_changed() {
                            shared_state.start_or_stop_socks5_proxy().await;
                        } else if diff.geo_exclusion_excluded_countries_changed() {
                            shared_state.set_socks5_proxy_excluded_countries();
                        }

                        if diff.enable_ad_blocking_changed() {
                            shared_state.enable_ad_blocking(shared_state.tunnel_settings.enable_ad_blocking).await;
                        }

                        NextTunnelState::SameState(self)
                    }
                    TunnelCommand::Block(reason) => {
                        if !matches!(self.after_disconnect, PrivateActionAfterDisconnect::Nothing) {
                            self.after_disconnect = PrivateActionAfterDisconnect::Error(reason);
                        }
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
