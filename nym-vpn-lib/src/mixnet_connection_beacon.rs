// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_ip_packet_requests::request::IpPacketRequest;
use nym_sdk::mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient};
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::error::Result;

pub struct MixnetConnectionBeacon {
    pub mixnet_client_sender: MixnetClientSender,
    pub our_address: Recipient,
}

impl MixnetConnectionBeacon {
    async fn send_beep(&self) -> Result<u64> {
        let (request, request_id) = IpPacketRequest::new_ping(self.our_address);
        let input_message = InputMessage::new_regular(
            self.our_address,
            request.to_bytes().unwrap(),
            TransmissionLane::General,
            None,
        );
        self.mixnet_client_sender.send(input_message).await?;
        Ok(request_id)
    }

    pub async fn run(self, mut shutdown: TaskClient) -> Result<()> {
        info!("Mixnet connection beacon is running");
        loop {
            tokio::select! {
                _ = shutdown.recv_with_delay() => {
                    info!("MixnetConnectionBeacon: Received shutdown");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    log::info!("BEEP");
                    let _ping_id = match self.send_beep().await {
                        Ok(id) => id,
                        Err(err) => {
                            error!("Failed to send ping: {err}");
                            continue;
                        }
                    };
                }
            }
        }
        info!("MixnetConnectionBeacon: Exiting");
        Ok(())
    }
}

pub fn start_mixnet_connection_beacon(
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    info!("Creating mixnet connection beacon");
    let beacon = MixnetConnectionBeacon {
        mixnet_client_sender,
        our_address,
    };
    tokio::spawn(async move {
        let ret = beacon.run(shutdown_listener).await;
        if let Err(err) = ret {
            error!("Mixnet connection beacon error: {err}");
            Err(err)
        } else {
            ret
        }
    })
}
