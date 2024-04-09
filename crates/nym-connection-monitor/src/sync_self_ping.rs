// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use nym_ip_packet_requests::request::IpPacketRequest;
use nym_sdk::mixnet::{MixnetClient, MixnetMessageSender, Recipient};
use tracing::{debug, error};

use crate::{
    error::{Error, Result},
    mixnet_beacon::create_self_ping,
};

type SharedMixnetClient = Arc<tokio::sync::Mutex<Option<MixnetClient>>>;

// Send mixnet self ping and wait for the response
pub async fn self_ping_and_wait(
    our_address: Recipient,
    mixnet_client: SharedMixnetClient,
) -> Result<()> {
    let request_ids = send_self_pings(our_address, &mixnet_client).await?;
    wait_for_self_ping_return(&mixnet_client, &request_ids).await
}

async fn send_self_pings(
    our_address: Recipient,
    mixnet_client: &SharedMixnetClient,
) -> Result<Vec<u64>> {
    // Send pings
    let request_ids = futures::stream::iter(1..=3)
        .then(|_| async {
            let (input_message, request_id) = create_self_ping(our_address);
            mixnet_client
                .lock()
                .await
                .as_mut()
                .unwrap()
                .send(input_message)
                .await?;
            Ok::<u64, Error>(request_id)
        })
        .collect::<Vec<_>>()
        .await;

    // Check the vec of results and return the first error, if any. If there are not errors, unwrap
    // all the results into a vec of u64s.
    request_ids.into_iter().collect::<Result<Vec<_>>>()
}

async fn wait_for_self_ping_return(
    mixnet_client: &SharedMixnetClient,
    request_ids: &[u64],
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
                        // This is a common case when we are reconnecting to a gateway and receive
                        // all sorts of messages that are buffered since out last connection.
                        debug!("Failed to deserialize reconstructed message");
                        continue;
                    };
                    if request_ids.iter().any(|&id| response.id() == Some(id)) {
                        debug!("Got the ping we were waiting for");
                        return Ok(());
                    }
                }
            }
        }
    }
}
