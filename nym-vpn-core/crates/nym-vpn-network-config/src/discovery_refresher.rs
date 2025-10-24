// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use crate::{
    Error, Network, NymNetwork, Result, discovery::Discovery, envs::RegisteredNetworks,
    network_from_discovery,
};
use nym_common::trace_err_chain;
use nym_vpn_api_client::{ResolverOverrides, VpnApiClient};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

struct DiscoveryRefresher {
    client: VpnApiClient,
    config_path: PathBuf,
    commands_rx: Receiver<DiscoveryRefresherCommand>,
    events_tx: Sender<DiscoveryRefresherEvent>,
    cancel_token: CancellationToken,
    current_resolver_overrides: Option<ResolverOverrides>,
    paused: bool,
}

impl DiscoveryRefresher {
    async fn new(
        config_path: PathBuf,
        commands_rx: Receiver<DiscoveryRefresherCommand>,
        events_tx: Sender<DiscoveryRefresherEvent>,
        cancel_token: CancellationToken,
    ) -> Result<Self> {
        let current_resolver_overrides = None;
        let client = Discovery::create_client(current_resolver_overrides).await?;
        Ok(Self {
            client,
            config_path,
            commands_rx,
            events_tx,
            cancel_token,
            current_resolver_overrides: current_resolver_overrides.cloned(),
            paused: false,
        })
    }

    async fn refresh_discovery_file(&self, network_name: &str) -> Result<Option<Discovery>> {
        if Discovery::path_is_stale(self.config_path.as_path(), network_name)? {
            let discovery = Discovery::fetch(&self.client, network_name).await?;
            discovery.write_to_file(self.config_path.as_path())?;
            Ok(Some(discovery))
        } else {
            Ok(None)
        }
    }

    async fn refresh_nym_network_file(
        &self,
        network_name: &str,
        discovery: &Discovery,
    ) -> Result<()> {
        if NymNetwork::path_is_stale(self.config_path.as_path(), network_name)? {
            discovery.update_nym_network_file(&self.config_path).await?;
        }

        Ok(())
    }

    async fn refresh_envs_file(&self) -> Result<()> {
        RegisteredNetworks::try_update_file(&self.config_path).await
    }

    async fn run(mut self, mut network: Network) {
        tracing::debug!("Discovery Refresher started");

        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        let mut checked_consistency = false;

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    tracing::debug!("Discovery refresher cancelled");
                    break;
                }
                Some(command) = self.commands_rx.recv() => {
                    match command {
                        DiscoveryRefresherCommand::Pause(pause) => {
                            if self.paused == pause {
                                tracing::debug!("Discovery Refresher already {}", if pause {"paused"} else {"resumed"} );
                            } else {
                                tracing::debug!("Discovery Refresher {}", if pause {"pausing"} else {"resuming"} );
                                self.paused = pause;
                            }
                        }
                        DiscoveryRefresherCommand::UseResolverOverrides(resolver_overrides) => {
                            if self.current_resolver_overrides.as_ref() == resolver_overrides.as_deref() {
                                tracing::debug!("Discovery Refresher received identical resolver overrides; ignoring");
                                continue;
                            }
                            let enabled = if resolver_overrides.is_some() {"enabled"} else {"disabled"};
                            tracing::debug!("Discovery Refresher using {enabled} resolver overrides");
                                match Discovery::create_client(resolver_overrides.as_deref()).await {
                                    Ok(client) => {self.client = client;},
                                    Err(err) => {
                                        tracing::error!("Failed to create new client with {enabled} resolver overrides: {err:?}");
                                        self.events_tx
                                            .send(DiscoveryRefresherEvent::Error(err))
                                            .await
                                            .ok();
                                    }
                                }
                        }
                    }
                }
                _ = interval.tick() => {
                    if self.paused {
                        continue;
                    }

                    if !checked_consistency {
                        match network.check_consistency().await {
                            Err(e) => tracing::warn!("Discovery refresher could not check consistency: {e:?}"),
                            Ok(false) => {
                                tracing::error!("Inconsistent network");
                                self.events_tx
                                    .send(DiscoveryRefresherEvent::Error(Error::InconsistentNetwork))
                                    .await
                                    .ok();
                            }
                            Ok(true) => {
                                checked_consistency = true;
                            }
                        }
                    }

                    if let Err(err) = self.refresh_envs_file().await {
                        trace_err_chain!(err, "Failed to refresh envs file");
                    }

                    match self
                        .refresh_discovery_file(&network.nym_network.network.network_name)
                        .await
                    {
                        Err(err) => {
                            trace_err_chain!(err, "Failed to refresh discovery file");
                            self.events_tx
                                .send(DiscoveryRefresherEvent::Error(Error::RefreshDiscoveryFile))
                                .await
                                .ok();
                        }
                        Ok(Some(discovery)) => {
                            if let Err(err) = self
                                .refresh_nym_network_file(
                                    &network.nym_network.network.network_name,
                                    &discovery,
                                )
                                .await
                            {
                                trace_err_chain!(err, "Failed to refresh nym network file");
                            }

                            match network_from_discovery(&self.config_path, discovery).await {
                                Err(err) => {
                                    trace_err_chain!(err, "Failed to parse refreshed discovery file");
                                    self.events_tx
                                        .send(DiscoveryRefresherEvent::Error(Error::ParseDiscoveryFile))
                                        .await
                                        .ok();
                                }
                                Ok(new_network) => {
                                    network = new_network.clone();
                                    self.events_tx
                                        .send(DiscoveryRefresherEvent::NewNetwork(Box::new(
                                            new_network,
                                        )))
                                        .await
                                        .ok();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        tracing::debug!("Discovery Refresher exiting");
    }
}

pub async fn start_discovery_refresher(
    config_path: PathBuf,
    network: Network,
    commands_rx: Receiver<DiscoveryRefresherCommand>,
    events_tx: Sender<DiscoveryRefresherEvent>,
    cancel_token: CancellationToken,
) -> Result<JoinHandle<()>> {
    let refresher =
        DiscoveryRefresher::new(config_path, commands_rx, events_tx, cancel_token).await?;

    let join_handle = tokio::spawn(refresher.run(network));

    Ok(join_handle)
}

#[derive(Debug)]
pub enum DiscoveryRefresherCommand {
    Pause(bool),
    UseResolverOverrides(Option<Box<ResolverOverrides>>),
}

#[derive(Debug)]
pub enum DiscoveryRefresherEvent {
    NewNetwork(Box<Network>),
    Error(Error),
}
