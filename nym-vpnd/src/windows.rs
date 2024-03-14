use crate::service::VpnServiceCommand;
use tracing::{info, error};

#[derive(Debug)]
pub(crate) enum VpnCtrlMessage {
    Stop,
}

pub fn start_command_interface() -> (std::thread::JoinHandle<()>, Receiver<VpnServiceCommand>) {
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::channel(32);

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            // Spawn command interface
            tokio::task::spawn(async {
                CommandInterface::new(vpn_command_tx)
                    .listen()
                    .await
            });

            // Signal listener
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("Received Ctrl-C, shutting down");
                }
            }

            // Signal shutdown here
            // ...

            // Wait for shutdown here
            // ...

            println!("Command interface exiting");
        });
    });

    (handle, vpn_command_rx)
}

struct CommandInterface {
    vpn_command_tx: Sender<VpnServiceCommand>,
}

impl CommandInterface {
    fn new(vpn_command_tx: Sender<VpnServiceCommand>) -> Self {
        Self {
            vpn_command_tx,
        }
    }

    async fn listen(self) {
        info!("Listening for commands");
        error!("NOT IMPLEMENTED, EXITING");
    }
}
