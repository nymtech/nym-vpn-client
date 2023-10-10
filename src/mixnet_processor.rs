// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use log::*;
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use tokio::net::UdpSocket;

const MAX_PACKET: usize = 65535;

pub struct Config {
    pub port: u16,
    pub recipient: Recipient,
}

impl Config {
    pub fn new(port: u16, recipient: Recipient) -> Self {
        Config { port, recipient }
    }
}

pub struct MixnetProcessor {
    socket: UdpSocket,
    mixnet_client: MixnetClient,
    recipient: Recipient,
}

impl MixnetProcessor {
    pub fn new(socket: UdpSocket, mixnet_client: MixnetClient, recipient: Recipient) -> Self {
        MixnetProcessor {
            socket,
            mixnet_client,
            recipient,
        }
    }

    pub async fn run(self, mut shutdown: TaskClient) {
        let mut buf = [0u8; MAX_PACKET];
        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Ok(_) = self.socket.recv(&mut buf) => {
                    let ret = self.mixnet_client.send_message(self.recipient, buf, IncludedSurbs::ExposeSelfAddress).await;
                    if ret.is_err() {
                        error!("Could not forward datagram to the mixnet. The packet will be dropped.");
                    }
                }
            }
        }
    }
}

pub async fn start_processor(
    config: Config,
    shutdown: &TaskManager,
) -> Result<(), crate::error::Error> {
    let socket = UdpSocket::bind(format!("127.0.0.1:{}", config.port)).await?;
    let mixnet_client = MixnetClient::connect_new().await?;
    let processor = MixnetProcessor::new(socket, mixnet_client, config.recipient);
    let shutdown_listener = shutdown.subscribe();
    tokio::spawn(async move { processor.run(shutdown_listener) });
    Ok(())
}
