// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_common::{ErrorExt, trace_err_chain};

use async_trait::async_trait;
use nym_client_core::{NymTopology, client::topology_control::nym_api_provider::Config};
use nym_http_api_client::{Url, UserAgent};
use nym_sdk::{NymApiTopologyProvider, TopologyProvider};
use nym_vpn_api_client::{ResolverOverrides, error::VpnApiClientError, fronted_http_client};
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot::{self, Sender},
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::mixnet::{DEFAULT_MIN_GATEWAY_PERFORMANCE, DEFAULT_MIN_MIXNODE_PERFORMANCE};

enum Command {
    Fetch {
        completion_tx: Sender<Option<NymTopology>>,
    },
    UpdateConfig {
        min_mixnode_performance: u8,
        min_gateway_performance: u8,
        resolver_overrides: Option<ResolverOverrides>,
        completion_tx: Sender<()>,
    },
}

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

    pub fn make_topology_provider(&self) -> VpnTopologyProvider {
        VpnTopologyProvider::new(self.tx.clone())
    }
}

const DEFAULT_CONFIG: Config = Config {
    min_mixnode_performance: DEFAULT_MIN_MIXNODE_PERFORMANCE,
    min_gateway_performance: DEFAULT_MIN_GATEWAY_PERFORMANCE,
    use_extended_topology: false,
    ignore_egress_epoch_role: true,
};

#[derive(Debug)]
struct VpnTopologyService {
    nym_api_urls: Vec<Url>,
    user_agent: UserAgent,
    latest_topology: Option<NymTopology>,
    shutdown_token: CancellationToken,
    rx: UnboundedReceiver<Command>,
}

impl VpnTopologyService {
    pub async fn spawn(
        nym_api_urls: Vec<Url>,
        user_agent: UserAgent,
        shutdown_token: CancellationToken,
    ) -> (VpnTopologyServiceHandle, JoinHandle<()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let service = Self {
            nym_api_urls,
            user_agent,
            latest_topology: None,
            shutdown_token,
            rx,
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
                        todo!();
                    }
                    Command::UpdateConfig {
                        min_mixnode_performance,
                        min_gateway_performance,
                        resolver_overrides,
                        completion_tx,
                    } => {
                        todo!();
                    }
                },
                _ = self.shutdown_token.cancelled() => break,
            }
        }
    }

    async fn fetch(&mut self) {
        let validator_client = fronted_http_client(
            self.nym_api_urls.clone(),
            Some(self.user_agent.clone()),
            None,
            None,
        )
        .await
        .expect("http client");

        let mut topology_provider = NymApiTopologyProvider::new(
            DEFAULT_CONFIG,
            self.nym_api_urls
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
            validator_client,
        );

        match topology_provider.get_new_topology().await {
            Some(new_topology) => {
                self.latest_topology.replace(new_topology);
            }
            None => {
                tracing::error!("Failed to fetch new topology");
            }
        }
    }
}

pub struct VpnTopologyProvider {
    tx: UnboundedSender<Command>,
}

impl VpnTopologyProvider {
    pub fn new(tx: UnboundedSender<Command>) -> Self {
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
