// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::result::Result;

use bytes::Bytes;
use futures::{channel::mpsc, StreamExt};
use nym_connection_monitor::{ConnectionMonitorTask, ConnectionStatusEvent};
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, v7::request::IpPacketRequest};
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::{InputMessage, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient, TaskManager};
use tokio::task::JoinHandle;
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

    fn create_input_message(&self, bundled_packets: Bytes) -> Result<InputMessage, MixnetError> {
        let packet = IpPacketRequest::new_data_request(bundled_packets).to_bytes()?;

        let lane = TransmissionLane::General;
        let packet_type = None;
        let input_message = InputMessage::new_regular(self.recipient, packet, lane, packet_type);
        Ok(input_message)
    }
}

struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: SharedMixnetClient,
    connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,
    ip_packet_router_address: Recipient,
    our_ips: nym_ip_packet_requests::IpPair,
    icmp_beacon_identifier: u16,
}

impl MixnetProcessor {
    fn new(
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

        // Starting the mixnet listener.
        // NOTE: we are cloning the shutdown handle here, which is not ideal. What we actually need
        // is another subscription from the TaskManager to be able to listen to the shutdown event
        // in both tasks independently.
        debug!("Starting mixnet listener");
        let mixnet_listener = super::mixnet_listener::MixnetListener::new(
            self.mixnet_client,
            task_client_mix_listener,
            tun_device_sink,
            self.icmp_beacon_identifier,
            self.our_ips,
            self.connection_event_tx.clone(),
        )
        .await;
        let mixnet_listener_handle = mixnet_listener.start();

        info!("Mixnet processor is running");
        while !task_client_mix_processor.is_shutdown() {
            tokio::select! {
                _ = task_client_mix_processor.recv_with_delay() => {
                    trace!("MixnetProcessor: Received shutdown");
                    break;
                }
                // To make sure we don't wait too long before filling up the buffer, which destroys
                // latency, cap the time waiting for the buffer to fill
                Some(bundled_packets) = multi_ip_packet_encoder.buffer_timeout() => {
                    assert!(!bundled_packets.is_empty());

                    match message_creator.create_input_message(bundled_packets) {
                        Ok(input_message) => {
                            tokio::select! {
                                ret = sender.send(input_message) => {
                                    if ret.is_err() && !task_client_mix_processor.is_shutdown_poll() {
                                        error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                                    }
                                }
                                _ = task_client_mix_processor.recv_with_delay() => {
                                    trace!("MixnetProcessor: Received shutdown while sending.");
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
                        match message_creator.create_input_message(input_message) {
                            Ok(input_message) => {
                                tokio::select! {
                                    ret = sender.send(input_message) => {
                                        if ret.is_err() && !task_client_mix_processor.is_shutdown_poll() {
                                            error!("Could not forward IP packet to the mixnet. The packet(s) will be dropped.");
                                        }
                                    }
                                    _ = task_client_mix_processor.recv_with_delay() => {
                                        trace!("MixnetProcessor: Received shutdown while sending.");
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

        debug!("MixnetProcessor: Exiting");
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
) -> JoinHandle<Result<AsyncDevice, MixnetError>> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(
        dev,
        mixnet_client,
        connection_monitor,
        config.ip_packet_router_address,
        our_ips,
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
