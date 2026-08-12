// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use async_trait::async_trait;
use nym_client_core::{NymTopology, client::topology_control::nym_api_provider::Config};
use nym_common::trace_err_chain;
use nym_http_api_client::{Url, UserAgent};
use nym_sdk::{NymApiTopologyProvider, TopologyProvider};
use nym_vpn_api_client::{ResolverOverrides, error::VpnApiClientError, fronted_http_client};
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
    },
    task::JoinHandle,
    time::Instant,
};
use tokio_util::sync::CancellationToken;

use crate::mixnet::{DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE};

#[derive(Debug, Clone)]
pub struct VpnTopologyServiceHandle {
    tx: UnboundedSender<Command>,
}

impl VpnTopologyServiceHandle {
    fn new(tx: UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    pub async fn fetch(&self) -> Option<NymTopology> {
        let (completion_tx, completion_rx) = oneshot::channel();
        if self.tx.send(Command::Fetch { completion_tx }).is_ok() {
            completion_rx.await.ok().flatten()
        } else {
            None
        }
    }

    pub async fn update_config(&self, min_mixnode_performance: u8, min_gateway_performance: u8) {
        let (completion_tx, completion_rx) = oneshot::channel();
        if self
            .tx
            .send(Command::UpdateConfig {
                min_mixnode_performance,
                min_gateway_performance,
                completion_tx,
            })
            .is_ok()
        {
            completion_rx.await.ok();
        }
    }

    pub fn make_topology_provider(&self) -> VpnTopologyProvider {
        VpnTopologyProvider::new(self.tx.clone())
    }

    /// Clear the cached topology. This should be called when the network environment changes.
    pub async fn clear_cache(&self) {
        let (completion_tx, completion_rx) = oneshot::channel();
        if self.tx.send(Command::ClearCache { completion_tx }).is_ok() {
            completion_rx.await.ok();
        }
    }
}

const DEFAULT_CONFIG: Config = Config {
    min_mixnode_performance: DEFAULT_MIN_MIXNODE_PERFORMANCE,
    min_gateway_performance: DEFAULT_MIN_GATEWAY_PERFORMANCE,
    use_extended_topology: false,
    ignore_egress_epoch_role: true,
};

const TOPOLOGY_EXPIRY: Duration = Duration::from_secs(60 * 5);

#[derive(Debug)]
pub struct VpnTopologyService {
    rx: UnboundedReceiver<Command>,
    latest_topology: Option<CachedTopology>,
    nym_api_urls: Vec<Url>,
    user_agent: UserAgent,
    config: Config,
    shutdown_token: CancellationToken,
}

impl VpnTopologyService {
    pub fn spawn(
        nym_api_urls: Vec<Url>,
        user_agent: UserAgent,
        shutdown_token: CancellationToken,
    ) -> (VpnTopologyServiceHandle, JoinHandle<()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = DEFAULT_CONFIG;
        let service = Self {
            rx,
            nym_api_urls,
            user_agent,
            latest_topology: None,
            config,
            shutdown_token,
        };

        let join_handle = tokio::spawn(service.run());
        let service_handle = VpnTopologyServiceHandle::new(tx);

        (service_handle, join_handle)
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(command) = self.rx.recv() => match command {
                    Command::Fetch { completion_tx } => {
                        completion_tx.send(self.get_cached_or_fetch().await).ok();
                    }
                    Command::UpdateConfig {
                        min_mixnode_performance,
                        min_gateway_performance,
                        completion_tx,
                    } => {
                        self.config.min_mixnode_performance = min_mixnode_performance;
                        self.config.min_gateway_performance = min_gateway_performance;
                        completion_tx.send(()).ok();
                    }
                    Command::ClearCache { completion_tx } => {
                        tracing::info!("Clearing topology cache");
                        self.latest_topology = None;
                        completion_tx.send(()).ok();
                    }
                },
                _ = self.shutdown_token.cancelled() => break,
            }
        }
    }

    /// Get cached topology or fetch one from network if cached topology has expired or if there is no cached topology.
    async fn get_cached_or_fetch(&mut self) -> Option<NymTopology> {
        if let Some(cached_topology) = self.latest_topology.as_ref() {
            if cached_topology.timestamp.elapsed() > TOPOLOGY_EXPIRY {
                tracing::info!("Topology cache has expired. Refreshing from network...");

                match self.fetch().await {
                    Ok(topology) => {
                        tracing::info!("Updated topology");
                        self.latest_topology.replace(CachedTopology {
                            topology: topology.clone(),
                            timestamp: Instant::now(),
                        });
                        Some(topology)
                    }
                    Err(err) => {
                        trace_err_chain!(err, "failed to fetch topology");
                        tracing::info!("Fallback to using cached topology");
                        Some(cached_topology.topology.clone())
                    }
                }
            } else {
                tracing::info!("Cached topology is still valid. Returning it");

                Some(cached_topology.topology.clone())
            }
        } else {
            tracing::info!("No topology cache found. Fetching from network...");

            match self.fetch().await {
                Ok(topology) => {
                    tracing::info!("Updated topology");
                    self.latest_topology.replace(CachedTopology {
                        topology: topology.clone(),
                        timestamp: Instant::now(),
                    });
                    Some(topology)
                }
                Err(err) => {
                    trace_err_chain!(err, "failed to fetch topology");
                    tracing::info!("No fallback topology available");
                    None
                }
            }
        }
    }

    /// Fetch new topology from the network.
    async fn fetch(&self) -> Result<NymTopology, VpnTopologyServiceError> {
        // Prune once and reuse for both the HTTP client and the topology provider, so the
        // provider's own list of API URLs doesn't reintroduce routes the client already knows
        // are unusable.
        let nym_api_urls = ResolverOverrides::resolve_and_prune(&self.nym_api_urls).await;

        let validator_client =
            fronted_http_client(nym_api_urls.clone(), Some(self.user_agent.clone()), None)
                .await
                .map_err(VpnTopologyServiceError::CreateHttpClient)?;

        let mut topology_provider = NymApiTopologyProvider::new(
            clone_config(&self.config),
            nym_api_urls.into_iter().map(Into::into).collect(),
            validator_client,
        );

        let topology = topology_provider
            .get_new_topology()
            .await
            .ok_or(VpnTopologyServiceError::NetworkError)?;

        Ok(topology)
    }
}

fn clone_config(config: &Config) -> Config {
    Config {
        min_mixnode_performance: config.min_mixnode_performance,
        min_gateway_performance: config.min_gateway_performance,
        use_extended_topology: config.use_extended_topology,
        ignore_egress_epoch_role: config.ignore_egress_epoch_role,
    }
}

enum Command {
    Fetch {
        completion_tx: Sender<Option<NymTopology>>,
    },
    UpdateConfig {
        min_mixnode_performance: u8,
        min_gateway_performance: u8,
        completion_tx: Sender<()>,
    },
    ClearCache {
        completion_tx: Sender<()>,
    },
}

#[derive(Debug)]
struct CachedTopology {
    topology: NymTopology,
    timestamp: Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum VpnTopologyServiceError {
    #[error("failed to create http client")]
    CreateHttpClient(VpnApiClientError),

    #[error("failed to fetch topology from network")]
    NetworkError,
}

#[derive(Debug, Clone)]
pub struct VpnTopologyProvider {
    tx: UnboundedSender<Command>,
}

impl VpnTopologyProvider {
    fn new(tx: UnboundedSender<Command>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl TopologyProvider for VpnTopologyProvider {
    async fn get_new_topology(&mut self) -> Option<NymTopology> {
        let (tx, rx) = oneshot::channel();

        if self.tx.send(Command::Fetch { completion_tx: tx }).is_ok() {
            rx.await.ok().flatten()
        } else {
            None
        }
    }
}
