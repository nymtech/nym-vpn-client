use std::time::Duration;

use futures::future::{Fuse, FutureExt};
use talpid_routing::RouteManager;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use super::{
    super::{NextTunnelState, SharedState, TunnelCommand, TunnelState, TunnelStateHandler},
    DisconnectedState,
};

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

    async fn clear_routes(route_manager: &mut RouteManager) {
        if let Err(e) = route_manager.clear_routes() {
            tracing::error!("Failed to clear routes: {}", e);
        }

        #[cfg(target_os = "linux")]
        if let Err(e) = route_manager.clear_routing_rules().await {
            tracing::error!("Failed to clear routes: {}", e);
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
                // todo: reset routing table etc..
                NextTunnelState::NewState(DisconnectedState::enter())
            }
            _ = (&mut self.wait_handle) => {
                // todo: reset routing table etc..
                NextTunnelState::NewState(DisconnectedState::enter())
            },
            else => NextTunnelState::Finished
        }
    }
}
