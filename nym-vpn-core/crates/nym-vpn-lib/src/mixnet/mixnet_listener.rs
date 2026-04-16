// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{SinkExt, StreamExt, prelude::stream::SplitSink};
use nym_ip_packet_client::{IprListener, MixnetMessageOutcome};
use nym_sdk::mixnet::{EventReceiver, MixTrafficEvent, MixnetClient, MixnetClientEvent};
use tokio::task::JoinHandle;
use tokio_util::{codec::Framed, sync::CancellationToken};
use tun::{AsyncDevice, TunPacketCodec};

// The mixnet listener is responsible for listening for incoming mixnet messages from the mixnet
// client, and if they contain IP packets, forward them to the tun device.
pub(super) struct MixnetListener {
    // Mixnet client for receiving messages
    mixnet_client: MixnetClient,

    // IPR client for handling responses
    ipr_listener: IprListener,

    // Sink for sending packets to the tun device
    tun_device_sink: SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>>,

    // Cancellation token
    shutdown_token: CancellationToken,

    // Mixnet client event receiver
    event_rx: EventReceiver,
}

impl MixnetListener {
    pub(super) fn spawn(
        mixnet_client: MixnetClient,
        tun_device_sink: SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>>,
        shutdown_token: CancellationToken,
        event_rx: EventReceiver,
    ) -> JoinHandle<SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>>> {
        let ipr_listener = IprListener::new();
        let mixnet_listener = Self {
            mixnet_client,
            ipr_listener,
            tun_device_sink,
            shutdown_token,
            event_rx,
        };
        tokio::spawn(mixnet_listener.run())
    }

    // we exit the loop if :
    // - Processor tells us to
    // - Mixnet client crashed
    // - We received the disconnect ack
    // - Mixnet stream ended (it crashed)
    async fn run(mut self) -> SplitSink<Framed<AsyncDevice, TunPacketCodec>, Vec<u8>> {
        let mixnet_cancel_token = self.mixnet_client.cancellation_token().clone();
        loop {
            tokio::select! {
                biased;
                _ = self.shutdown_token.cancelled() => {
                    tracing::debug!("Mixnet listener: Received shutdown from processor");
                    break;
                }
                _ = mixnet_cancel_token.cancelled() => {
                    tracing::debug!("Mixnet listener: Mixnet client stopped");
                    break;
                }
                Some(event) = self.event_rx.next() => {
                    match event {
                        MixnetClientEvent::Traffic(MixTrafficEvent::FailedSendingSphinx) => break,
                    }
                }
                reconstructed_message = self.mixnet_client.next() => match reconstructed_message {
                    Some(reconstructed_message) => {
                        // We're just going to assume that all incoming messags are IPR messages
                        match self.ipr_listener.handle_reconstructed_message(reconstructed_message).await {
                            Ok(Some(MixnetMessageOutcome::IpPackets(packets))) => {
                                for packet in packets {
                                    // Consider not including packets that are ICMP ping replies to our beacon
                                    // in the responses. We are defensive here just in case we incorrectly
                                    // label real packets as ping replies to our beacon.
                                    if let Err(err) = self.tun_device_sink.send(packet.to_vec()).await {
                                        tracing::error!("Failed to send packet to tun device: {err}");
                                    }
                                }
                            }
                            Ok(Some(MixnetMessageOutcome::MixnetSelfPing)) => {
                                // Ignore self-ping messages
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

        if !self.mixnet_client.cancellation_token().is_cancelled() {
            tracing::info!("Disconnecting mixnet client");
            self.mixnet_client.disconnect().await;
        }

        tracing::debug!("Mixnet listener: Exiting");
        self.tun_device_sink
    }
}
