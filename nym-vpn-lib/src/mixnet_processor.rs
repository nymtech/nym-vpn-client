// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use bytes::BytesMut;
use bytes::{Buf, Bytes};
use futures::{SinkExt, StreamExt};
use nym_ip_packet_requests::{IpPacketRequest, IpPacketResponse, IpPacketResponseData};
use nym_sdk::mixnet::{InputMessage, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use nym_validator_client::models::DescribedGateway;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::Decoder;
use tokio_util::codec::Encoder;
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

// Tokio codec for bundling multiple IP packets into one buffer that is at most 1500 bytes long.
// These packets are separated by a 2 byte length prefix.
struct BundledIpPacketCodec {
    buffer: BytesMut,
    timer: tokio::time::Interval,
}

impl BundledIpPacketCodec {
    fn new() -> Self {
        BundledIpPacketCodec {
            buffer: BytesMut::new(),
            timer: tokio::time::interval(Duration::from_millis(20)),
        }
    }

    fn append_packet(&mut self, packet: Bytes) -> Option<Bytes> {
        let mut bundled_packets = BytesMut::new();
        self.encode(packet, &mut bundled_packets).unwrap();
        if bundled_packets.is_empty() {
            None
        } else {
            // log::info!("Sphinx packet utilization: {:.2}", self.buffer.len() as f64 / 1500.0);
            Some(bundled_packets.freeze())
        }
    }

    fn flush_current_buffer(&mut self) -> Bytes {
        // let mut buffer_so_far = BytesMut::new();
        // // TODO: is it possible to move the buffer instead of copying it?
        // buffer_so_far.extend_from_slice(&self.buffer);
        // self.buffer = BytesMut::new();
        // self.timer.reset();
        // buffer_so_far.freeze();

        let mut output_buffer = BytesMut::new();
        std::mem::swap(&mut output_buffer, &mut self.buffer);
        output_buffer.freeze()
    }

    async fn bundle_timeout(&mut self) -> Option<Bytes> {
        let _a = self.timer.tick().await;
        let b = self.flush_current_buffer();
        if b.is_empty() {
            None
        } else {
            Some(b)
        }
    }

    // fn is_empty(&self) -> bool {
    //     self.buffer.is_empty()
    // }
}

impl Encoder<Bytes> for BundledIpPacketCodec {
    type Error = Error;

    fn encode(&mut self, packet: Bytes, dst: &mut BytesMut) -> Result<()> {
        if self.buffer.is_empty() {
            self.timer.reset();
        }
        let packet_size = packet.len();

        if self.buffer.len() + packet_size + 2 > 1500 {
            // If the packet doesn't fit in the buffer, send the buffer and then add it to the buffer
            dst.extend_from_slice(&self.buffer);
            self.buffer = BytesMut::new();
            self.timer.reset();
        }

        // Add the packet size
        self.buffer
            .extend_from_slice(&(packet_size as u16).to_be_bytes());
        // Add the packet to the buffer
        self.buffer.extend_from_slice(&packet);

        Ok(())
    }
}

impl Decoder for BundledIpPacketCodec {
    type Item = Bytes;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 2 {
            // Not enough bytes to read the length prefix
            return Ok(None);
        }

        let packet_size = u16::from_be_bytes([src[0], src[1]]) as usize;

        if src.len() < packet_size + 2 {
            // Not enough bytes to read the packet
            return Ok(None);
        }

        // Remove the length prefix
        src.advance(2);

        // Read the packet
        let packet = src.split_to(packet_size);

        Ok(Some(packet.freeze()))
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
    // TODO: handle this as part of setting up the mixnet client
    enable_two_hop: bool,
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
        let Ok(packet) = IpPacketRequest::new_ip_packet(bundled_packets).to_bytes() else {
            error!("Failed to serialize packet");
            // continue;
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
            self.device.get_ref().name().unwrap(),
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

        // debug!("Setting up mixnet stream");
        // let mixnet_stream = mixnet_client
        //     .filter_map(|reconstructed_message| async move {
        //         match IpPacketResponse::from_reconstructed_message(&reconstructed_message) {
        //             Ok(response) => match response.data {
        //                 IpPacketResponseData::StaticConnect(_) => {
        //                     info!("Received static connect response when already connected - ignoring");
        //                     None
        //                 },
        //                 IpPacketResponseData::DynamicConnect(_) => {
        //                     info!("Received dynamic connect response when already connected - ignoring");
        //                     None
        //                 },
        //                 IpPacketResponseData::Data(data_response) => {
        //                     // Un-bunch
        //                     let mut codec = BundledIpPacketCodec::new();
        //                     let mut bytes = BytesMut::new();
        //                     bytes.extend_from_slice(&data_response.ip_packet);
        //                     while let Ok(Some(packet)) = codec.decode(&mut bytes) {
        //                         // Handle packet
        //                         let tun_packet = TunPacket::new(packet.into());
        //                     }
        //
        //
        //                     Some(Ok(TunPacket::new(data_response.ip_packet.into())))
        //                 }
        //             },
        //             Err(err) => {
        //                 error!("failed to deserialize reconstructed message: {err}");
        //                 None
        //             }
        //         }
        //     });
        // tokio::pin!(mixnet_stream);

        // buffer to accumulate packets before sending them to the mixnet
        // let mut buffer_used = 0;
        // let mut packets_in_buffer = 0;

        let mut bundled_packet_codec = BundledIpPacketCodec::new();
        let mut bundled_packet_decoder = BundledIpPacketCodec::new();

        let message_creator = MessageCreator::new(recipient.0, self.enable_two_hop);

        // tokio timer for flushing the buffer
        // let mut bundle_timer = tokio::time::interval(Duration::from_millis(100));

        info!("Mixnet processor is running");
        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv_with_delay() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                // _ = bundle_timer.tick() => {
                // _ = bundled_packet_codec.timer.tick() => {
                Some(bundled_packets) = bundled_packet_codec.bundle_timeout() => {
                    assert!(!bundled_packets.is_empty());
                    // log::info!("Sending packet before filled up");

                    if let Some(input_message) = message_creator.create_input_message(bundled_packets) {
                        let ret = sender.send(input_message).await;
                        if ret.is_err() && !shutdown.is_shutdown_poll() {
                            error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                        }
                    };
                }
                Some(Ok(packet)) = stream.next() => {
                    // If we are the first packet, start the timer
                    // if bundled_packet_codec.is_empty() {
                    //     bundle_timer.reset();
                    // }

                    // TODO: properly investigate the binary format here and the overheard
                    // dbg!(&packet.get_bytes().len());
                    // let packet_size = packet.get_bytes().len();
                    // dbg!(packet_size);

                    // let packet = packet.into_bytes();
                    // TODO: static buffer
                    // let bundled_packets = {
                    //     let mut bundled_packets = BytesMut::new();
                    //     bundled_packet_codec.encode(packet.into_bytes(), &mut bundled_packets).unwrap();
                    //     if bundled_packets.is_empty() {
                    //         continue;
                    //     }
                    //     bundled_packets.freeze()
                    // };

                    // if let Some(bundled_packets) = bundled_packet_codec.append_packet(packet.into_bytes()) {
                    //     if let Some(input_message) = message_creator.create_input_message(bundled_packets) {
                    //         let ret = sender.send(input_message).await;
                    //         if ret.is_err() && !shutdown.is_shutdown_poll() {
                    //             error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                    //         }
                    //     }
                    // }

                    if let Some(input_message) = bundled_packet_codec
                        .append_packet(packet.into_bytes())
                        .and_then(|bundled_packets| message_creator.create_input_message(bundled_packets))
                    {
                        let ret = sender.send(input_message).await;
                        if ret.is_err() && !shutdown.is_shutdown_poll() {
                            error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                        }
                    }
                    // bundle_timer.reset();

                    // let Ok(packet) = IpPacketRequest::new_ip_packet(packet.into_bytes()).to_bytes() else {
                    //let Ok(packet) = IpPacketRequest::new_ip_packet(bundled_packets).to_bytes() else {
                    //    error!("Failed to serialize packet");
                    //    continue;
                    //};

                    //let lane = TransmissionLane::General;
                    //let packet_type = None;
                    //let hops = self.enable_two_hop.then_some(0);
                    //let input_message = InputMessage::new_regular_with_custom_hops(
                    //        recipient.0,
                    //        packet,
                    //        lane,
                    //        packet_type,
                    //        hops,
                    //    );

                    // let Some(input_message) = message_creator.create_input_message(bundled_packets) else {
                    //     continue;
                    // };

                    // let ret = sender.send(input_message).await;
                    // if ret.is_err() && !shutdown.is_shutdown_poll() {
                    //     error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                    // }
                }
                Some(packet) = mixnet_client.next() => {
                    match IpPacketResponse::from_reconstructed_message(&packet) {
                        Ok(response) => match response.data {
                            IpPacketResponseData::StaticConnect(_) => {
                                info!("Received static connect response when already connected - ignoring");
                            },
                            IpPacketResponseData::DynamicConnect(_) => {
                                info!("Received dynamic connect response when already connected - ignoring");
                            },
                            IpPacketResponseData::Data(data_response) => {
                                // Un-bunch
                                // let mut codec = BundledIpPacketCodec::new();
                                // let mut bytes = {
                                //     let mut bytes = BytesMut::new();
                                //     bytes.extend_from_slice(&data_response.ip_packet);
                                //     bytes
                                // };
                                let mut bytes = BytesMut::from(&*data_response.ip_packet);
                                while let Ok(Some(packet)) = bundled_packet_decoder.decode(&mut bytes) {
                                    sink.send(TunPacket::new(packet.into())).await?;
                                }
                            }
                        },
                        Err(err) => {
                            error!("failed to deserialize reconstructed message: {err}");
                        }
                    };
                }
                // res = sink.send_all(&mut mixnet_stream) => {
                //     warn!("Mixnet stream finished. This may mean that the gateway was shut down");
                //     if let Err(e) = res {
                //         error!("Could not forward mixnet traffic to the client - {:?}", e);
                //     }
                //     break;
                // }
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
