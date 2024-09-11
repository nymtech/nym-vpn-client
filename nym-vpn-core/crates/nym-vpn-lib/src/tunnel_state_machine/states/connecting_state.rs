use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{
    super::{NextTunnelState, SharedState, TunnelCommand, TunnelState, TunnelStateHandler},
    DisconnectingState, ErrorState,
};
use crate::tunnel_state_machine::mixnet_tunnel::MixnetTunnel;

pub struct ConnectingState;

impl ConnectingState {
    pub async fn enter(
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let tunnel_shutdown_token = CancellationToken::new();

        match MixnetTunnel::spawn(
            shared_state.config.clone(),
            tunnel_shutdown_token.child_token(),
        )
        .await
        {
            Ok(tunnel_handle) => {
                shared_state.tunnel_shutdown_token = Some(tunnel_shutdown_token);
                shared_state.tunnel_handle = Some(tunnel_handle);

                (Box::new(Self), TunnelState::Connecting)
            }
            Err(e) => {
                tracing::error!("Failed to start the tunnel: {}", e);
                ErrorState::enter()
            }
        }
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
            _ = shutdown_token.cancelled() => {
                NextTunnelState::NewState(DisconnectingState::enter(shared_state))
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(shared_state))
                    },
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
