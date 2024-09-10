mod states;

use states::DisconnectedState;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[async_trait::async_trait]
trait TunnelStateHandler: Send {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
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
}

#[derive(Debug)]
pub enum TunnelEvent {
    NewState(TunnelState),
}

pub struct TunnelStateMachine {
    current_state_handler: Box<dyn TunnelStateHandler>,
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

        let tunnel_state_machine = Self {
            current_state_handler,
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
                .handle_event(&self.shutdown_token, &mut self.command_receiver)
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
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
