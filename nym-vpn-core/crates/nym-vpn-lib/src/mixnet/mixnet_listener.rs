// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bytes::Bytes;
use futures::{channel::mpsc, prelude::stream::SplitSink, SinkExt, StreamExt};
use nym_connection_monitor::{ConnectionStatusEvent, IcmpBeaconReply, Icmpv6BeaconReply};
use nym_ip_packet_client::{IprListener, MixnetMessageOutcome};
use nym_ip_packet_requests::IpPair;
use nym_mixnet_client::SharedMixnetClient;
use nym_task::TaskClient;
use tokio::{sync::oneshot, task::JoinHandle};
use tokio_util::codec::Framed;
use tun::{AsyncDevice, TunPacket, TunPacketCodec};

// The mixnet listener is responsible for listening for incoming mixnet messages from the mixnet
// client, and if they contain IP packets, forward them to the tun device.
pub(super) struct MixnetListener {
    // Mixnet client for receiving messages
    mixnet_client: SharedMixnetClient,

    // IPR client for handling responses
    ipr_listener: IprListener,

    // Task client for receiving shutdown signals
    task_client: TaskClient,

    // Sink for sending packets to the tun device
    tun_device_sink: SplitSink<Framed<AsyncDevice, TunPacketCodec>, TunPacket>,

    // Identifier for ICMP beacon
    icmp_beacon_identifier: u16,

    // Our IP addresses
    our_ips: IpPair,

    // Connection event sender
    connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,
}

impl MixnetListener {
    pub(super) async fn new(
        mixnet_client: SharedMixnetClient,
        task_client: TaskClient,
        tun_device_sink: SplitSink<Framed<AsyncDevice, TunPacketCodec>, TunPacket>,
        icmp_beacon_identifier: u16,
        our_ips: IpPair,
        connection_event_tx: mpsc::UnboundedSender<ConnectionStatusEvent>,
    ) -> Self {
        let ipr_listener = IprListener::new();
        Self {
            mixnet_client,
            ipr_listener,
            task_client,
            tun_device_sink,
            icmp_beacon_identifier,
            our_ips,
            connection_event_tx,
        }
    }

    fn send_connection_event(&self, event: ConnectionStatusEvent) {
        let res = self.connection_event_tx.unbounded_send(event);
        if res.is_err() && !self.task_client.is_shutdown() {
            tracing::error!("Failed to send connection event to connection monitor");
        }
    }

    fn check_for_icmp_beacon_reply(&self, packet: &Bytes) {
        if let Some(connection_event) =
            check_for_icmp_beacon_reply(packet, self.icmp_beacon_identifier, self.our_ips)
        {
            self.send_connection_event(connection_event);
        }
    }

    async fn run(mut self) -> SplitSink<Framed<AsyncDevice, TunPacketCodec>, TunPacket> {
        // We are the only one listening for mixnet messages when this is active
        let mut mixnet_client_binding = self.mixnet_client.lock().await;
        let mut mixnet_client = mixnet_client_binding.take().unwrap();

        while !self.task_client.is_shutdown() {
            tokio::select! {
                _ = self.task_client.recv_with_delay() => {
                    tracing::debug!("Mixnet listener: Received shutdown");
                    break;
                }
                reconstructed_message = mixnet_client.next() => match reconstructed_message {
                    Some(reconstructed_message) => {
                        // We're just going to assume that all incoming messags are IPR messages
                        match self.ipr_listener.handle_reconstructed_message(reconstructed_message).await {
                            Ok(Some(MixnetMessageOutcome::IpPackets(packets))) => {
                                for packet in packets {
                                    self.check_for_icmp_beacon_reply(&packet);

                                    // Consider not including packets that are ICMP ping replies to our beacon
                                    // in the responses. We are defensive here just in case we incorrectly
                                    // label real packets as ping replies to our beacon.
                                    if let Err(err) = self.tun_device_sink.send(TunPacket::new(packet.to_vec())).await {
                                        tracing::error!("Failed to send packet to tun device: {err}");
                                    }
                                }
                            }
                            Ok(Some(MixnetMessageOutcome::MixnetSelfPing)) => {
                                self.send_connection_event(ConnectionStatusEvent::MixnetSelfPing);
                            }
                            Ok(Some(MixnetMessageOutcome::Disconnect)) => {
                                tracing::debug!("Mixnet listener: Received disconnect message");
                                break;
                            }
                            Ok(None) => {}
                            Err(err) => {
                                tracing::error!("Mixnet listener: {err}");
                            }
                        }
                    },
                    None => {
                        tracing::error!("Mixnet listener: mixnet stream ended");
                        break;
                    }
                }
            }
        }

        // Restore the mixnet client
        mixnet_client_binding.replace(mixnet_client);

        tracing::debug!("Mixnet listener: Exiting");
        self.tun_device_sink
    }

    pub(super) fn start(
        self,
        is_done: oneshot::Sender<()>,
    ) -> JoinHandle<SplitSink<Framed<AsyncDevice, TunPacketCodec>, TunPacket>> {
        tokio::spawn(async move {
            let tun_device_sink = self.run().await;
            let _ = is_done.send(());
            tun_device_sink
        })
    }
}

fn check_for_icmp_beacon_reply(
    packet: &Bytes,
    icmp_beacon_identifier: u16,
    our_ips: IpPair,
) -> Option<ConnectionStatusEvent> {
    match nym_connection_monitor::is_icmp_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv4)
    {
        Some(IcmpBeaconReply::TunDeviceReply) => {
            tracing::trace!("Received ping response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv4IprTunDevicePingReply);
        }
        Some(IcmpBeaconReply::ExternalPingReply(_source)) => {
            tracing::trace!("Received ping response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv4IprExternalPingReply);
        }
        None => {}
    }

    match nym_connection_monitor::is_icmp_v6_beacon_reply(
        packet,
        icmp_beacon_identifier,
        our_ips.ipv6,
    ) {
        Some(Icmpv6BeaconReply::TunDeviceReply) => {
            tracing::trace!("Received ping v6 response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv6IprTunDevicePingReply);
        }
        Some(Icmpv6BeaconReply::ExternalPingReply(_source)) => {
            tracing::trace!("Received ping v6 response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv6IprExternalPingReply);
        }
        None => {}
    }

    None
}
