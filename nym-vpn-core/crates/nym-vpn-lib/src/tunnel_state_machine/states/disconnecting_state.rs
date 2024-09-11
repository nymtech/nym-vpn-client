use std::time::Duration;

use futures::future::{Fuse, FutureExt};
use talpid_routing::RouteManager;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use super::{
    super::{NextTunnelState, SharedState, TunnelCommand, TunnelState, TunnelStateHandler},
    DisconnectedState,
};
use crate::tunnel_state_machine::mixnet_route_handler::MixnetRouteHandler;

pub struct DisconnectingState {
    wait_handle: Fuse<JoinHandle<()>>,
}

impl DisconnectingState {
    pub fn enter() -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let wait_handle = tokio::spawn(async {
            // todo: disconnect the tunnel
            tokio::time::sleep(Duration::from_secs(1)).await;
        })
        .fuse();

        (Box::new(Self { wait_handle }), TunnelState::Disconnecting)
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
                shared_state.route_handler.remove_routes().await;
                NextTunnelState::NewState(DisconnectedState::enter())
            }
            _ = (&mut self.wait_handle) => {
                shared_state.route_handler.remove_routes().await;
                NextTunnelState::NewState(DisconnectedState::enter())
            },
            else => NextTunnelState::Finished
        }
    }
}
