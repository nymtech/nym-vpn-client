use futures::future::{Fuse, FutureExt};
use tokio::{
    sync::mpsc,
    task::{JoinError, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

use crate::tunnel_state_machine::{
    dns_handler::DnsHandler,
    states::{ConnectingState, DisconnectedState, ErrorState},
    tunnel::any_tunnel_handle::AnyTunnelHandle,
    ActionAfterDisconnect, NextTunnelState, SharedState, TunnelCommand, TunnelState,
    TunnelStateHandler,
};

type WaitHandle = JoinHandle<Option<Vec<AsyncDevice>>>;

pub struct DisconnectingState {
    after_disconnect: ActionAfterDisconnect,
    wait_handle: Fuse<WaitHandle>,
}

impl DisconnectingState {
    pub fn enter(
        after_disconnect: ActionAfterDisconnect,
        mut tunnel_handle: Option<AnyTunnelHandle>,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        if let Some(tunnel_handle) = tunnel_handle.as_mut() {
            tunnel_handle.cancel();
        }

        let wait_handle = tokio::spawn(async move {
            tunnel_handle?
                .wait()
                .await
                .inspect_err(|e| {
                    tracing::error!("Tunnel exited with error: {}", e);
                })
                .ok()
        })
        .fuse();

        (
            Box::new(Self {
                after_disconnect,
                wait_handle,
            }),
            TunnelState::Disconnecting { after_disconnect },
        )
    }

    async fn on_tunnel_exit(
        &self,
        result: Result<Option<Vec<AsyncDevice>>, JoinError>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        shared_state.route_handler.remove_routes().await;

        match result {
            Ok(Some(tun_devices)) => {
                if let Err(e) = shared_state.dns_handler.reset_before_interface_removal() {
                    tracing::error!("Failed to reset dns before interface removal: {}", e);
                }
                tracing::debug!("Closing tunnel {} device(s).", tun_devices.len());
                let _ = tun_devices;
            }
            Ok(None) => {
                Self::reset_dns(&mut shared_state.dns_handler);
                tracing::debug!("Tunnel device has already been closed.");
            }
            Err(e) => {
                Self::reset_dns(&mut shared_state.dns_handler);
                tracing::error!("Failed to join on tunnel handle: {}", e);
            }
        }

        shared_state.route_handler.remove_routes().await;
        // todo: reset firewall

        match self.after_disconnect {
            ActionAfterDisconnect::Nothing => NextTunnelState::NewState(DisconnectedState::enter()),
            ActionAfterDisconnect::Error(reason) => {
                NextTunnelState::NewState(ErrorState::enter(reason))
            }
            ActionAfterDisconnect::Reconnect => {
                NextTunnelState::NewState(ConnectingState::enter(shared_state))
            }
        }
    }

    fn reset_dns(dns_handler: &mut DnsHandler) {
        if let Err(e) = dns_handler.reset() {
            tracing::error!("Failed to reset dns: {}", e);
        }
    }

    async fn reset_on_cancel(shared_state: &mut SharedState) {
        Self::reset_dns(&mut shared_state.dns_handler);
        shared_state.route_handler.remove_routes().await;
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
            _ = shutdown_token.cancelled() => {
                Self::reset_on_cancel(shared_state).await;
                NextTunnelState::NewState(DisconnectedState::enter())
            }
            result = (&mut self.wait_handle) => {
                self.on_tunnel_exit(result, shared_state).await
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        self.after_disconnect = ActionAfterDisconnect::Reconnect;
                    },
                    TunnelCommand::Disconnect => {
                        self.after_disconnect = ActionAfterDisconnect::Nothing;
                    }
                }
                NextTunnelState::SameState(self)
            }
            else => NextTunnelState::Finished
        }
    }
}
