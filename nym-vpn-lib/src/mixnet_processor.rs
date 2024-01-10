// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use nym_ip_packet_requests::{IpPacketRequest, IpPacketResponse, IpPacketResponseData};
use nym_sdk::mixnet::{InputMessage, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use nym_validator_client::models::DescribedGateway;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{debug, error, info, trace, warn};
use tun::{AsyncDevice, Device, TunPacket};

use crate::{
    error::{Error, Result},
    mixnet_connect::SharedMixnetClient,
};

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

    pub fn try_from_described_gateway(gateway: &DescribedGateway) -> Result<Self> {
        let address = gateway
            .self_described
            .clone()
            .and_then(|described_gateway| described_gateway.ip_packet_router)
            .map(|ipr| ipr.address)
            .ok_or(Error::MissingIpPacketRouterAddress)?;
        Ok(Self(
            Recipient::try_from_base58_string(address)
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
    mixnet_client: SharedMixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
    // TODO: handle this as part of setting up the mixnet client
    enable_two_hop: bool,
}

impl MixnetProcessor {
    pub fn new(
        device: AsyncDevice,
        mixnet_client: SharedMixnetClient,
        ip_packet_router_address: IpPacketRouterAddress,
        enable_two_hop: bool,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            ip_packet_router_address,
            enable_two_hop,
        }
    }

    pub async fn run(self, mut shutdown: TaskClient) -> Result<AsyncDevice> {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name()
        );

        debug!("Splitting tun device into sink and stream");
        let (mut sink, mut stream) = self.device.into_framed().split();

        // We are the exclusive owner of the mixnet client, so we can unwrap it here
        debug!("Acquiring mixnet client");
        let mut mixnet_handle = timeout(Duration::from_secs(2), self.mixnet_client.lock())
            .await
            .map_err(|_| Error::MixnetClientDeadlock)?;
        let mixnet_client = mixnet_handle.as_mut().unwrap();

        debug!("Split mixnet sender");
        let sender = mixnet_client.split_sender();
        let recipient = self.ip_packet_router_address;

        debug!("Setting up mixnet stream");
        let mixnet_stream = mixnet_client
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

        info!("Mixnet processor is running");
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

                    let lane = TransmissionLane::General;
                    let packet_type = None;
                    let hops = self.enable_two_hop.then_some(0);
                    let input_message = InputMessage::new_regular_with_custom_hops(
                            recipient.0,
                            packet,
                            lane,
                            packet_type,
                            hops,
                        );

                    let ret = sender.send(input_message).await;
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
        Ok(sink
            .reunite(stream)
            .expect("reunite should work because of same device split")
            .into_inner())
    }
}

pub async fn start_processor(
    config: Config,
    dev: tun::AsyncDevice,
    mixnet_client: SharedMixnetClient,
    task_manager: &TaskManager,
    enable_two_hop: bool,
) -> JoinHandle<Result<AsyncDevice>> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        config.ip_packet_router_address,
        enable_two_hop,
    );
    let shutdown_listener = task_manager.subscribe();
    tokio::spawn(async move {
        let ret = processor.run(shutdown_listener).await;
        if let Err(err) = ret {
            error!("Mixnet processor error: {err}");
            Err(err)
        } else {
            ret
        }
    })
}
