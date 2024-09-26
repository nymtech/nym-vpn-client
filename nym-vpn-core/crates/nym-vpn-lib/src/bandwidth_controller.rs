// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sdk::TaskClient;

use crate::mixnet::SharedMixnetClient;

pub(crate) struct BandwidthController<C, St> {
    inner: nym_bandwidth_controller::BandwidthController<C, St>,
    shared_mixnet_client: SharedMixnetClient,
    shutdown: TaskClient,
}

impl<C, St> BandwidthController<C, St> {
    pub(crate) fn new(
        inner: nym_bandwidth_controller::BandwidthController<C, St>,
        shared_mixnet_client: SharedMixnetClient,
        shutdown: TaskClient,
    ) -> Self {
        BandwidthController {
            inner,
            shared_mixnet_client,
            shutdown,
        }
    }

    pub(crate) async fn run(mut self) {
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                _ = self.shutdown.recv() => {
                    log::trace!("BandwidthController: Received shutdown");
                    self.shared_mixnet_client.clone().disconnect().await;
                }
            }
        }
    }
}
