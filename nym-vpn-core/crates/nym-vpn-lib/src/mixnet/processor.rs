// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{result::Result, time::Duration};

use bytes::{Bytes, BytesMut};
use futures::{FutureExt, StreamExt, channel::mpsc, future::Fuse, pin_mut};
use nym_connection_monitor::{ConnectionMonitorTask, ConnectionStatusEvent};
use nym_gateway_directory::IpPacketRouterAddress;
use nym_ip_packet_requests::{
    IpPair,
    codec::{IprPacket, MultiIpPacketCodec},
    v8::request::IpPacketRequest,
};
use nym_sdk::{
    ShutdownManager,
    mixnet::{
        InputMessage, MixnetClient, MixnetClientSender, MixnetMessageSender,
        MixnetMessageSinkTranslator, Recipient, TransmissionLane,
    },
};
use tokio::task::JoinHandle;
use tokio_util::{codec::Encoder, sync::CancellationToken};
use tun::{AsyncDevice, Device};

use super::{MixnetError, backpressure::MixnetBackpressureMonitor};

/// How much time to wait for ipr disconnect before proceeding to shutdown.
const IPR_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Interval between attempts to send ipr disconnect
const IPR_DISCONNECT_RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct MixnetProcessorConfig {
    pub ip_packet_router_address: IpPacketRouterAddress,
    pub our_ips: IpPair,
}

impl MixnetProcessorConfig {
    pub fn new(ip_packet_router_address: IpPacketRouterAddress, our_ips: IpPair) -> Self {
        MixnetProcessorConfig {
            ip_packet_router_address,
            our_ips,
        }
    }
}

struct MessageCreator {
    recipient: Recipient,
}

impl MessageCreator {
    fn new(recipient: Recipient) -> Self {
        Self { recipient }
    }

    fn create_disconnect_message(&self) -> Result<InputMessage, MixnetError> {
        let (packet, _) = IpPacketRequest::new_disconnect_request();

        let packet = packet.to_bytes()?;
        let lane = TransmissionLane::General;
        let packet_type = None;
        let surbs = 0;
        let input_message =
            InputMessage::new_anonymous(self.recipient, packet, surbs, lane, packet_type);
        Ok(input_message)
    }
}

struct MixnetProcessor {
    // The tun device we're reading from and writing to
    device: AsyncDevice,

    // The mixnet client for sending and receiving messages from the mixnet
    mixnet_client: MixnetClient,

    // Mixnet_shutdown_manager to lister to mixnet failure
    mixnet_shutdown_manager: ShutdownManager,

    // The connection monitor for sending connection events
    connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,

    // The address of the IP packet router we're sending messages to
    ip_packet_router_address: IpPacketRouterAddress,

    // Our IP addresses
    our_ips: IpPair,

    // Identifier for ICMP beacon, so we can check incoming ICMP packets to see if we should
    // forward them to the connection monitor
    icmp_beacon_identifier: u16,

    // Listen for when we should disconnect from the IPR and being shutting down
    cancel_token: CancellationToken,
}

impl MixnetProcessor {
    fn new(
        device: AsyncDevice,
        mixnet_client: MixnetClient,
        mixnet_shutdown_manager: ShutdownManager,
        connection_monitor: &ConnectionMonitorTask,
        ip_packet_router_address: IpPacketRouterAddress,
        our_ips: IpPair,
        cancel_token: CancellationToken,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            mixnet_shutdown_manager,
            connection_event_tx: connection_monitor.event_sender(),
            ip_packet_router_address,
            our_ips,
            icmp_beacon_identifier: connection_monitor.icmp_beacon_identifier(),
            cancel_token,
        }
    }

    async fn run(self) -> Result<AsyncDevice, MixnetError> {
        tracing::info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name().unwrap(),
        );

        tracing::debug!("Splitting tun device into sink and stream");
        let (tun_device_sink, mut tun_device_stream) = self.device.into_framed().split();

        tracing::debug!("Split mixnet sender");
        let (mixnet_sender, lane_queue_lengths) = {
            (
                self.mixnet_client.split_sender(),
                self.mixnet_client.shared_lane_queue_lengths(),
            )
        };
        let mixnet_client_shutdown_token = self
            .mixnet_shutdown_manager
            .child_shutdown_token()
            .inner()
            .clone();

        let message_creator = MessageCreator::new(self.ip_packet_router_address.into());

        // Starting the mixnet listener.
        tracing::debug!("Starting mixnet listener");
        let mut mixnet_listener_handle = super::mixnet_listener::MixnetListener::spawn(
            self.mixnet_client,
            self.mixnet_shutdown_manager,
            tun_device_sink,
            self.icmp_beacon_identifier,
            self.our_ips,
            self.connection_event_tx.clone(),
            self.cancel_token.clone(),
        );

        // Keep track of whether we've sent the disconnect message, so we don't send it multiple
        // times
        let mut has_sent_ipr_disconnect = false;

        // Keeps track of whether ipr disconnect timeout has been activated.
        let mut is_disconnect_timeout_active = false;

        // Ipr disconnect timeout future set upon cancellation
        let ipr_disconnect_timeout = Fuse::terminated();
        pin_mut!(ipr_disconnect_timeout);

        let mut payload_topup_interval =
            tokio::time::interval(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

        // The packet bundler is the buffer where we bundle multiple IP packets into a single
        // mixnet payload.
        let mut packet_bundler = MultiIpPacketCodec::new();

        // Create input messages for the mixnet client from bundled IP packets
        let input_message_creator = ToIprDataRequest::new(self.ip_packet_router_address);

        // The backpressure monitor checks for backpressure inside the mixnet client. More
        // specifically, at the queues in the Poisson process
        let backpressure_monitor =
            MixnetBackpressureMonitor::new(lane_queue_lengths.clone(), None).start();
        let notify_backpressure_lifted = backpressure_monitor.get_notify_backpressure_lifted();

        tracing::info!("Mixnet processor is running");
        loop {
            // Disable the TUN read select branch if we are in backpressure
            let is_backpressure = backpressure_monitor.is_backpressure();

            tokio::select! {
                biased;
                _ = &mut ipr_disconnect_timeout => {
                    tracing::warn!("Timed out waiting for ipr disconnect");
                    break;
                }
                // Handle task manager shutdown
                _ = mixnet_client_shutdown_token.cancelled() => {
                    tracing::debug!("Mixnet client has shut down");
                    break;
                }
                // When we get the cancel token, send a disconnect message to the IPR. We keep
                // running until the mixnet listener receives the disconnect response, so we can
                // make sure we've fully disconnected before we return.
                _ = self.cancel_token.cancelled(), if !has_sent_ipr_disconnect => {
                    // Start disconnect timeout upon receiving cancellation in the very first time.
                    if is_disconnect_timeout_active {
                        tracing::debug!("Re-sending disconnect message");
                    } else {
                        is_disconnect_timeout_active = true;
                        ipr_disconnect_timeout.set(tokio::time::sleep(IPR_DISCONNECT_TIMEOUT).fuse());
                        tracing::debug!("Cancel token triggered, sending disconnect message");
                    }

                    let input_message = match message_creator.create_disconnect_message() {
                        Ok(input_message) => input_message,
                        Err(err) => {
                            tracing::error!("Failed to create disconnect message: {err}");
                            tokio::time::sleep(IPR_DISCONNECT_RETRY_DELAY).await;
                            continue;
                        }
                    };
                    if let Err(err) = mixnet_sender.send(input_message).await {
                        tracing::error!("Failed to send disconnect message: {err}");
                        tokio::time::sleep(IPR_DISCONNECT_RETRY_DELAY).await;
                        continue;
                    }

                    tracing::info!("Sent disconnect message");
                    has_sent_ipr_disconnect = true;
                }
                // When the mixnet listener receives the disconnect response, it will notify us
                // that it's done. This means we can now stop
                _ = &mut mixnet_listener_handle => {
                    tracing::debug!("Mixnet listener has finished");
                    break;
                }
                // The backpressure monitor will notify us when the backpressure is lifted, so we
                // can restart the select with updated preconditions
                _ = notify_backpressure_lifted.notified(), if is_backpressure => {
                    tracing::trace!("Backpressure lifted");
                    continue;
                }
                // Read from the tun device and send the IP packet to the mixnet
                tun_packet = tun_device_stream.next(), if !is_backpressure => match tun_packet {
                    Some(Ok(tun_packet)) => {
                        payload_topup_interval.reset();
                        let packet = IprPacket::from(tun_packet.into_bytes());
                        tokio::select! {
                            ret = handle_packet(packet, &mut packet_bundler, &input_message_creator, &mixnet_sender) => {
                                if ret.is_err() && !mixnet_client_shutdown_token.is_cancelled() {
                                    tracing::error!("Failed to send IP packet to the mixnet");
                                }
                            }
                            _ = mixnet_client_shutdown_token.cancelled() => {
                                tracing::debug!("Mixnet client has shut down while sending");
                                break;
                            }
                            _ = self.cancel_token.cancelled() => {
                                tracing::debug!("Received cancellation while sending");
                                break;
                            }
                        }
                    }
                    Some(Err(err)) => {
                        tracing::error!("Failed to read from tun device: {err}");
                        break;
                    }
                    None => {
                        tracing::error!("Tun device stream ended");
                        break;
                    }
                },
                // To make sure we don't wait too long before filling up the buffer, which destroys
                // latency, cap the time waiting for the buffer to fill
                _ = payload_topup_interval.tick() => {
                    tracing::trace!("Buffer timeout");

                    // If we already have pending packets that we are waiting to send to the
                    // mixnet, there is no point in flushing the current buffer. Instead keep
                    // filling up so we can fit more IP packets in the mixnet packet payload.
                    let packet_queue = backpressure_monitor.packet_queue_length();
                    if packet_queue > 0 {
                        tracing::trace!("Skipping payload topup timeout (queue: {packet_queue})");
                        continue;
                    }

                    tokio::select! {
                        ret = handle_packet(IprPacket::Flush, &mut packet_bundler, &input_message_creator, &mixnet_sender) => {
                            if ret.is_err() && !mixnet_client_shutdown_token.is_cancelled() {
                                tracing::error!("Failed to flush the multi IP packet sink");
                            }
                        }
                        _ = mixnet_client_shutdown_token.cancelled() => {
                            tracing::debug!("Mixnet client has shut down while flushing");
                            break;
                        }
                        _ = self.cancel_token.cancelled() => {
                            tracing::debug!("Received shutdown while flushing");
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!("Stopping mixnet backpressure monitor");
        backpressure_monitor.stop().await;

        tracing::info!("Waiting for mixnet listener to finish");
        let tun_device_sink = mixnet_listener_handle
            .await
            .map_err(MixnetError::JoinMixnetListener)?;

        tracing::debug!("Exiting");
        Ok(tun_device_sink
            .reunite(tun_device_stream)
            .expect("reunite should work because of same device split")
            .into_inner())
    }
}

fn bundle_packet(
    packet: IprPacket,
    packet_bundler: &mut MultiIpPacketCodec,
) -> Result<Option<Bytes>, MixnetError> {
    let mut bundled_packets = BytesMut::new();
    packet_bundler
        .encode(packet, &mut bundled_packets)
        .map_err(MixnetError::BundlePacket)?;
    if bundled_packets.is_empty() {
        Ok(None)
    } else {
        Ok(Some(bundled_packets.freeze()))
    }
}

async fn handle_packet(
    packet: IprPacket,
    packet_bundler: &mut MultiIpPacketCodec,
    input_message_creator: &ToIprDataRequest,
    mixnet_client_sender: &MixnetClientSender,
) -> Result<(), MixnetError> {
    let bundled_packets = match bundle_packet(packet, packet_bundler)? {
        Some(bundled_packets) => bundled_packets,
        None => return Ok(()),
    };

    let input_message = input_message_creator
        .to_input_message(&bundled_packets)
        .map_err(|err| MixnetError::CreateInputMessage(Box::new(err)))?;

    mixnet_client_sender
        .send(input_message)
        .await
        .map_err(|err| MixnetError::SendInputMessage(Box::new(err)))
}

struct ToIprDataRequest {
    recipient: Recipient,
}

impl ToIprDataRequest {
    fn new(recipient: IpPacketRouterAddress) -> Self {
        Self {
            recipient: recipient.into(),
        }
    }
}

impl MixnetMessageSinkTranslator for ToIprDataRequest {
    fn to_input_message(&self, bundled_ip_packets: &[u8]) -> Result<InputMessage, nym_sdk::Error> {
        let packets = BytesMut::from(bundled_ip_packets).freeze();
        let packet = IpPacketRequest::new_data_request(packets).to_bytes()?;
        let lane = TransmissionLane::General;
        let packet_type = None;
        // Create an anonymous message without any bundled SURBs. We supply SURBs separate from
        // sphinx packets that carry the actual data, since we try to keep the payload for IP
        // traffic contained within a single sphinx packet.
        let surbs = 0;
        Ok(
            InputMessage::new_anonymous(self.recipient, packet, surbs, lane, packet_type)
                .with_max_retransmissions(0),
        )
    }
}

pub async fn start_processor(
    config: MixnetProcessorConfig,
    dev: AsyncDevice,
    mixnet_client: MixnetClient,
    mixnet_shutdown_manager: ShutdownManager,
    connection_monitor: &ConnectionMonitorTask,
    cancel_token: CancellationToken,
) -> JoinHandle<Result<AsyncDevice, MixnetError>> {
    tracing::info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        mixnet_shutdown_manager,
        connection_monitor,
        config.ip_packet_router_address,
        config.our_ips,
        cancel_token,
    );

    tokio::spawn(async move {
        processor.run().await.inspect_err(|err| {
            tracing::error!("Mixnet processor error: {err}");
        })
    })
}
