// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sdk::TaskClient;

use crate::mixnet_connect::SharedMixnetClient;

pub struct BandwidthController {
    shared_mixnet_client: SharedMixnetClient,
    shutdown: TaskClient,
}

impl BandwidthController {
    pub fn new(shared_mixnet_client: SharedMixnetClient, shutdown: TaskClient) -> Self {
        BandwidthController {
            shared_mixnet_client,
            shutdown,
        }
    }

    pub async fn run(mut self) {
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
