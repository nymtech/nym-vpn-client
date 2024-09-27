// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sdk::TaskClient;
use nym_wg_gateway_client::{ErrorMessage, WgGatewayClient};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tracing::{trace, warn};

use crate::mixnet::SharedMixnetClient;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(10); // 10 seconds
const ASSUMED_BANDWIDTH_DEPLETION_RATE: u64 = 10 * 1024 * 1024; // 10 MB/s

fn update_dynamic_check_interval(remaining_bandwidth: u64) -> Option<Duration> {
    let estimated_depletion_secs = remaining_bandwidth / ASSUMED_BANDWIDTH_DEPLETION_RATE;
    // try and have 10 logs before depletion...
    let next_timeout_secs = estimated_depletion_secs / 10;
    if next_timeout_secs == 0 {
        return None;
    }
    // ... but not faster then the gateway bandwidth refresh
    if next_timeout_secs > DEFAULT_PEER_TIMEOUT_CHECK.as_secs() {
        Some(Duration::from_secs(next_timeout_secs))
    } else {
        Some(DEFAULT_PEER_TIMEOUT_CHECK)
    }
}

pub(crate) struct BandwidthController<C, St> {
    inner: nym_bandwidth_controller::BandwidthController<C, St>,
    shared_mixnet_client: SharedMixnetClient,
    wg_entry_gateway_client: WgGatewayClient,
    wg_exit_gateway_client: WgGatewayClient,
    shutdown: TaskClient,
}

impl<C, St> BandwidthController<C, St> {
    pub(crate) fn new(
        inner: nym_bandwidth_controller::BandwidthController<C, St>,
        shared_mixnet_client: SharedMixnetClient,
        wg_entry_gateway_client: WgGatewayClient,
        wg_exit_gateway_client: WgGatewayClient,
        shutdown: TaskClient,
    ) -> Self {
        BandwidthController {
            inner,
            shared_mixnet_client,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            shutdown,
        }
    }

    async fn check_bandwidth(&mut self, entry: bool) -> Option<Duration> {
        let wg_gateway_client = if entry {
            &mut self.wg_entry_gateway_client
        } else {
            &mut self.wg_exit_gateway_client
        };
        match wg_gateway_client.query_bandwidth().await {
            Err(e) => warn!("Error querying remaining bandwidth {:?}", e),
            Ok(Some(remaining_bandwidth)) => {
                match update_dynamic_check_interval(remaining_bandwidth) {
                    Some(new_duration) => {
                        return Some(new_duration);
                    }
                    None => {
                        // TODO: try to return this error in the JoinHandle instead
                        self.shutdown
                            .send_we_stopped(Box::new(ErrorMessage::OutOfBandwidth {
                                gateway_id: Box::new(*wg_gateway_client.auth_recipient().gateway()),
                                authenticator_address: Box::new(wg_gateway_client.auth_recipient()),
                            }));
                    }
                }
            }
            Ok(None) => {}
        }
        None
    }

    pub(crate) async fn run(mut self) {
        let mut timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));
        // Skip the first, immediate tick
        timeout_check_interval.next().await;
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                _ = self.shutdown.recv() => {
                    trace!("BandwidthController: Received shutdown");
                    self.shared_mixnet_client.clone().disconnect().await;
                }
                _ = timeout_check_interval.next() => {
                    let entry_duration = self.check_bandwidth(true).await;
                    let exit_duration = self.check_bandwidth(false).await;
                    if let Some(minimal_duration) = match (entry_duration, exit_duration) {
                        (Some(d1), Some(d2)) => {
                            if d1 < d2 {
                                Some(d1)
                            } else {
                                Some(d2)
                            }
                        },
                        (Some(d), None) => Some(d),
                        (None, Some(d)) => Some(d),
                        _ => None,
                    } {
                        timeout_check_interval = IntervalStream::new(tokio::time::interval(minimal_duration));
                        // Skip the first, immediate tick
                        timeout_check_interval.next().await;
                    }
                }
            }
        }
    }
}
