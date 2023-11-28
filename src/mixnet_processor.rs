// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use std::{net::IpAddr, time::Duration};

use futures::{SinkExt, StreamExt};
use nym_ip_packet_requests::{
    IpPacketRequest, IpPacketResponse, IpPacketResponseData, StaticConnectResponse,
};
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use tracing::{debug, error, info, trace, warn};
use tun::{AsyncDevice, Device, TunPacket};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Config {
    pub ip_packet_router_address: IpPacketRouterAddress,
}

impl Config {
    pub fn new(ip_packet_router_address: IpPacketRouterAddress) -> Self {
        Config {
            ip_packet_router_address,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IpPacketRouterAddress(pub Recipient);

impl IpPacketRouterAddress {
    pub fn try_from_base58_string(ip_packet_router_nym_address: &str) -> Result<Self> {
        Ok(Self(
            Recipient::try_from_base58_string(ip_packet_router_nym_address)
                .map_err(|_| Error::RecipientFormattingError)?,
        ))
    }
}

impl std::fmt::Display for IpPacketRouterAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: MixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
}

impl MixnetProcessor {
    pub fn new(
        device: AsyncDevice,
        mixnet_client: MixnetClient,
        ip_packet_router_address: IpPacketRouterAddress,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            ip_packet_router_address,
        }
    }

    pub async fn run(self, mut shutdown: TaskClient) {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name()
        );
        let (mut sink, mut stream) = self.device.into_framed().split();
        let sender = self.mixnet_client.split_sender();
        let recipient = self.ip_packet_router_address;

        let mixnet_stream = self
            .mixnet_client
            .filter_map(|reconstructed_message| async move {
                match IpPacketResponse::from_reconstructed_message(&reconstructed_message) {
                    Ok(response) => match response.data {
                        IpPacketResponseData::StaticConnect(_) => {
                            info!("Received static connect response when already connected - ignoring");
                            None
                        },
                        IpPacketResponseData::DynamicConnect(_) => {
                            info!("Received dynamic connect response when already connected - ignoring");
                            None
                        },
                        IpPacketResponseData::Data(data_response) => {
                            Some(Ok(TunPacket::new(data_response.ip_packet.into())))
                        }
                    },
                    Err(err) => {
                        error!("failed to deserialize reconstructed message: {err}");
                        None
                    }
                }
            });
        tokio::pin!(mixnet_stream);

        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv_with_delay() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Some(Ok(packet)) = stream.next() => {
                    // TODO: properly investigate the binary format here and the overheard
                    let Ok(packet) = IpPacketRequest::new_ip_packet(packet.into_bytes()).to_bytes() else {
                        error!("Failed to serialize packet");
                        continue;
                    };

                    // The enum here about IncludedSurbs and ExposeSelfAddress is misleading. It is
                    // not being used. Basically IncludedSurbs::ExposeSelfAddress just omits the
                    // surbs, assuming that it is exposed inside the message. (This is the case
                    // for SOCKS5 too).
                    let ret = sender.send_message(recipient.0, &packet, IncludedSurbs::ExposeSelfAddress).await;
                    if ret.is_err() && !shutdown.is_shutdown_poll() {
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
        debug!("MixnetProcessor: Exiting");
    }
}

pub async fn start_processor(
    config: Config,
    dev: tun::AsyncDevice,
    mixnet_client: MixnetClient,
    task_manager: &TaskManager,
) -> Result<()> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(dev, mixnet_client, config.ip_packet_router_address);
    let shutdown_listener = task_manager.subscribe();
    tokio::spawn(processor.run(shutdown_listener));
    Ok(())
}
