// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use async_trait::async_trait;
use tokio::{
    sync::{
        Mutex, RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use nym_client_core::{NymTopology, client::topology_control::nym_api_provider::Config};
use nym_sdk::{NymApiTopologyProvider, TopologyProvider};

enum FetcherCommand {
    Fetch {
        response: oneshot::Sender<Option<NymTopology>>,
    },
    UpdateConfig {
        min_mixnode_performance: Option<u8>,
        min_gateway_performance: Option<u8>,
        response: oneshot::Sender<()>,
    },
}

struct Fetcher {
    topology_provider: NymApiTopologyProvider,
    nym_api_urls: Vec<Url>,
    validator_client: nym_http_api_client::Client,
    command_rx: UnboundedReceiver<FetcherCommand>,
    cancel_token: CancellationToken,
}

impl Fetcher {
    const DEFAULT_CONFIG: Config = Config {
        min_mixnode_performance: 0,
        min_gateway_performance: 0,
        use_extended_topology: true,
        ignore_egress_epoch_role: true,
    };

    fn new(
        nym_api_urls: Vec<Url>,
        validator_client: nym_http_api_client::Client,
        command_rx: UnboundedReceiver<FetcherCommand>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            topology_provider: NymApiTopologyProvider::new(
                Self::DEFAULT_CONFIG,
                nym_api_urls.clone(),
                validator_client.clone(),
            ),
            nym_api_urls,
            validator_client,
            command_rx,
            cancel_token,
        }
    }

    async fn fetch_topology(&mut self) -> Option<NymTopology> {
        let topology = self.topology_provider.get_new_topology().await;
        if topology.is_none() {
            tracing::error!("VpnTopologyProvider: Failed to fetch topology from nym-api");
        }
        topology
    }

    fn update_config(
        &mut self,
        min_mixnode_performance: Option<u8>,
        min_gateway_performance: Option<u8>,
    ) {
        let mut config = Self::DEFAULT_CONFIG;
        if let Some(min_mixnode_performance) = min_mixnode_performance {
            config.min_mixnode_performance = min_mixnode_performance;
        }
        if let Some(min_gateway_performance) = min_gateway_performance {
            config.min_gateway_performance = min_gateway_performance;
        }
        self.topology_provider = NymApiTopologyProvider::new(
            config,
            self.nym_api_urls.clone(),
            self.validator_client.clone(),
        );
    }

    async fn handle_command(&mut self, cmd: FetcherCommand) {
        match cmd {
            FetcherCommand::Fetch { response } => {
                let latest_topology = self.fetch_topology().await;
                let _ = response.send(latest_topology);
            }
            FetcherCommand::UpdateConfig {
                min_mixnode_performance,
                min_gateway_performance,
                response,
            } => {
                self.update_config(min_mixnode_performance, min_gateway_performance);
                let _ = response.send(());
            }
        }
    }

    async fn run(mut self) {
        while !self.cancel_token.is_cancelled() {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                   tracing::trace!("Topology Fetcher: Received shutdown");
                }
                Some(cmd) = self.command_rx.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CachedNymTopology {
    latest_topology: Option<NymTopology>,
    use_network: bool,
}

#[derive(Debug, Clone)]
pub struct VpnTopologyProvider {
    cached_topology: Arc<RwLock<CachedNymTopology>>,
    in_progress_fetch: Arc<Mutex<Option<JoinHandle<()>>>>,
    command_tx: UnboundedSender<FetcherCommand>,
}

impl VpnTopologyProvider {
    pub fn new(
        nym_api_urls: Vec<Url>,
        validator_client: nym_http_api_client::Client,
        use_network: bool,
        cancel_token: CancellationToken,
    ) -> Self {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let refresher = Fetcher::new(nym_api_urls, validator_client, command_rx, cancel_token);
        tokio::spawn(refresher.run());

        Self {
            cached_topology: Arc::new(RwLock::new(CachedNymTopology {
                latest_topology: None,
                use_network,
            })),
            in_progress_fetch: Arc::new(Mutex::new(None)),
            command_tx,
        }
    }

    /// Get topology from network asynchronously, regardless of the set value of use_network
    pub async fn fetch(&self) {
        let mut in_progress_fetch = self.in_progress_fetch.lock().await;

        // Make sure we consume previous handle, if it's already finished
        in_progress_fetch.take_if(|handle| handle.is_finished());

        if in_progress_fetch.is_some() {
            return;
        }

        let (signal_finished_tx, signal_finished_rx) = oneshot::channel();
        if self
            .command_tx
            .send(FetcherCommand::Fetch {
                response: signal_finished_tx,
            })
            .is_err()
        {
            tracing::debug!("Fetcher terminated");
            return;
        }
        let cached_topology = self.cached_topology.clone();
        let handle = tokio::spawn(async move {
            if let Ok(latest_topology) = signal_finished_rx.await {
                cached_topology.write().await.latest_topology = latest_topology;
            } else {
                tracing::warn!("Could not fetch topology from network");
            }
        });
        *in_progress_fetch = Some(handle);
    }

    pub async fn maybe_wait_on_fetch(&self) {
        if let Some(handle) = self.in_progress_fetch.lock().await.take() {
            let _ = handle.await;
        }
    }

    pub async fn use_network(&mut self, use_network: bool) {
        self.cached_topology.write().await.use_network = use_network;
    }

    pub async fn update_config(
        &self,
        min_mixnode_performance: Option<u8>,
        min_gateway_performance: Option<u8>,
    ) {
        let (signal_finished_tx, signal_finished_rx) = oneshot::channel();
        if self
            .command_tx
            .send(FetcherCommand::UpdateConfig {
                min_mixnode_performance,
                min_gateway_performance,
                response: signal_finished_tx,
            })
            .is_err()
        {
            tracing::debug!("Fetcher terminated");
            return;
        }
        if signal_finished_rx.await.is_err() {
            tracing::warn!("Could not update topology provider configuration");
        }
    }
}

#[async_trait]
impl TopologyProvider for VpnTopologyProvider {
    async fn get_new_topology(&mut self) -> Option<NymTopology> {
        let cached_topology = self.cached_topology.read().await.clone();
        if cached_topology.use_network || cached_topology.latest_topology.is_none() {
            self.fetch().await;
            self.maybe_wait_on_fetch().await;

            let result = self.cached_topology.read().await.latest_topology.clone();
            if result.is_none() {
                tracing::error!("VpnTopologyProvider: Fetch completed but topology is None");
            }
            result
        } else {
            if cached_topology.latest_topology.is_none() {
                tracing::error!("VpnTopologyProvider: Cached topology is None!");
            }
            cached_topology.latest_topology
        }
    }
}
