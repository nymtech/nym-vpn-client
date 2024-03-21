// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use futures::StreamExt;
use nym_ip_packet_requests::request::IpPacketRequest;
use nym_sdk::mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient};
use rand_distr::Distribution;
use tokio::{task::JoinHandle, time::Instant};
use tracing::{debug, error, trace};

use crate::{
    error::{Error, Result},
    mixnet_connect::SharedMixnetClient,
};

const MIXNET_SELF_PING_AVG_INTERVAL: Duration = Duration::from_millis(500);

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
    // We want to send a bunch of pings and wait for the first one to return
    let request_ids: Vec<_> = futures::stream::iter(1..=3)
        .then(|_| async {
            let (input_message, request_id) = create_self_ping(our_address);
            mixnet_client.send(input_message).await?;
            Ok::<u64, Error>(request_id)
        })
        .collect::<Vec<_>>()
        .await;
    // Check the vec of results and return the first error, if any. If there are not errors, unwrap
    // all the results into a vec of u64s.
    let request_ids = request_ids.into_iter().collect::<Result<Vec<_>>>()?;
    wait_for_self_ping_return(&mixnet_client, &request_ids).await
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
                        // TODO: consider just not logging here since we expect this to be
                        // common when reconnecting to a gateway
                        error!("Failed to deserialize reconstructed message");
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

struct PoissonDelayTimer<R> {
    rng: R,
    average_sending_delay_sec: f64,
}

impl<R> PoissonDelayTimer<R>
where
    R: nym_crypto::aes::cipher::crypto_common::rand_core::RngCore + std::marker::Send,
{
    fn new(rng: R, average_sending_delay: f64) -> Self {
        Self {
            rng,
            average_sending_delay_sec: average_sending_delay,
        }
    }

    // The stream will yield a value every `average_sending_delay` seconds on average.
    fn as_stream(&mut self) -> impl futures::Stream<Item = ()> + Send + '_ {
        futures::stream::unfold(
            (self, Instant::now()),
            |(poisson_delay_timer, last_instant)| async move {
                let average_sending_delay_sec = poisson_delay_timer.average_sending_delay_sec;
                let exp_dist = rand_distr::Exp::new(1.0 / average_sending_delay_sec).unwrap();
                let delay_duration = exp_dist.sample(&mut poisson_delay_timer.rng);
                let next_delay = Duration::from_secs_f64(delay_duration);
                dbg!(next_delay);

                // Make sure that the future yiels with an interval that isn't affected by the time
                // spent e.g in the select body
                tokio::time::sleep_until(last_instant + next_delay).await;

                Some(((), (poisson_delay_timer, Instant::now())))
            },
        )
    }
}

struct MixnetConnectionBeacon {
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
}

impl MixnetConnectionBeacon {
    async fn send_mixnet_self_ping(&self) -> Result<u64> {
        error!("Sending mixnet self ping");
        let (input_message, request_id) = create_self_ping(self.our_address);
        self.mixnet_client_sender.send(input_message).await?;
        Ok(request_id)
    }

    pub async fn run(self, mut shutdown: TaskClient) -> Result<()> {
        debug!("Mixnet connection beacon is running");
        // let mut ping_interval = tokio::time::interval(MIXNET_SELF_PING_INTERVAL);
        let rng = nym_crypto::aes::cipher::crypto_common::rand_core::OsRng;
        let mut timer = PoissonDelayTimer::new(rng, MIXNET_SELF_PING_AVG_INTERVAL.as_secs_f64());
        let timer_stream = timer.as_stream();
        tokio::pin!(timer_stream);

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetConnectionBeacon: Received shutdown");
                    break;
                }
                // _ = ping_interval.tick() => {
                _ = timer_stream.next() => {
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
