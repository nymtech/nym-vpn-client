// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_ip_packet_requests::request::IpPacketRequest;
use nym_sdk::mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient};
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::{
    error::{Error, Result},
    mixnet_connect::SharedMixnetClient,
};

const MIXNET_SELF_PING_INTERVAL: Duration = Duration::from_millis(1000);

struct MixnetConnectionBeacon {
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
}

fn create_self_ping(our_address: Recipient) -> (InputMessage, u64) {
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

// Send mixnet self ping and wait for the response
pub(crate) async fn self_ping_and_wait(
    our_address: Recipient,
    mixnet_client: SharedMixnetClient,
) -> Result<()> {
    let (input_message, request_id) = create_self_ping(our_address);
    mixnet_client.send(input_message).await?;
    wait_for_self_ping_return(&mixnet_client, request_id).await
}

async fn wait_for_self_ping_return(
    mixnet_client: &SharedMixnetClient,
    request_id: u64,
) -> Result<()> {
    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
    // to just grab ahold of the mutex and keep it until we get the response.
    let mut mixnet_client_handle = mixnet_client.lock().await;
    let mixnet_client = mixnet_client_handle.as_mut().unwrap();

    loop {
        tokio::select! {
            _ = &mut timeout => {
                error!("Timed out waiting for mixnet self ping to return");
                return Err(Error::TimeoutWaitingForConnectResponse);
            }
            Some(msgs) = mixnet_client.wait_for_messages() => {
                for msg in msgs {
                    let Ok(response) = IpPacketRequest::from_reconstructed_message(&msg) else {
                        // TODO: consider just not logging here since we expect this to be
                        // common when reconnecting to a gateway
                        error!("Failed to deserialize reconstructed message");
                        continue;
                    };
                    if response.id() == Some(request_id) {
                        debug!("Got the ping we were waiting for");
                        return Ok(());
                    }
                }
            }
        }
    }
}

impl MixnetConnectionBeacon {
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
                    let _ping_id = match self.send_mixnet_self_ping().await {
                        Ok(id) => id,
                        Err(err) => {
                            error!("Failed to send mixnet self ping: {err}");
                            continue;
                        }
                    };
                    // TODO: store ping_id to be able to monitor or ping timeouts
                }
            }
        }
        debug!("MixnetConnectionBeacon: Exiting");
        Ok(())
    }
}

pub fn start_mixnet_connection_beacon(
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating mixnet connection beacon");
    let beacon = MixnetConnectionBeacon {
        mixnet_client_sender,
        our_address,
    };
    tokio::spawn(async move {
        beacon.run(shutdown_listener).await.inspect_err(|err| {
            error!("Mixnet connection beacon error: {err}");
        })
    })
}
