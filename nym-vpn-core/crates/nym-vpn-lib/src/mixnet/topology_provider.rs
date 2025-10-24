// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use crate::mixnet::error::MixnetError;
use async_trait::async_trait;
use nym_http_api_client::{Url, UserAgent};
use nym_vpn_api_client::{ResolverOverrides, fronted_http_client};
use tokio::{
    sync::{
        Mutex, RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use nym_client_core::{NymTopology, client::topology_control::nym_api_provider::Config};
use nym_sdk::{NymApiTopologyProvider, TopologyProvider};

enum FetcherCommand {
    Fetch {
        response: oneshot::Sender<Option<NymTopology>>,
    },
    UpdateConfig {
        min_mixnode_performance: Option<u8>,
        min_gateway_performance: Option<u8>,
        resolver_overrides: ResolverOverrides,
        response: oneshot::Sender<()>,
    },
}

struct Fetcher {
    topology_provider: NymApiTopologyProvider,
    nym_api_urls: Vec<Url>,
    user_agent: UserAgent,
    command_rx: UnboundedReceiver<FetcherCommand>,
    cancel_token: CancellationToken,
}

impl Fetcher {
    const DEFAULT_CONFIG: Config = Config {
        min_mixnode_performance: 0,
        min_gateway_performance: 0,
        use_extended_topology: false,
        ignore_egress_epoch_role: true,
    };

    async fn new(
        nym_api_urls: Vec<Url>,
        user_agent: UserAgent,
        command_rx: UnboundedReceiver<FetcherCommand>,
        cancel_token: CancellationToken,
    ) -> Result<Self, MixnetError> {
        let validator_client =
            fronted_http_client(nym_api_urls.clone(), Some(user_agent.clone()), None, None)
                .await
                .map_err(MixnetError::CreateHTTPClient)?;

        let topology_provider = NymApiTopologyProvider::new(
            Self::DEFAULT_CONFIG,
            nym_api_urls.clone().into_iter().map(Into::into).collect(),
            validator_client,
        );

        Ok(Self {
            topology_provider,
            nym_api_urls,
            user_agent,
            command_rx,
            cancel_token,
        })
    }

    async fn fetch_topology(&mut self) -> Option<NymTopology> {
        self.topology_provider.get_new_topology().await
    }

    async fn update_config(
        &mut self,
        min_mixnode_performance: Option<u8>,
        min_gateway_performance: Option<u8>,
        resolver_overrides: &ResolverOverrides,
    ) -> Result<(), MixnetError> {
        let mut config = Self::DEFAULT_CONFIG;
        if let Some(min_mixnode_performance) = min_mixnode_performance {
            config.min_mixnode_performance = min_mixnode_performance;
        }
        if let Some(min_gateway_performance) = min_gateway_performance {
            config.min_gateway_performance = min_gateway_performance;
        }

        let validator_client = fronted_http_client(
            self.nym_api_urls.clone(),
            Some(self.user_agent.clone()),
            None,
            Some(resolver_overrides),
        )
        .await
        .map_err(MixnetError::CreateHTTPClient)?;

        self.topology_provider = NymApiTopologyProvider::new(
            config,
            self.nym_api_urls
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
            validator_client,
        );

        Ok(())
    }

    async fn handle_command(&mut self, cmd: FetcherCommand) -> Result<(), MixnetError> {
        match cmd {
            FetcherCommand::Fetch { response } => {
                let latest_topology = self.fetch_topology().await;
                let _ = response.send(latest_topology);
            }
            FetcherCommand::UpdateConfig {
                min_mixnode_performance,
                min_gateway_performance,
                resolver_overrides,
                response,
            } => {
                self.update_config(
                    min_mixnode_performance,
                    min_gateway_performance,
                    &resolver_overrides,
                )
                .await?;
                let _ = response.send(());
            }
        }

        Ok(())
    }

    async fn run(mut self) {
        while !self.cancel_token.is_cancelled() {
            tokio::select! {
                biased;
                _ = self.cancel_token.cancelled() => {
                   tracing::trace!("Topology Fetcher: Received shutdown");
                }
                Some(cmd) = self.command_rx.recv() => {
                    if let Err(err) = self.handle_command(cmd).await {
                        tracing::error!("Topology Fetcher: error handling command: {err:?}");
                        // Carry on
                    }
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
    pub async fn new(
        nym_api_urls: Vec<Url>,
        user_agent: UserAgent,
        use_network: bool,
        cancel_token: CancellationToken,
    ) -> Result<Self, MixnetError> {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let refresher = Fetcher::new(nym_api_urls, user_agent, command_rx, cancel_token).await?;
        tokio::spawn(refresher.run());

        Ok(Self {
            cached_topology: Arc::new(RwLock::new(CachedNymTopology {
                latest_topology: None,
                use_network,
            })),
            in_progress_fetch: Arc::new(Mutex::new(None)),
            command_tx,
        })
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
        resolver_overrides: ResolverOverrides,
    ) {
        let (signal_finished_tx, signal_finished_rx) = oneshot::channel();
        if self
            .command_tx
            .send(FetcherCommand::UpdateConfig {
                min_mixnode_performance,
                min_gateway_performance,
                resolver_overrides,
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
            // wait for the fetch to complete in cache
            self.maybe_wait_on_fetch().await;

            self.cached_topology.read().await.latest_topology.clone()
        } else {
            cached_topology.latest_topology
        }
    }
}
