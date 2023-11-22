// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use futures::{SinkExt, StreamExt};
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use serde::{Deserialize, Serialize};
use tracing::{error, info, trace, warn};
use tun::{AsyncDevice, Device, TunPacket};

#[derive(Debug)]
pub struct Config {
    pub recipient: Recipient,
}

impl Config {
    pub fn new(recipient: Recipient) -> Self {
        Config { recipient }
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: MixnetClient,
    recipient: Recipient,
}

#[derive(Serialize, Deserialize)]
pub struct TaggedPacket {
    packet: bytes::Bytes,
    return_address: Recipient,
    return_mix_hops: Option<u8>,
}

impl TaggedPacket {
    fn new(packet: bytes::Bytes, return_address: Recipient, return_mix_hops: Option<u8>) -> Self {
        TaggedPacket {
            packet,
            return_address,
            return_mix_hops,
        }
    }
    fn to_tagged_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        use bincode::Options;
        let bincode_serializer = make_bincode_serializer();
        let packet: Vec<u8> = bincode_serializer.serialize(self)?;
        Ok(packet)
    }
}

fn make_bincode_serializer() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_varint_encoding()
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
        let (mut sink, mut stream) = self.device.into_framed().split();
        let sender = self.mixnet_client.split_sender();
        let recipient = self.recipient;
        let mut mixnet_stream = self
            .mixnet_client
            .map(|reconstructed_message| Ok(TunPacket::new(reconstructed_message.message.clone())));

        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Some(Ok(packet)) = stream.next() => {
                    // TODO: properly investigate the binary format here and the overheard
                    let Ok(packet) = TaggedPacket::new(packet.into_bytes(), recipient, None).to_tagged_bytes() else {
                        error!("Failed to serialize packet");
                        continue;
                    };

                    // The enum here about IncludedSurbs and ExposeSelfAddress is misleading. It is
                    // not being used. Basically IncludedSurbs::ExposeSelfAddress just omits the
                    // surbs, assuming that it is exposed in side the message. (This is the case
                    // for SOCKS5 too).
                    let ret = sender.send_message(recipient, &packet, IncludedSurbs::ExposeSelfAddress).await;
                    if ret.is_err() {
                        error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                    }
                }
                res = sink.send_all(&mut mixnet_stream) => {
                    warn!("Mixnet stream finished. This may mean that the gateway was shut down");
                    if let Err(e) = res {
                        error!("Could not forward mixnet traffic to the client - {:?}", e);
                    }
                    break;
                }
            }
        }
    }
}

pub async fn start_processor(
    config: Config,
    dev: tun::AsyncDevice,
    mixnet_client: MixnetClient,
    task_manager: &TaskManager,
) -> Result<(), crate::error::Error> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(dev, mixnet_client, config.recipient);
    let shutdown_listener = task_manager.subscribe();
    tokio::spawn(processor.run(shutdown_listener));
    Ok(())
}
