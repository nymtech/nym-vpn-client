use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    mixnet_tunnel, states::DisconnectingState, ActionAfterDisconnect, ErrorStateReason,
    NextTunnelState, SharedState, TunnelCommand, TunnelState, TunnelStateHandler,
};

pub struct ConnectedState {
    tun_event_rx: mixnet_tunnel::EventReceiver,
}

impl ConnectedState {
    pub fn enter(
        tun_event_rx: mixnet_tunnel::EventReceiver,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        (Box::new(Self { tun_event_rx }), TunnelState::Connected)
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
                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, shared_state))
            }
            Some(event) = self.tun_event_rx.recv() => {
                match event {
                    mixnet_tunnel::Event::Up { .. } => {
                        tracing::warn!("Received tunnel up event which must not happen!");
                        NextTunnelState::SameState(self)
                    },
                    mixnet_tunnel::Event::Down(error) => {
                        if let Some(error) = error {
                            tracing::error!("Tunnel went down with error: {}", error)
                        } else {
                            tracing::info!("Tunnel went down");
                        }
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Error(ErrorStateReason::TunnelDown), shared_state))
                    }
                }
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, shared_state))
                    },
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
