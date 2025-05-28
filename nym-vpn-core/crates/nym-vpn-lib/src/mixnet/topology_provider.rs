// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;
use tokio::{sync::RwLock, time::interval};
use tokio_util::sync::CancellationToken;
use url::Url;

use std::{sync::Arc, time::Duration};

use nym_client_core::NymTopology;
use nym_sdk::{NymApiTopologyProvider, TopologyProvider, UserAgent};

use crate::MixnetClientConfig;

const DEFAULT_TOPOLOGY_CACHE_TTL: Duration = Duration::from_secs(10 * 60);

struct Refresher {
    topology_provider: NymApiTopologyProvider,
    current_topology: Arc<RwLock<Option<NymTopology>>>,
    cancel_token: CancellationToken,
}

impl Refresher {
    fn new(
        topology_provider: NymApiTopologyProvider,
        current_topology: Arc<RwLock<Option<NymTopology>>>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            topology_provider,
            current_topology,
            cancel_token,
        }
    }

    async fn refresh_topology(&mut self) {
        *self.current_topology.write().await = self.topology_provider.get_new_topology().await;
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
                _ = full_refresh_interval.tick() => {
                    self.refresh_topology().await;
                }
            }
        }
    }
}

pub struct CachedTopologyProvider {
    current_topology: Arc<RwLock<Option<NymTopology>>>,
}

impl CachedTopologyProvider {
    pub async fn new(
        config: MixnetClientConfig,
        nym_api_url: Url,
        user_agent: Option<UserAgent>,
        cancel_token: CancellationToken,
    ) -> Self {
        let mut topology_provider =
            NymApiTopologyProvider::new(config, vec![nym_api_url], user_agent);
        let current_topology = Arc::new(RwLock::new(topology_provider.get_new_topology().await));
        let refresher = Refresher::new(topology_provider, current_topology.clone(), cancel_token);
        tokio::spawn(refresher.run());

        Self { current_topology }
    }
}

#[async_trait]
impl TopologyProvider for CachedTopologyProvider {
    async fn get_new_topology(&mut self) -> Option<NymTopology> {
        self.current_topology.read().await.clone()
    }
}
