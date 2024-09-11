use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    mixnet_tunnel::{self, MixnetTunnel},
    states::{ConnectedState, DisconnectingState},
    ActionAfterDisconnect, ErrorStateReason, NextTunnelState, SharedState, TunnelCommand,
    TunnelState, TunnelStateHandler,
};

pub struct ConnectingState {
    tun_event_rx: mixnet_tunnel::EventReceiver,
}

impl ConnectingState {
    pub async fn enter(
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let tunnel_shutdown_token = CancellationToken::new();
        let shutdown_child_token = tunnel_shutdown_token.child_token();
        let (tun_event_tx, tun_event_rx) = mpsc::unbounded_channel();
        shared_state.tunnel_shutdown_token = Some(tunnel_shutdown_token);
        shared_state.tunnel_handle = Some(MixnetTunnel::spawn(
            shared_state.config.clone(),
            tun_event_tx,
            shutdown_child_token,
        ));
        (Box::new(Self { tun_event_rx }), TunnelState::Connecting)
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
                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, shared_state))
            }
            Some(event) = self.tun_event_rx.recv() => {
                match event {
                    mixnet_tunnel::Event::Up {  entry_mixnet_gateway_ip, tun_name } => {
                        match shared_state.route_handler.add_routes(tun_name, entry_mixnet_gateway_ip).await {
                            Ok(()) => NextTunnelState::NewState(ConnectedState::enter(self.tun_event_rx)),
                            Err(e) => {
                                tracing::error!("Failed to add routes: {}", e);
                                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Error(ErrorStateReason::Routing), shared_state))
                            }
                        }
                    },
                    mixnet_tunnel::Event::Down(error) => {
                        if let Some(error) = error {
                            tracing::error!("Tunnel went down: {}", error);
                        } else {
                            tracing::error!("Tunnel went down without error.");
                        }
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Error(ErrorStateReason::EstablishConnection), shared_state))
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
