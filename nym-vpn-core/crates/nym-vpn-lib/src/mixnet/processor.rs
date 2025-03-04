// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::result::Result;

use bytes::Bytes;
use futures::{channel::mpsc, StreamExt};
use nym_connection_monitor::{ConnectionMonitorTask, ConnectionStatusEvent};
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, v8::request::IpPacketRequest};
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::{InputMessage, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use tokio::{sync::oneshot, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace};
use tun::{AsyncDevice, Device};

use super::MixnetError;

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) ip_packet_router_address: Recipient,
}

impl Config {
    pub(crate) fn new(ip_packet_router_address: Recipient) -> Self {
        Config {
            ip_packet_router_address,
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

    fn create_data_message(&self, bundled_packets: Bytes) -> Result<InputMessage, MixnetError> {
        let packet = IpPacketRequest::new_data_request(bundled_packets).to_bytes()?;

        let lane = TransmissionLane::General;
        let packet_type = None;
        // Create an anonymous message without any bundled SURBs. We supply SURBs separate from
        // sphinx packets that carry the actual data, since we try to keep the payload for IP
        // traffic contained within a single sphinx packet.
        let surbs = 0;
        let input_message =
            InputMessage::new_anonymous(self.recipient, packet, surbs, lane, packet_type);
        Ok(input_message)
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
    ip_packet_router_address: Recipient,

    // Our IP addresses
    our_ips: nym_ip_packet_requests::IpPair,

    // Identifier for ICMP beacon, so we can check incoming ICMP packets to see if we should
    // forward them to the connection monitor
    icmp_beacon_identifier: u16,

    // Listen for when we should disconnect from the IPR and being shutting down
    cancel_token: CancellationToken,

    // Once we've disconnected from the IPR, we need to notify the connection monitor
    is_disconnected_from_ipr_tx: oneshot::Sender<()>,
}

impl MixnetProcessor {
    fn new(
        device: AsyncDevice,
        mixnet_client: SharedMixnetClient,
        connection_monitor: &ConnectionMonitorTask,
        ip_packet_router_address: Recipient,
        our_ips: nym_ip_packet_requests::IpPair,
        cancel_token: CancellationToken,
        ipr_disconnect_tx: oneshot::Sender<()>,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            connection_event_tx: connection_monitor.event_sender(),
            ip_packet_router_address,
            our_ips,
            icmp_beacon_identifier: connection_monitor.icmp_beacon_identifier(),
            cancel_token,
            is_disconnected_from_ipr_tx: ipr_disconnect_tx,
        }
    }

    async fn run(
        self,
        mut task_client_mix_processor: TaskClient,
        task_client_mix_listener: TaskClient,
    ) -> Result<AsyncDevice, MixnetError> {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name().unwrap(),
        );

        debug!("Splitting tun device into sink and stream");
        let (tun_device_sink, mut tun_device_stream) = self.device.into_framed().split();

        debug!("Split mixnet sender");
        let sender = self.mixnet_client.split_sender().await;
        let recipient = self.ip_packet_router_address;

        let mut multi_ip_packet_encoder =
            MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

        let message_creator = MessageCreator::new(recipient);

        // Listen for when the mixnet listener is done
        let (mixnet_listener_done_tx, mixnet_listener_done) = oneshot::channel();
        tokio::pin!(mixnet_listener_done);

        // Starting the mixnet listener.
        debug!("Starting mixnet listener");
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

        info!("Mixnet processor is running");
        while !task_client_mix_processor.is_shutdown() {
            tokio::select! {
                _ = self.cancel_token.cancelled(), if !has_sent_ipr_disconnect => {
                    info!("MixnetProcessor: Cancel token triggered, sending disconnect message");
                    let input_message = match message_creator.create_disconnect_message() {
                        Ok(input_message) => input_message,
                        Err(err) => {
                            error!("Failed to create disconnect message: {err}");
                            continue;
                        }
                    };
                    if let Err(err) = sender.send(input_message).await {
                        error!("Failed to send disconnect message: {err}");
                    }
                    has_sent_ipr_disconnect = true;
                }
                _ = &mut mixnet_listener_done => {
                    info!("MixnetProcessor: mixnet_listener has finished");
                    break;
                }
                _ = task_client_mix_processor.recv_with_delay() => {
                    info!("MixnetProcessor: Received shutdown");
                    break;
                }
                // To make sure we don't wait too long before filling up the buffer, which destroys
                // latency, cap the time waiting for the buffer to fill
                Some(bundled_packets) = multi_ip_packet_encoder.buffer_timeout() => {
                    assert!(!bundled_packets.is_empty());

                    match message_creator.create_data_message(bundled_packets) {
                        Ok(input_message) => {
                            tokio::select! {
                                ret = sender.send(input_message) => {
                                    if ret.is_err() && !task_client_mix_processor.is_shutdown_poll() {
                                        error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                                    }
                                }
                                _ = task_client_mix_processor.recv_with_delay() => {
                                    info!("MixnetProcessor: Received shutdown while sending.");
                                    break;
                                }
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
                        .append_packet(packet.into_bytes())
                    {
                        match message_creator.create_data_message(input_message) {
                            Ok(input_message) => {
                                tokio::select! {
                                    ret = sender.send(input_message) => {
                                        if ret.is_err() && !task_client_mix_processor.is_shutdown_poll() {
                                            error!("Could not forward IP packet to the mixnet. The packet(s) will be dropped.");
                                        }
                                    }
                                    _ = task_client_mix_processor.recv_with_delay() => {
                                        info!("MixnetProcessor: Received shutdown while sending.");
                                        break;
                                    }
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

        tracing::info!("Sending that ipr is disconnected");
        self.is_disconnected_from_ipr_tx.send(()).unwrap();

        task_client_mix_processor.recv_timeout().await;

        tracing::info!("MixnetProcessor: Exiting");
        Ok(tun_device_sink
            .reunite(tun_device_stream)
            .expect("reunite should work because of same device split")
            .into_inner())
    }
}

pub(crate) async fn start_processor(
    config: Config,
    dev: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    task_manager: &TaskManager,
    our_ips: nym_ip_packet_requests::IpPair,
    connection_monitor: &ConnectionMonitorTask,
    ipr_cancel_token: CancellationToken,
    ipr_disconnect_tx: oneshot::Sender<()>,
) -> JoinHandle<Result<AsyncDevice, MixnetError>> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        connection_monitor,
        config.ip_packet_router_address,
        our_ips,
        ipr_cancel_token,
        ipr_disconnect_tx,
    );

    // This is an unfortunate limitation of the TaskManager/TaskClient. Would be better if we could
    // have child clients like with tokio::CancellationToken, that can be crated from the parent
    let task_client_mix_processor = task_manager.subscribe_named("mixnet_processor");
    let task_client_mix_listener = task_manager.subscribe_named("mixnet_listener");

    tokio::spawn(async move {
        let ret = processor
            .run(task_client_mix_processor, task_client_mix_listener)
            .await;
        if let Err(err) = ret {
            error!("Mixnet processor error: {err}");
            Err(err)
        } else {
            ret
        }
    })
}
