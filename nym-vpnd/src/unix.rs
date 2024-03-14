use std::path::{Path, PathBuf};

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{service::VpnServiceCommand, command_interface::CommandInterfaceConnectionHandler};

pub fn start_command_interface() -> (std::thread::JoinHandle<()>, Receiver<VpnServiceCommand>) {
    // Channel to send commands to the vpn service
    let (vpn_command_tx, vpn_command_rx) = tokio::sync::mpsc::channel(32);

    let socket_path = Path::new("/var/run/nym-vpn.socket");

    let handle = std::thread::spawn(move || {
        let command_rt = tokio::runtime::Runtime::new().unwrap();
        command_rt.block_on(async {
            // Spawn command interface
            tokio::task::spawn(async {
                CommandInterface::new(vpn_command_tx, socket_path)
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
    socket_path: PathBuf,
}

impl CommandInterface {
    fn new(vpn_command_tx: Sender<VpnServiceCommand>, socket_path: &Path) -> Self {
        Self {
            vpn_command_tx,
            socket_path: socket_path.to_path_buf(),
        }
    }

    async fn listen(self) {
        // Remove any previous file just in case
        if let Err(err) = std::fs::remove_file(&self.socket_path) {
            println!(
                "Failed to remove previous command interface socket: {:?}",
                err
            );
        }

        let listener = tokio::net::UnixListener::bind(&self.socket_path).unwrap();

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone()).handle(socket);
        }
    }
}

impl Drop for CommandInterface {
    fn drop(&mut self) {
        println!("Removing command interface socket: {:?}", self.socket_path);
        match std::fs::remove_file(&self.socket_path) {
            Ok(_) => println!("Removed command interface socket: {:?}", self.socket_path),
            Err(e) => println!("Failed to remove command interface socket: {:?}", e),
        }
    }
}
