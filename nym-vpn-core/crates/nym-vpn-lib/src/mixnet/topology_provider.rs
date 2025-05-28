// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;
use tokio::{
    sync::{
        RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::interval,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use std::{sync::Arc, time::Duration};

use nym_client_core::NymTopology;
use nym_sdk::{NymApiTopologyProvider, TopologyProvider, UserAgent};

const DEFAULT_TOPOLOGY_CACHE_TTL: Duration = Duration::from_secs(10 * 60);

enum RefresherCommand {
    REFRESH,
}

struct Refresher {
    topology_provider: NymApiTopologyProvider,
    current_topology: Arc<RwLock<Option<NymTopology>>>,
    command_rx: UnboundedReceiver<(RefresherCommand, oneshot::Sender<()>)>,
    cancel_token: CancellationToken,
}

impl Refresher {
    fn new(
        topology_provider: NymApiTopologyProvider,
        current_topology: Arc<RwLock<Option<NymTopology>>>,
        command_rx: UnboundedReceiver<(RefresherCommand, oneshot::Sender<()>)>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            topology_provider,
            current_topology,
            command_rx,
            cancel_token,
        }
    }

    async fn refresh_topology(&mut self) {
        *self.current_topology.write().await = self.topology_provider.get_new_topology().await;
    }

    async fn handle_command(&mut self, cmd: RefresherCommand, signal_finish: oneshot::Sender<()>) {
        match cmd {
            RefresherCommand::REFRESH => self.refresh_topology().await,
        }
        let _ = signal_finish.send(());
    }

    async fn run(mut self) {
        let mut full_refresh_interval = interval(DEFAULT_TOPOLOGY_CACHE_TTL);
        full_refresh_interval.tick().await;

        while !self.cancel_token.is_cancelled() {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                   tracing::trace!("Topology Refresher: Received shutdown");
                }
                Some((cmd, signal_finish)) = self.command_rx.recv() => {
                    self.handle_command(cmd, signal_finish).await;
                }
                _ = full_refresh_interval.tick() => {
                    self.refresh_topology().await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachingTopologyProvider {
    current_topology: Arc<RwLock<Option<NymTopology>>>,
    command_tx: UnboundedSender<(RefresherCommand, oneshot::Sender<()>)>,
}

impl CachingTopologyProvider {
    pub fn new(
        nym_api_url: Url,
        user_agent: Option<UserAgent>,
        cancel_token: CancellationToken,
    ) -> Self {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let config = nym_client_core::client::topology_control::nym_api_provider::Config {
            min_mixnode_performance: 0,
            min_gateway_performance: 0,
            use_extended_topology: false,
            ignore_egress_epoch_role: true,
        };
        let topology_provider = NymApiTopologyProvider::new(config, vec![nym_api_url], user_agent);
        let current_topology = Arc::new(RwLock::new(None));
        let refresher = Refresher::new(
            topology_provider,
            current_topology.clone(),
            command_rx,
            cancel_token,
        );
        tokio::spawn(refresher.run());

        Self {
            current_topology,
            command_tx,
        }
    }

    pub async fn refresh(&self) {
        let (signal_finished_tx, signal_finished_rx) = oneshot::channel();
        if self
            .command_tx
            .send((RefresherCommand::REFRESH, signal_finished_tx))
            .is_err()
        {
            tracing::debug!("Refresher terminated");
            return;
        }
        let _ = signal_finished_rx.await;
    }
}

#[async_trait]
impl TopologyProvider for CachingTopologyProvider {
    async fn get_new_topology(&mut self) -> Option<NymTopology> {
        self.current_topology.read().await.clone()
    }
}
