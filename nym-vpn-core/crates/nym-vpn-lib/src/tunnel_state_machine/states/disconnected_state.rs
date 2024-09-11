use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::ConnectingState, NextTunnelState, SharedState, TunnelCommand, TunnelState,
    TunnelStateHandler,
};

pub struct DisconnectedState;

impl DisconnectedState {
    pub fn enter() -> (Box<dyn TunnelStateHandler>, TunnelState) {
        (Box::new(Self), TunnelState::Disconnected)
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
            _ = shutdown_token.cancelled() => {
                NextTunnelState::Finished
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        NextTunnelState::NewState(ConnectingState::enter(shared_state).await)
                    },
                    TunnelCommand::Disconnect => NextTunnelState::SameState(self),
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
