// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_sdk::{
    mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient},
    TaskClient,
};
use nym_task::connections::TransmissionLane;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::{error::Result, nym_ip_packet_requests_current::request::IpPacketRequest};

const MIXNET_SELF_PING_INTERVAL: Duration = Duration::from_millis(1000);

struct MixnetConnectionBeacon {
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
}

impl MixnetConnectionBeacon {
    fn new(mixnet_client_sender: MixnetClientSender, our_address: Recipient) -> Self {
        MixnetConnectionBeacon {
            mixnet_client_sender,
            our_address,
        }
    }

    async fn send_mixnet_self_ping(&self) -> Result<u64> {
        trace!("Sending mixnet self ping");
        let (input_message, request_id) = create_self_ping(self.our_address);
        self.mixnet_client_sender.send(input_message).await?;
        Ok(request_id)
    }

    pub async fn run(self, mut shutdown: TaskClient) -> Result<()> {
        debug!("Mixnet connection beacon is running");
        let mut ping_interval = tokio::time::interval(MIXNET_SELF_PING_INTERVAL);
        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetConnectionBeacon: Received shutdown");
                    break;
                }
                _ = ping_interval.tick() => {
                    tokio::select! {
                        _ = shutdown.recv() => {
                            trace!("MixnetConnectionBeacon: Received shutdown");
                            break;
                        },
                        ping_result = self.send_mixnet_self_ping() => {
                            let _ping_id = match ping_result {
                                Ok(id) => id,
                                Err(err) => {
                                    error!("Failed to send mixnet self ping: {err}");
                                    continue;
                                }
                            };
                            // TODO: store ping_id to be able to monitor or ping timeouts
                        }
                    };
                }
            }
        }
        debug!("MixnetConnectionBeacon: Exiting");
        Ok(())
    }
}

pub fn create_self_ping(our_address: Recipient) -> (InputMessage, u64) {
    let (request, request_id) = IpPacketRequest::new_ping(our_address);
    (
        InputMessage::new_regular(
            our_address,
            request.to_bytes().unwrap(),
            TransmissionLane::General,
            None,
        ),
        request_id,
    )
}

pub fn start_mixnet_connection_beacon(
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating mixnet connection beacon");
    let beacon = MixnetConnectionBeacon::new(mixnet_client_sender, our_address);
    tokio::spawn(async move {
        beacon.run(shutdown_listener).await.inspect_err(|err| {
            error!("Mixnet connection beacon error: {err}");
        })
    })
}
