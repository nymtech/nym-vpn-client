// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bytes::{Bytes, BytesMut};
use futures::{channel::mpsc, prelude::stream::SplitSink, SinkExt, StreamExt};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    request::{IpPacketRequest, IpPacketRequestData},
    response::IpPacketResponseData,
    response::{InfoLevel, IpPacketResponse},
    IpPair,
};
use nym_sdk::mixnet::{InputMessage, MixnetMessageSender, Recipient, ReconstructedMessage};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use tokio::task::JoinHandle;
use tokio_util::codec::{Decoder, Framed};
use tracing::{debug, error, info, trace, warn};
use tun2::{AbstractDevice, AsyncDevice, TunPacketCodec};

use nym_connection_monitor::{
    is_icmp_beacon_reply, is_icmp_v6_beacon_reply, ConnectionMonitorTask, ConnectionStatusEvent,
    IcmpBeaconReply, Icmpv6BeaconReply,
};

use crate::{error::Result, mixnet_connect::SharedMixnetClient};

#[derive(Debug)]
pub struct Config {
    pub ip_packet_router_address: Recipient,
}

impl Config {
    pub fn new(ip_packet_router_address: Recipient) -> Self {
        Config {
            ip_packet_router_address,
        }
    }
}

pub struct MessageCreator {
    recipient: Recipient,
}

impl MessageCreator {
    pub fn new(recipient: Recipient) -> Self {
        Self { recipient }
    }

    pub fn create_input_message(&self, bundled_packets: Bytes) -> Result<InputMessage> {
        let packet = IpPacketRequest::new_data_request(bundled_packets).to_bytes()?;

        let lane = TransmissionLane::General;
        let packet_type = None;
        let input_message = InputMessage::new_regular(self.recipient, packet, lane, packet_type);
        Ok(input_message)
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,
    ip_packet_router_address: Recipient,
    our_ips: nym_ip_packet_requests::IpPair,
    icmp_beacon_identifier: u16,
}

impl MixnetProcessor {
    pub(crate) fn new(
        device: AsyncDevice,
        mixnet_client: SharedMixnetClient,
        connection_monitor: &ConnectionMonitorTask,
        ip_packet_router_address: Recipient,
        our_ips: nym_ip_packet_requests::IpPair,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            connection_event_tx: connection_monitor.event_sender(),
            ip_packet_router_address,
            our_ips,
            icmp_beacon_identifier: connection_monitor.icmp_beacon_identifier(),
        }
    }

    pub async fn run(self, mut task_client: TaskClient) -> Result<AsyncDevice> {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.as_ref().tun_name().unwrap(),
        );

        debug!("Splitting tun device into sink and stream");
        let (tun_device_sink, mut tun_device_stream) = self.device.into_framed().split();

        // We are the exclusive owner of the mixnet client, so we can unwrap it here
        debug!("Acquiring mixnet client");
        // let our_address = self.mixnet_client.nym_address().await;

        debug!("Split mixnet sender");
        let sender = self.mixnet_client.split_sender().await;
        let recipient = self.ip_packet_router_address;

        let mut multi_ip_packet_encoder =
            MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

        let message_creator = MessageCreator::new(recipient);

        // Starting the mixnet listener.
        // NOTE: we are cloning the shutdown handle here, which is not ideal. What we actually need
        // is another subscription from the TaskManager to be able to listen to the shutdown event
        // in both tasks independently.
        debug!("Starting mixnet listener");
        let mixnet_listener = MixnetListener::new(
            self.mixnet_client,
            task_client.fork("mixnet_listener"),
            tun_device_sink,
            self.icmp_beacon_identifier,
            self.our_ips,
            self.connection_event_tx.clone(),
        )
        .await;
        let mixnet_listener_handle = mixnet_listener.start();

        info!("Mixnet processor is running");
        while !task_client.is_shutdown() {
            tokio::select! {
                _ = task_client.recv_with_delay() => {
                    trace!("MixnetProcessor: Received shutdown");
                    break;
                }
                // To make sure we don't wait too long before filling up the buffer, which destroys
                // latency, cap the time waiting for the buffer to fill
                Some(bundled_packets) = multi_ip_packet_encoder.buffer_timeout() => {
                    assert!(!bundled_packets.is_empty());

                    match message_creator.create_input_message(bundled_packets) {
                        Ok(input_message) => {
                            let ret = sender.send(input_message).await;
                            if ret.is_err() && !task_client.is_shutdown_poll() {
                                error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                            }
                        }
                        Err(err) => {
                            error!("Failed to create input message: {err}");
                        }
                    };
                }
                Some(Ok(packet)) = tun_device_stream.next() => {
                    // Bundle up IP packets into a single mixnet message
                    if let Some(input_message) = multi_ip_packet_encoder
                        .append_packet(packet.into())
                    {
                        match message_creator.create_input_message(input_message) {
                            Ok(input_message) => {
                                let ret = sender.send(input_message).await;
                                if ret.is_err() && !task_client.is_shutdown_poll() {
                                    error!("Could not forward IP packet to the mixnet. The packet(s) will be dropped.");
                                }
                            }
                            Err(err) => {
                                error!("Failed to create input message, the packet(s) will be dropped: {err}");
                            }
                        }
                    }
                }
                else => {
                    error!("Mixnet processor: tun device stream ended");
                    break;
                }
            }
        }

        info!("Waiting for mixnet listener to finish");
        let tun_device_sink = mixnet_listener_handle.await.unwrap();

        debug!("MixnetProcessor: Exiting");
        Ok(tun_device_sink
            .reunite(tun_device_stream)
            .expect("reunite should work because of same device split")
            .into_inner())
    }
}

// The mixnet listener is responsible for listening for incoming mixnet messages from the mixnet
// client, and if they contain IP packets, forward them to the tun device.
struct MixnetListener {
    // Mixnet client for receiving messages
    mixnet_client: SharedMixnetClient,

    // Task client for receiving shutdown signals
    task_client: TaskClient,

    // Sink for sending packets to the tun device
    tun_device_sink: SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>>,

    // Identifier for ICMP beacon
    icmp_beacon_identifier: u16,

    // Our IP addresses
    our_ips: IpPair,

    // Connection event sender
    connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,

    // Our mixnet address
    our_address: Recipient,
}

impl MixnetListener {
    async fn new(
        mixnet_client: SharedMixnetClient,
        task_client: TaskClient,
        tun_device_sink: SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>>,
        icmp_beacon_identifier: u16,
        our_ips: IpPair,
        connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,
    ) -> Self {
        let our_address = mixnet_client.nym_address().await;

        Self {
            mixnet_client,
            task_client,
            tun_device_sink,
            icmp_beacon_identifier,
            our_ips,
            connection_event_tx,
            our_address,
        }
    }

    fn send_connection_event(&self, event: ConnectionStatusEvent) {
        let res = self.connection_event_tx.unbounded_send(event);
        if res.is_err() && !self.task_client.is_shutdown() {
            error!("Failed to send connection event to connection monitor");
        }
    }

    fn check_for_mix_self_ping(&self, request: &IpPacketRequest) {
        match request.data {
            IpPacketRequestData::Ping(ref ping_request)
                if ping_request.reply_to == self.our_address =>
            {
                self.send_connection_event(ConnectionStatusEvent::MixnetSelfPing);
            }
            ref request => {
                debug!("Received unexpected request: {request:?}");
            }
        }
    }

    fn check_for_icmp_beacon_reply(&self, packet: &Bytes) {
        if let Some(connection_event) =
            check_for_icmp_beacon_reply(packet, self.icmp_beacon_identifier, self.our_ips)
        {
            self.send_connection_event(connection_event);
        }
    }

    async fn handle_reconstructed_message(
        &self,
        message: ReconstructedMessage,
        multi_ip_packet_decoder: &mut MultiIpPacketCodec,
    ) -> Result<Option<MixnetMessageOutcome>> {
        match IpPacketResponse::from_reconstructed_message(&message) {
            Ok(response) => match response.data {
                IpPacketResponseData::StaticConnect(_) => {
                    info!("Received static connect response when already connected - ignoring");
                }
                IpPacketResponseData::DynamicConnect(_) => {
                    info!("Received dynamic connect response when already connected - ignoring");
                }
                IpPacketResponseData::Disconnect(_) => {
                    // Disconnect is not yet handled on the IPR side anyway
                    info!("Received disconnect response, ignoring for now");
                }
                IpPacketResponseData::UnrequestedDisconnect(_) => {
                    info!("Received unrequested disconnect response, ignoring for now");
                }
                IpPacketResponseData::Data(data_response) => {
                    // Un-bundle the mixnet message and send the individual IP packets
                    // to the tun device
                    let mut bytes = BytesMut::from(&*data_response.ip_packet);
                    let mut responses = vec![];
                    while let Ok(Some(packet)) = multi_ip_packet_decoder.decode(&mut bytes) {
                        self.check_for_icmp_beacon_reply(&packet);

                        // Consider not including packets that are ICMP ping replies to our beacon
                        // in the responses. We are defensive here just in case we incorrectly
                        // label real packets as ping replies to our beacon.
                        responses.push(packet);
                    }
                    return Ok(Some(MixnetMessageOutcome::IpPackets(responses)));
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
                        InfoLevel::Info => info!("{msg}"),
                        InfoLevel::Warn => warn!("{msg}"),
                        InfoLevel::Error => error!("{msg}"),
                    }
                }
            },
            Err(err) => {
                // The exception to when we are not expecting a response, is when we
                // are sending a ping to ourselves.
                if let Ok(request) = IpPacketRequest::from_reconstructed_message(&message) {
                    self.check_for_mix_self_ping(&request);
                } else {
                    warn!("Failed to deserialize reconstructed message: {err}");
                }
            }
        }
        Ok(None)
    }

    async fn run(mut self) -> SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>> {
        // We are the only one listening for mixnet messages when this is active
        let mut mixnet_client_binding = self.mixnet_client.lock().await;
        let mixnet_client = mixnet_client_binding.as_mut().unwrap();

        let mut decoder = MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

        while !self.task_client.is_shutdown() {
            tokio::select! {
                _ = self.task_client.recv_with_delay() => {
                    trace!("Mixnet listener: Received shutdown");
                    break;
                }
                Some(reconstructed_message) = mixnet_client.next() => {
                    // We're just going to assume that all incoming messags are IPR messages
                    if let Some(version) = reconstructed_message.message.first() {
                        if *version != nym_ip_packet_requests::CURRENT_VERSION {
                            error!("Received packet with invalid version: v{version}, is your client up to date?");
                            continue;
                        }
                    }
                    match self.handle_reconstructed_message(reconstructed_message, &mut decoder).await {
                        Ok(Some(MixnetMessageOutcome::IpPackets(packets))) => {
                            for packet in packets {
                                if let Err(err) = self.tun_device_sink.send(packet.into()).await {
                                    error!("Failed to send packet to tun device: {err}");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            error!("Mixnet listener: {err}");
                        }
                    }
                }
                else => {
                    error!("Mixnet listener: mixnet stream ended");
                    break;
                }
            }
        }

        debug!("Mixnet listener: Exiting");
        self.tun_device_sink
    }

    fn start(self) -> JoinHandle<SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>>> {
        tokio::spawn(async move { self.run().await })
    }
}

enum MixnetMessageOutcome {
    IpPackets(Vec<Bytes>),
}

fn check_for_icmp_beacon_reply(
    packet: &Bytes,
    icmp_beacon_identifier: u16,
    our_ips: IpPair,
) -> Option<ConnectionStatusEvent> {
    match is_icmp_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv4) {
        Some(IcmpBeaconReply::TunDeviceReply) => {
            debug!("Received ping response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv4IprTunDevicePingReply);
        }
        Some(IcmpBeaconReply::ExternalPingReply(_source)) => {
            debug!("Received ping response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv4IprExternalPingReply);
        }
        None => {}
    }

    match is_icmp_v6_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv6) {
        Some(Icmpv6BeaconReply::TunDeviceReply) => {
            debug!("Received ping v6 response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv6IprTunDevicePingReply);
        }
        Some(Icmpv6BeaconReply::ExternalPingReply(_source)) => {
            debug!("Received ping v6 response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv6IprExternalPingReply);
        }
        None => {}
    }

    None
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn start_processor(
    config: Config,
    dev: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    task_manager: &TaskManager,
    our_ips: nym_ip_packet_requests::IpPair,
    connection_monitor: &ConnectionMonitorTask,
) -> JoinHandle<Result<AsyncDevice>> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        connection_monitor,
        config.ip_packet_router_address,
        our_ips,
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
