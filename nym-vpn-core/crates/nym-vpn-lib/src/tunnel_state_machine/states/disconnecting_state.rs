use futures::future::{Fuse, FutureExt};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState, ErrorState},
    ActionAfterDisconnect, NextTunnelState, SharedState, TunnelCommand, TunnelState,
    TunnelStateHandler,
};

pub struct DisconnectingState {
    after_disconnect: ActionAfterDisconnect,
    wait_handle: Fuse<JoinHandle<()>>,
}

impl DisconnectingState {
    pub fn enter(
        after_disconnect: ActionAfterDisconnect,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        if let Some(token) = shared_state.tunnel_shutdown_token.take() {
            token.cancel();
        }

        let tunnel_handle = shared_state.tunnel_handle.take();
        let wait_handle = tokio::spawn(async move {
            if let Some(tunnel_handle) = tunnel_handle {
                _ = tunnel_handle.await;
            }
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

                match self.after_disconnect {
                    ActionAfterDisconnect::Nothing => NextTunnelState::NewState(DisconnectedState::enter()),
                    ActionAfterDisconnect::Error(reason) => NextTunnelState::NewState(ErrorState::enter(reason)),
                    ActionAfterDisconnect::Reconnect => NextTunnelState::NewState(ConnectingState::enter(shared_state))
                }
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
