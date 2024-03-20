// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures::{channel::mpsc, SinkExt, StreamExt};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    request::{IpPacketRequest, IpPacketRequestData},
    response::IpPacketResponseData,
    response::{InfoLevel, IpPacketResponse},
};
use nym_sdk::mixnet::{InputMessage, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use nym_validator_client::models::DescribedGateway;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::Decoder;
use tracing::{debug, error, info, trace, warn};
use tun2::{AbstractDevice, AsyncDevice};

use crate::{
    connection_monitor,
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

struct MessageCreator {
    recipient: Recipient,
    enable_two_hop: bool,
}

impl MessageCreator {
    fn new(recipient: Recipient, enable_two_hop: bool) -> Self {
        Self {
            recipient,
            enable_two_hop,
        }
    }

    fn create_input_message(&self, bundled_packets: Bytes) -> Option<InputMessage> {
        let Ok(packet) = IpPacketRequest::new_data_request(bundled_packets).to_bytes() else {
            error!("Failed to serialize packet");
            return None;
        };

        let lane = TransmissionLane::General;
        let packet_type = None;
        let hops = self.enable_two_hop.then_some(0);
        let input_message = InputMessage::new_regular_with_custom_hops(
            self.recipient,
            packet,
            lane,
            packet_type,
            hops,
        );
        Some(input_message)
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    connection_event_tx: mpsc::UnboundedSender<connection_monitor::ConnectionEvent>,
    ip_packet_router_address: IpPacketRouterAddress,
    // TODO: handle this as part of setting up the mixnet client
    enable_two_hop: bool,
}

impl MixnetProcessor {
    pub fn new(
        device: AsyncDevice,
        mixnet_client: SharedMixnetClient,
        connection_event_tx: futures::channel::mpsc::UnboundedSender<
            connection_monitor::ConnectionEvent,
        >,
        ip_packet_router_address: IpPacketRouterAddress,
        enable_two_hop: bool,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            connection_event_tx,
            ip_packet_router_address,
            enable_two_hop,
        }
    }

    pub async fn run(self, mut shutdown: TaskClient) -> Result<AsyncDevice> {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.as_ref().tun_name().unwrap(),
        );

        debug!("Splitting tun device into sink and stream");
        let (mut tun_device_sink, mut tun_device_stream) = self.device.into_framed().split();

        // We are the exclusive owner of the mixnet client, so we can unwrap it here
        debug!("Acquiring mixnet client");
        let mut mixnet_handle = timeout(Duration::from_secs(2), self.mixnet_client.lock())
            .await
            .map_err(|_| Error::MixnetClientDeadlock)?;
        let mixnet_client = mixnet_handle.as_mut().unwrap();
        let our_address = *mixnet_client.nym_address();

        debug!("Split mixnet sender");
        let sender = mixnet_client.split_sender();
        let recipient = self.ip_packet_router_address;

        let mut multi_ip_packet_encoder =
            MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);
        let mut multi_ip_packet_decoder =
            MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

        let message_creator = MessageCreator::new(recipient.0, self.enable_two_hop);

        info!("Mixnet processor is running");
        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv_with_delay() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                // To make sure we don't wait too long before filling up the buffer, which destroys
                // latency, cap the time waiting for the buffer to fill
                Some(bundled_packets) = multi_ip_packet_encoder.buffer_timeout() => {
                    assert!(!bundled_packets.is_empty());

                    if let Some(input_message) = message_creator.create_input_message(bundled_packets) {
                        let ret = sender.send(input_message).await;
                        if ret.is_err() && !shutdown.is_shutdown_poll() {
                            error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                        }
                    };
                }
                Some(Ok(packet)) = tun_device_stream.next() => {
                    // Bundle up IP packets into a single mixnet message
                    if let Some(input_message) = multi_ip_packet_encoder
                        .append_packet(packet.into())
                        .and_then(|bundled_packets| message_creator.create_input_message(bundled_packets))
                    {
                        let ret = sender.send(input_message).await;
                        if ret.is_err() && !shutdown.is_shutdown_poll() {
                            error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                        }
                    }
                }
                Some(reconstructed_message) = mixnet_client.next() => {
                    // Check version of request
                    if let Some(version) = reconstructed_message.message.first() {
                        if *version != nym_ip_packet_requests::CURRENT_VERSION {
                            log::error!("Received packet with invalid version: v{version}, is your client up to date?");
                            continue;
                        }
                    }

                    match IpPacketResponse::from_reconstructed_message(&reconstructed_message) {
                        Ok(response) => match response.data {
                            IpPacketResponseData::StaticConnect(_) => {
                                info!("Received static connect response when already connected - ignoring");
                            },
                            IpPacketResponseData::DynamicConnect(_) => {
                                info!("Received dynamic connect response when already connected - ignoring");
                            },
                            IpPacketResponseData::Disconnect(_) => {
                                // Disconnect is not yet handled on the IPR side anyway
                                info!("Received disconnect response, ignoring for now");
                            },
                            IpPacketResponseData::UnrequestedDisconnect(_) => {
                                info!("Received unrequested disconnect response, ignoring for now");
                            },
                            IpPacketResponseData::Data(data_response) => {
                                // Un-bundle the mixnet message and send the individual IP packets
                                // to the tun device
                                let mut bytes = BytesMut::from(&*data_response.ip_packet);
                                while let Ok(Some(packet)) = multi_ip_packet_decoder.decode(&mut bytes) {
                                    tun_device_sink.send(packet.into()).await?;
                                }
                            }
                            IpPacketResponseData::Pong(_) => {
                                info!("Received pong response, ignoring for now");
                            }
                            IpPacketResponseData::Health(_) => {
                                info!("Received health response, ignoring for now");
                            }
                            IpPacketResponseData::Info(info) => {
                                let msg = format!("Received info response from the mixnet: {}", info.reply);
                                match info.level {
                                    InfoLevel::Info => log::info!("{msg}"),
                                    InfoLevel::Warn => log::warn!("{msg}"),
                                    InfoLevel::Error => log::error!("{msg}"),
                                }
                            }
                        },
                        Err(err) => {
                            // The exception to when we are not expecting a response, is when we
                            // are sending a ping to ourselves.
                            if let Ok(request) = IpPacketRequest::from_reconstructed_message(&reconstructed_message) {
                                match request.data {
                                    IpPacketRequestData::Ping(ref ping_request) => {
                                        if ping_request.reply_to == our_address {
                                            self.connection_event_tx.unbounded_send(connection_monitor::ConnectionEvent::MixnetSelfPing).unwrap();
                                        } else {
                                            info!("Received unexpected ping - ignoring");
                                        }
                                    },
                                    ref request => {
                                        info!("Received unexpected request: {request:?}");
                                    }
                                }
                            } else {
                                warn!("Failed to deserialize reconstructed message: {err}");
                            }
                        }
                    }
                }
            }
        }
        debug!("MixnetProcessor: Exiting");
        Ok(tun_device_sink
            .reunite(tun_device_stream)
            .expect("reunite should work because of same device split")
            .into_inner())
    }
}

pub async fn start_processor(
    config: Config,
    dev: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    task_manager: &TaskManager,
    enable_two_hop: bool,
    connection_event_tx: mpsc::UnboundedSender<connection_monitor::ConnectionEvent>,
) -> JoinHandle<Result<AsyncDevice>> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        connection_event_tx,
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
