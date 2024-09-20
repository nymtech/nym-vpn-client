use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::DisconnectingState, tunnel::mixnet::connected_tunnel::TunnelHandle,
    ActionAfterDisconnect, ErrorStateReason, NextTunnelState, SharedState, TunnelCommand,
    TunnelState, TunnelStateHandler,
};

pub struct ConnectedState {
    tunnel_handle: TunnelHandle,
}

impl ConnectedState {
    pub fn enter(
        tunnel_handle: TunnelHandle,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        (Box::new(Self { tunnel_handle }), TunnelState::Connected)
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
