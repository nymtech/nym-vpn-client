mod mixnet_route_handler;
mod mixnet_tunnel;
mod states;

use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use mixnet_route_handler::MixnetRouteHandler;
use mixnet_tunnel::MixnetTunnelHandle;
use states::DisconnectedState;

#[async_trait::async_trait]
trait TunnelStateHandler: Send {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState;
}

pub enum NextTunnelState {
    NewState((Box<dyn TunnelStateHandler>, TunnelState)),
    SameState(Box<dyn TunnelStateHandler>),
    Finished,
}

#[derive(Debug)]
enum TunnelCommand {
    Connect,
    Disconnect,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

#[derive(Debug)]
pub enum TunnelEvent {
    NewState(TunnelState),
}

pub struct SharedState {
    route_handler: MixnetRouteHandler,
    tunnel_shutdown_token: Option<CancellationToken>,
    tunnel_handle: Option<MixnetTunnelHandle>,
}

pub struct TunnelStateMachine {
    current_state_handler: Box<dyn TunnelStateHandler>,
    shared_state: SharedState,
    command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
    event_sender: mpsc::UnboundedSender<TunnelEvent>,
    shutdown_token: CancellationToken,
}

impl TunnelStateMachine {
    pub async fn spawn(
        command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
        event_sender: mpsc::UnboundedSender<TunnelEvent>,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        let (current_state_handler, _) = DisconnectedState::enter();

        let route_handler = MixnetRouteHandler::new().await?;

        let shared_state = SharedState {
            route_handler,
            tunnel_shutdown_token: None,
            tunnel_handle: None,
        };

        let tunnel_state_machine = Self {
            current_state_handler,
            shared_state,
            command_receiver,
            event_sender,
            shutdown_token,
        };

        Ok(tokio::spawn(tunnel_state_machine.run()))
    }

    async fn run(mut self) {
        loop {
            let next_state = self
                .current_state_handler
                .handle_event(
                    &self.shutdown_token,
                    &mut self.command_receiver,
                    &mut self.shared_state,
                )
                .await;

            match next_state {
                NextTunnelState::NewState((new_state_handler, new_state)) => {
                    self.current_state_handler = new_state_handler;

                    log::debug!("New tunnel state: {:?}", new_state);
                    let _ = self.event_sender.send(TunnelEvent::NewState(new_state));
                }
                NextTunnelState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextTunnelState::Finished => break,
            }
        }

        self.shared_state.route_handler.stop().await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("routing failure")]
    RouteHandler(#[source] mixnet_route_handler::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
