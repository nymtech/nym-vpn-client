// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::future::{Fuse, FutureExt};
use tokio::{
    sync::mpsc,
    task::{JoinError, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use crate::tunnel_state_machine::dns_handler::DnsHandlerHandle;

use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState, ErrorState},
    tunnel_monitor::TunnelMonitorHandle,
    NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
};

type WaitHandle = JoinHandle<Option<Vec<AsyncDevice>>>;

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

        let wait_handle = tokio::spawn(async move {
            monitor_handle.cancel();
            monitor_handle.wait().await;
            // TODO: return async devices?
            Some(vec![])
        })
        .fuse();

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
                wait_handle,
            }),
            PrivateTunnelState::Disconnecting { after_disconnect },
        )
    }

    async fn on_tunnel_exit(
        result: Result<Option<Vec<AsyncDevice>>, JoinError>,
        _shared_state: &mut SharedState,
    ) {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        _shared_state.route_handler.remove_routes().await;

        match result {
            Ok(Some(tun_devices)) => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                if let Err(e) = _shared_state
                    .dns_handler
                    .reset_before_interface_removal()
                    .await
                {
                    tracing::error!("Failed to reset dns before interface removal: {}", e);
                }
                tracing::debug!("Closing tunnel {} device(s).", tun_devices.len());
                let _ = tun_devices;
            }
            Ok(None) => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                Self::reset_dns(&mut _shared_state.dns_handler).await;
                tracing::debug!("Tunnel device has already been closed.");
            }
            Err(e) => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                Self::reset_dns(&mut _shared_state.dns_handler).await;
                tracing::error!("Failed to join on tunnel handle: {}", e);
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        _shared_state.route_handler.remove_routes().await;
        // todo: reset firewall
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn reset_dns(dns_handler: &mut DnsHandlerHandle) {
        if let Err(e) = dns_handler.reset().await {
            tracing::error!("Failed to reset dns: {}", e);
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
        tokio::select! {
            _ = shutdown_token.cancelled() => {
                // Wait for tunnel to exit anyway because it's unsafe to drop the task manager.
                let result = self.wait_handle.await;
                Self::on_tunnel_exit(result, shared_state).await;

                NextTunnelState::NewState(DisconnectedState::enter())
            }
            result = (&mut self.wait_handle) => {
                Self::on_tunnel_exit(result, shared_state).await;

                match self.after_disconnect {
                    PrivateActionAfterDisconnect::Nothing => NextTunnelState::NewState(DisconnectedState::enter()),
                    PrivateActionAfterDisconnect::Error(reason) => {
                        NextTunnelState::NewState(ErrorState::enter(reason))
                    },
                    PrivateActionAfterDisconnect::Reconnect { retry_attempt } => {
                        NextTunnelState::NewState(ConnectingState::enter(retry_attempt, None, shared_state))
                    }
                }
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        self.after_disconnect = PrivateActionAfterDisconnect::Reconnect { retry_attempt: self.retry_attempt };
                    },
                    TunnelCommand::Disconnect => {
                        self.after_disconnect = PrivateActionAfterDisconnect::Nothing;
                    }
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                    }
                }
                NextTunnelState::SameState(self)
            }
            else => NextTunnelState::Finished
        }
    }
}
