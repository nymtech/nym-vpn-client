use std::time::Duration;

use futures::{
    future::{BoxFuture, Fuse},
    FutureExt,
};

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::{ConnectedState, DisconnectingState},
    tunnel::{self, ConnectedMixnet},
    ActionAfterDisconnect, ErrorStateReason, NextTunnelState, SharedState, TunnelCommand,
    TunnelState, TunnelStateHandler,
};

type ConnectFut = BoxFuture<'static, tunnel::Result<ConnectedMixnet>>;

pub struct ConnectingState {
    connect_fut: Fuse<ConnectFut>,
}

impl ConnectingState {
    pub fn enter(shared_state: &mut SharedState) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let config = shared_state.config.clone();

        let connect_fut = tunnel::connect_mixnet(config, false).boxed().fuse();

        (Box::new(Self { connect_fut }), TunnelState::Connecting)
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
            result = &mut self.connect_fut => {
                match result {
                    Ok(connected_mixnet) => {
                        match tunnel::run_mixnet_tunnel(shared_state.config.clone(), connected_mixnet).await {
                            Ok(tunnel_handle) => {
                                let old_tunnel_handle = shared_state.tunnel_handle.replace(tunnel_handle);
                                assert!(old_tunnel_handle.is_none());

                                // todo: remain in connecting state until tunnel connection is verified.
                                NextTunnelState::NewState(ConnectedState::enter(shared_state))
                            }
                            Err(e) => {
                                // todo: map error to error state reason
                                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Error(ErrorStateReason::EstablishMixnetConnection), shared_state))
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to connect mixnet: {}", e);
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Error(ErrorStateReason::EstablishMixnetConnection), shared_state))
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
