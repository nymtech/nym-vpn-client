// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use nym_sdk::mixnet::{LaneQueueLengths, TransmissionLane};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const DEFAULT_MIXNET_BACKPRESSURE_THRESHOLD: usize = 4;

// The mixnet backpressure monitor is responsible for monitoring the queue length of the general
// transmission lane in the real traffic stream.
// If will return if there is backpressure detected, so that we can disable reading from the TUN
// device, and then notify once backpressure has lifted so we can continue.
pub(super) struct MixnetBackpressureMonitor {
    lane_queue_lengths: LaneQueueLengths,
    threshold: usize,
    notify_backpressure_lifted: Arc<tokio::sync::Notify>,
    cancel_token: CancellationToken,
}

impl MixnetBackpressureMonitor {
    pub(super) fn new(lane_queue_lengths: LaneQueueLengths, threshold: Option<usize>) -> Self {
        Self {
            lane_queue_lengths,
            threshold: threshold.unwrap_or(DEFAULT_MIXNET_BACKPRESSURE_THRESHOLD),
            notify_backpressure_lifted: Arc::new(tokio::sync::Notify::new()),
            cancel_token: CancellationToken::new(),
        }
    }

    fn queue_length(&self) -> usize {
        self.lane_queue_lengths
            .get(&TransmissionLane::General)
            .unwrap_or_default()
    }

    async fn run(self) {
        let mut check_interval = tokio::time::interval(Duration::from_millis(40));
        let mut previous_should_read = true;
        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    break;
                }
                _ = check_interval.tick() => {
                    let queue_len = self.queue_length();
                    let should_read = queue_len <= self.threshold;

                    if should_read && !previous_should_read {
                        self.notify_backpressure_lifted.notify_one();
                    }

                    previous_should_read = should_read;
                }
            }
        }
        tracing::debug!("Mixnet backpressure monitor exiting");
    }

    pub(super) fn start(self) -> MixnetBackpressureMonitorHandle {
        let cancel_token = self.cancel_token.clone();
        let threshold = self.threshold;
        let lane_queue_lengths = self.lane_queue_lengths.clone();
        let notify_backpressure_lifted = self.notify_backpressure_lifted.clone();

        let handle = tokio::spawn(self.run());

        MixnetBackpressureMonitorHandle {
            threshold,
            lane_queue_lengths,
            notify_backpressure_lifted,
            handle,
            cancel_token,
        }
    }
}

pub(super) struct MixnetBackpressureMonitorHandle {
    threshold: usize,
    lane_queue_lengths: LaneQueueLengths,
    notify_backpressure_lifted: Arc<tokio::sync::Notify>,
    handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

impl MixnetBackpressureMonitorHandle {
    pub(super) fn packet_queue_length(&self) -> usize {
        self.lane_queue_lengths
            .get(&TransmissionLane::General)
            .unwrap_or_default()
    }

    pub(super) fn is_backpressure(&self) -> bool {
        self.packet_queue_length() > self.threshold
    }

    pub(super) fn get_notify_backpressure_lifted(&self) -> Arc<tokio::sync::Notify> {
        self.notify_backpressure_lifted.clone()
    }

    pub(super) async fn stop(mut self) {
        self.cancel_token.cancel();
        tokio::select! {
            _ = &mut self.handle => {
                tracing::debug!("Mixnet backpressure monitor stopped");
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                tracing::error!("Failed to stop mixnet backpressure monitor: forcing");
                self.handle.abort();
            }
        }
    }
}
