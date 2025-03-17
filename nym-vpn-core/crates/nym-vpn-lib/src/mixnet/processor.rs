// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::result::Result;

use bytes::{Bytes, BytesMut};
use futures::{channel::mpsc, StreamExt};
use nym_connection_monitor::{ConnectionMonitorTask, ConnectionStatusEvent};
use nym_gateway_directory::IpPacketRouterAddress;
use nym_ip_packet_requests::{
    codec::{IprPacket, MultiIpPacketCodec},
    v8::request::IpPacketRequest,
    IpPair,
};
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::{
    InputMessage, MixnetClientSender, MixnetMessageSender, MixnetMessageSinkTranslator, Recipient,
};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use tokio::{sync::oneshot, task::JoinHandle};
use tokio_util::{codec::Encoder, sync::CancellationToken};
use tun::{AsyncDevice, Device};

use super::MixnetError;

#[derive(Debug)]
pub(crate) struct MixnetProcessorConfig {
    pub(crate) ip_packet_router_address: IpPacketRouterAddress,
    our_ips: IpPair,
}

impl MixnetProcessorConfig {
    pub(crate) fn new(ip_packet_router_address: IpPacketRouterAddress, our_ips: IpPair) -> Self {
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
    mixnet_client: SharedMixnetClient,

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

    // Once we've disconnected from the IPR, we need to notify the connection monitor
    notify_disconnected: oneshot::Sender<()>,
}

impl MixnetProcessor {
    fn new(
        device: AsyncDevice,
        mixnet_client: SharedMixnetClient,
        connection_monitor: &ConnectionMonitorTask,
        ip_packet_router_address: IpPacketRouterAddress,
        our_ips: IpPair,
        cancel_token: CancellationToken,
        notify_disconnected: oneshot::Sender<()>,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            connection_event_tx: connection_monitor.event_sender(),
            ip_packet_router_address,
            our_ips,
            icmp_beacon_identifier: connection_monitor.icmp_beacon_identifier(),
            cancel_token,
            notify_disconnected,
        }
    }

    async fn run(
        self,
        mut task_client_mix_processor: TaskClient,
        task_client_mix_listener: TaskClient,
    ) -> Result<AsyncDevice, MixnetError> {
        tracing::info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name().unwrap(),
        );

        tracing::debug!("Splitting tun device into sink and stream");
        let (tun_device_sink, mut tun_device_stream) = self.device.into_framed().split();

        tracing::debug!("Split mixnet sender");
        let mixnet_sender = self.mixnet_client.split_sender().await;

        let lane_queue_lengths = self.mixnet_client.shared_lane_queue_lengths().await;

        let message_creator = MessageCreator::new(self.ip_packet_router_address.into());

        // Listen for when the mixnet listener is done
        let (mixnet_listener_done_tx, mixnet_listener_done) = oneshot::channel();
        tokio::pin!(mixnet_listener_done);

        // Starting the mixnet listener.
        tracing::debug!("Starting mixnet listener");
        let mixnet_listener = super::mixnet_listener::MixnetListener::new(
            self.mixnet_client.clone(),
            task_client_mix_listener,
            tun_device_sink,
            self.icmp_beacon_identifier,
            self.our_ips,
            self.connection_event_tx.clone(),
        )
        .await;
        let mixnet_listener_handle = mixnet_listener.start(mixnet_listener_done_tx);

        // Keep track of whether we've sent the disconnect message, so we don't send it multiple
        // times
        let mut has_sent_ipr_disconnect = false;

        let mut payload_topup_interval =
            tokio::time::interval(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

        let mut packet_bundler = MultiIpPacketCodec::new();

        let input_message_creator = ToIprDataRequest::new(self.ip_packet_router_address);

        tracing::info!("Mixnet processor is running");
        while !task_client_mix_processor.is_shutdown() {
            tokio::select! {
                biased;
                // When we get the cancel token, send a disconnect message to the IPR. We keep
                // running until the mixnet listener receives the disconnect response, so we can
                // make sure we've fully disconnected before we return.
                _ = self.cancel_token.cancelled(), if !has_sent_ipr_disconnect => {
                    tracing::debug!("MixnetProcessor: cancel token triggered, sending disconnect message");
                    let input_message = match message_creator.create_disconnect_message() {
                        Ok(input_message) => input_message,
                        Err(err) => {
                            tracing::error!("Failed to create disconnect message: {err}");
                            continue;
                        }
                    };
                    if let Err(err) = mixnet_sender.send(input_message).await {
                        tracing::error!("Failed to send disconnect message: {err}");
                        continue;
                    }
                    has_sent_ipr_disconnect = true;
                }
                // When the mixnet listener receives the disconnect response, it will notify us
                // that it's done. This means we can now stop
                _ = &mut mixnet_listener_done => {
                    tracing::debug!("MixnetProcessor: mixnet listener has finished");
                    break;
                }
                // In the tunnel monitor, if it times out waiting to hear that we have
                // disconnected, it will call on the TaskManager to signal shutdown anyway. This is
                // captured here.
                _ = task_client_mix_processor.recv_with_delay() => {
                    tracing::debug!("MixnetProcessor: Received shutdown");
                    break;
                }
                // To make sure we don't wait too long before filling up the buffer, which destroys
                // latency, cap the time waiting for the buffer to fill
                _ = payload_topup_interval.tick() => {
                    tracing::trace!("MixnetProcessor: Buffer timeout");

                    // Check the lane queue lengths, which are the pending packets idling in the
                    // Poisson process in the mixnet client. If we already have pending packets
                    // that we are waiting to send to the mixnet, there is no point in flushing the
                    // current buffer. Instead keep filling up so we can fit more IP packets in the
                    // mixnet packet payload.
                    let packet_queue = lane_queue_lengths.get(&TransmissionLane::General).unwrap_or_default();
                    if packet_queue > 0 {
                        tracing::trace!("Skipping payload topup timeout (queue: {packet_queue})");
                        continue;
                    }

                    tokio::select! {
                        ret = handle_packet(IprPacket::Flush, &mut packet_bundler, &input_message_creator, &mixnet_sender) => {
                            if ret.is_err() && !task_client_mix_processor.is_shutdown_poll() {
                                tracing::error!("Failed to flush the multi IP packet sink");
                            }
                        }
                        _ = task_client_mix_processor.recv_with_delay() => {
                            tracing::debug!("MixnetProcessor: Received shutdown while flushing");
                            break;
                        }
                    }
                }
                // Read from the tun device and send the IP packet to the mixnet
                Some(Ok(tun_packet)) = tun_device_stream.next() => {
                    payload_topup_interval.reset();
                    let packet = IprPacket::from(tun_packet.into_bytes());
                    tokio::select! {
                        ret = handle_packet(packet, &mut packet_bundler, &input_message_creator, &mixnet_sender) => {
                            if ret.is_err() && !task_client_mix_processor.is_shutdown_poll() {
                                tracing::error!("Failed to send IP packet to the mixnet");
                            }
                        }
                        _ = task_client_mix_processor.recv_with_delay() => {
                            tracing::debug!("MixnetProcessor: Received shutdown while sending.");
                            break;
                        }
                    }
                }
                // NOTE: this will basically never fire. If the tun device stream ends, the select
                // will still wait for the other branches to complete before this branch is taken.
                //
                // TODO: consider changing this so that a if tun_device_stream.next() returns None,
                // break out of the loop directly
                else => {
                    tracing::error!("Mixnet processor: tun device stream ended");
                    break;
                }
            }
        }

        tracing::info!("Waiting for mixnet listener to finish");
        let tun_device_sink = mixnet_listener_handle.await.unwrap();

        // Notify the tunnel monitor that we are disconnected from the IPR, and don't need the
        // mixnet connection anymore. The tunnel monitor will in turn call on the TaskManager to
        // signal shutdown for the mixnet client.
        if self.notify_disconnected.send(()).is_err() {
            tracing::error!("Failed to notify that the IPR is disconnected");
        } else {
            // After we've notified that we are disconnected, wait until the TaskManager has signelled
            // shutdown before we return.
            // Possibly we don't need this, but it seems like the correct thing to do. The footgun here
            // is that we don't want to drop the mixnet client before the task manager has signalled
            // shutdown.
            task_client_mix_processor.recv_timeout().await;
        }

        tracing::debug!("MixnetProcessor: Exiting");
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
        .map_err(|source| MixnetError::FailedToBundlePacket { source })?;
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
        .map_err(|source| MixnetError::FailedToCreateInputMessage { source })?;

    mixnet_client_sender
        .send(input_message)
        .await
        .map_err(|source| MixnetError::FailedToSendInputMessage { source })
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

pub(crate) async fn start_processor(
    config: MixnetProcessorConfig,
    dev: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    task_manager: &TaskManager,
    connection_monitor: &ConnectionMonitorTask,
    cancel_token: CancellationToken,
    notify_disconnected: oneshot::Sender<()>,
) -> JoinHandle<Result<AsyncDevice, MixnetError>> {
    tracing::info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        connection_monitor,
        config.ip_packet_router_address,
        config.our_ips,
        cancel_token,
        notify_disconnected,
    );

    let task_client_mix_processor = task_manager.subscribe_named("mixnet_processor");
    let task_client_mix_listener = task_manager.subscribe_named("mixnet_listener");

    tokio::spawn(async move {
        processor
            .run(task_client_mix_processor, task_client_mix_listener)
            .await
            .inspect_err(|err| {
                tracing::error!("Mixnet processor error: {err}");
            })
    })
}
