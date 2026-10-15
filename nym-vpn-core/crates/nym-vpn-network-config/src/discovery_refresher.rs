// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use nym_common::trace_err_chain;
use nym_offline_monitor::ConnectivityMonitor;

use crate::{Network, NetworkCache};

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub struct DiscoveryRefresher {
    network_cache: NetworkCache,
    commands_rx: UnboundedReceiver<DiscoveryRefresherCommand>,
    events_tx: UnboundedSender<Box<Network>>,
    cancel_token: CancellationToken,
    paused: bool,
}

impl DiscoveryRefresher {
    pub fn spawn(
        network_cache: NetworkCache,
        commands_rx: UnboundedReceiver<DiscoveryRefresherCommand>,
        events_tx: UnboundedSender<Box<Network>>,
        connectivity_monitor: impl ConnectivityMonitor + 'static,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        let refresher = Self {
            network_cache,
            commands_rx,
            events_tx,
            cancel_token,
            paused: false,
        };

        tokio::spawn(refresher.run(connectivity_monitor))
    }

    async fn run(mut self, mut connectivity_monitor: impl ConnectivityMonitor + 'static) {
        tracing::debug!("Discovery Refresher started");

        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        let mut current_connectivity = connectivity_monitor.connectivity().await;

        loop {
            tokio::select! {
                Some(command) = self.commands_rx.recv() => {
                    match command {
                        DiscoveryRefresherCommand::Pause(pause) => {
                            if self.paused == pause {
                                tracing::debug!("Discovery refresher already {}", if pause {"paused"} else {"resumed"} );
                            } else {
                                tracing::debug!("Discovery refresher {}", if pause {"pausing"} else {"resuming"} );
                                self.paused = pause;
                            }
                        }
                    }
                }
                Some(connectivity) = connectivity_monitor.next() => {
                    current_connectivity = connectivity;
                }
                _ = interval.tick(), if !self.paused && current_connectivity.is_online() => {
                    match self.network_cache.fetch_if_stale().await {
                        Ok(()) => {
                            match self.network_cache.network() {
                                Ok(new_network) => {
                                    self.events_tx.send(new_network).ok();
                                }
                                Err(err) => {
                                    trace_err_chain!(err, "failed to obtain network");
                                }
                            }
                        }
                        Err(err) => {
                            trace_err_chain!(err, "failed to refresh network cache");
                        }
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    tracing::debug!("Discovery refresher cancelled");
                    break;
                }
            }
        }

        tracing::debug!("Discovery refresher exiting");
    }
}

#[derive(Debug)]
pub enum DiscoveryRefresherCommand {
    Pause(bool),
}
