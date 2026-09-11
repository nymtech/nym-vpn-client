// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{ops::Deref, pin::pin, result::Result, time::Duration};

use bytes::{Bytes, BytesMut};
use futures::{FutureExt, StreamExt, future::Fuse};
use nym_gateway_directory::IpPacketRouterAddress;
use nym_ip_packet_requests::{
    codec::{IprPacket, MultiIpPacketCodec},
    v8::request::IpPacketRequest,
};
use nym_sdk::mixnet::{
    EventReceiver, InputMessage, MixnetClient, MixnetClientSender, MixnetMessageSender,
    MixnetMessageSinkTranslator, Recipient, TransmissionLane,
};
use tokio::task::JoinHandle;
use tokio_util::{codec::Encoder, sync::CancellationToken};
use tun::{AbstractDevice, AsyncDevice};

use super::{MixnetError, backpressure::MixnetBackpressureMonitor};

/// How much time to wait for ipr disconnect before proceeding to shutdown.
const IPR_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// How much time to wait for the mixnet listener to finish
const MIXNET_LISTENER_TIMEOUT: Duration = Duration::from_secs(5);

/// Interval between attempts to send ipr disconnect
const IPR_DISCONNECT_RETRY_DELAY: Duration = Duration::from_millis(500);

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

    // The address of the IP packet router we're sending messages to
    ip_packet_router_address: IpPacketRouterAddress,

    // Listen for when we should disconnect from the IPR and being shutting down
    cancel_token: CancellationToken,

    // Receive events from the mixnet client
    event_rx: EventReceiver,
}

impl MixnetProcessor {
    pub fn spawn(
        device: AsyncDevice,
        mixnet_client: MixnetClient,
        ip_packet_router_address: IpPacketRouterAddress,
        cancel_token: CancellationToken,
        event_rx: EventReceiver,
    ) -> JoinHandle<Result<AsyncDevice, MixnetError>> {
        let processor = MixnetProcessor {
            device,
            mixnet_client,
            ip_packet_router_address,
            cancel_token,
            event_rx,
        };

        tokio::spawn(processor.run())
    }

    async fn run(self) -> Result<AsyncDevice, MixnetError> {
        tracing::info!(
            "Opened mixnet processor on tun device {}",
            self.device
                .deref()
                .tun_name()
                .unwrap_or("<unknown>".to_string()),
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
        let mixnet_client_shutdown_token = self.mixnet_client.cancellation_token().child_token();

        let message_creator = MessageCreator::new(self.ip_packet_router_address.into());

        // Starting the mixnet listener.
        tracing::debug!("Starting mixnet listener");
        // We need a dedicated token, because the listener needs to run a bit more after global shutdown
        let mixnet_listener_cancel_token = CancellationToken::new();
        let mut mixnet_listener_handle = super::mixnet_listener::MixnetListener::spawn(
            self.mixnet_client,
            tun_device_sink,
            mixnet_listener_cancel_token.clone(),
            self.event_rx,
        );

        // Keeps track of whether ipr disconnect timeout has been activated.
        let mut is_disconnect_timeout_set = false;
        let mut started_sleep_timeout = false;

        // Ipr disconnect timeout future set upon cancellation
        let ipr_disconnect_timeout = Fuse::terminated();
        let mut ipr_disconnect_timeout = pin!(ipr_disconnect_timeout);

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
                _ = self.cancel_token.cancelled() => {
                    // Start disconnect timeout upon receiving cancellation in the very first time.
                    if is_disconnect_timeout_set {
                        tracing::debug!("Re-sending disconnect message");
                    } else {
                        is_disconnect_timeout_set = true;
                        ipr_disconnect_timeout.set(tokio::time::sleep(IPR_DISCONNECT_TIMEOUT).fuse());
                        tracing::debug!("Cancel token triggered, sending disconnect message");
                    }

                    let input_message = match message_creator.create_disconnect_message() {
                        Ok(input_message) => input_message,
                        Err(err) => {
                            tracing::error!("Failed to create disconnect message: {err}");
                            tokio::time::sleep(IPR_DISCONNECT_RETRY_DELAY).await;
                            started_sleep_timeout = true;
                            continue;
                        }
                    };
                    //Fixme : bandaid, the disconnect logic has to be taken out of that loop and properly upper bounded
                    if let Err(err) = tokio::time::timeout(Duration::from_secs(2),mixnet_sender.send(input_message)).await {
                        tracing::error!("Failed to send disconnect message: {err}");
                        tokio::time::sleep(IPR_DISCONNECT_RETRY_DELAY).await;
                        started_sleep_timeout = true;
                        continue;
                    }

                    tracing::info!("Sent disconnect message");
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
                        let packet = IprPacket::from(tun_packet);
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
                                continue; // go through disconnect procedure
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
                            continue; // go through disconnect procedure
                        }
                    }
                }
            }
        }

        tracing::info!("Stopping mixnet backpressure monitor");
        backpressure_monitor.stop().await;

        tracing::info!("Waiting for mixnet listener to finish");

        // if loop terminates for some other reason without sending disconnect OR
        // we already started waiting some time, we force shutdown so we don't wait even more
        if !is_disconnect_timeout_set || started_sleep_timeout {
            mixnet_listener_cancel_token.cancel();
        }

        // Wait for the mixnet listener to finish, or force finish
        let mix_listener_result = tokio::select! {
            biased;
            tun_device = &mut mixnet_listener_handle => {
                tun_device
            },
            // backup timeout, just in case cancel didn't somehow get properly sent
            _ = tokio::time::sleep(MIXNET_LISTENER_TIMEOUT) => {
                    tracing::warn!("Timed out waiting for mixnet listener to finish");
                    mixnet_listener_cancel_token.cancel();
                    mixnet_listener_handle.await
            }
        };

        let tun_device_sink = mix_listener_result.map_err(MixnetError::JoinMixnetListener)?;

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
        Ok(InputMessage::new_anonymous(
            self.recipient,
            packet,
            surbs,
            lane,
            packet_type,
        ))
    }
}

pub async fn start_processor(
    ip_packet_router_address: IpPacketRouterAddress,
    dev: AsyncDevice,
    mixnet_client: MixnetClient,
    cancel_token: CancellationToken,
    event_rx: EventReceiver,
) -> JoinHandle<Result<AsyncDevice, MixnetError>> {
    tracing::info!("Creating mixnet processor");
    MixnetProcessor::spawn(
        dev,
        mixnet_client,
        ip_packet_router_address,
        cancel_token,
        event_rx,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_IPR_ADDRESS: &str = "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e";

    #[test]
    fn data_request_keeps_public_ipr_wire_format_with_normal_retransmissions() {
        let ipr_address = IpPacketRouterAddress::try_from_base58_string(TEST_IPR_ADDRESS).unwrap();
        let message_creator = ToIprDataRequest::new(ipr_address);
        let bundled_ip_packets = [0x45, 0, 0, 20, 0, 0, 0, 0];

        let expected_data = IpPacketRequest::new_data_request(
            BytesMut::from(bundled_ip_packets.as_slice()).freeze(),
        )
        .to_bytes()
        .unwrap();

        let message = message_creator
            .to_input_message(&bundled_ip_packets)
            .unwrap();

        let InputMessage::Anonymous {
            recipient,
            data,
            reply_surbs,
            lane,
            max_retransmissions,
        } = message
        else {
            panic!("IPR data requests must remain anonymous Mixnet messages");
        };

        assert_eq!(recipient, Recipient::from(ipr_address));
        assert_eq!(data, expected_data);
        assert_eq!(reply_surbs, 0);
        assert_eq!(lane, TransmissionLane::General);
        assert_eq!(max_retransmissions, None);
    }
}
