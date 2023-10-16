// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use futures::StreamExt;
use log::*;
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use tun::{AsyncDevice, Device};

pub struct Config {
    pub mixnet_tun_config: tun::Configuration,
    pub recipient: Recipient,
}

impl Config {
    pub fn new(recipient: Recipient) -> Self {
        let mut mixnet_tun_config = tun::Configuration::default();
        mixnet_tun_config.up();

        Config {
            mixnet_tun_config,
            recipient,
        }
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: MixnetClient,
    recipient: Recipient,
}

impl MixnetProcessor {
    pub fn new(device: AsyncDevice, mixnet_client: MixnetClient, recipient: Recipient) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            recipient,
        }
    }

    pub async fn run(self, mut shutdown: TaskClient) {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name()
        );
        let mut stream = self.device.into_framed();
        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Some(Ok(packet)) = stream.next() => {
                    let ret = self.mixnet_client.send_message(self.recipient, packet.get_bytes(), IncludedSurbs::ExposeSelfAddress).await;
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
    let mixnet_client = MixnetClient::connect_new().await?;
    let dev = tun::create_as_async(&config.mixnet_tun_config)?;
    let processor = MixnetProcessor::new(dev, mixnet_client, config.recipient);
    let shutdown_listener = shutdown.subscribe();
    tokio::spawn(async move {
        processor.run(shutdown_listener).await;
    });
    Ok(())
}
