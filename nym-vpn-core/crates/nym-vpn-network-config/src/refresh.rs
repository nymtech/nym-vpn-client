// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use nym_common::trace_err_chain;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    Error, Network, NymNetwork, Result, discovery::Discovery, envs::RegisteredNetworks,
    network_from_discovery,
};

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

struct FileRefresher {
    config_path: PathBuf,
    discovery_refresher_tx: tokio::sync::mpsc::Sender<Result<Network>>,
    cancel_token: CancellationToken,
}

impl FileRefresher {
    fn new(
        config_path: PathBuf,
        discovery_refresher_tx: tokio::sync::mpsc::Sender<Result<Network>>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            config_path,
            discovery_refresher_tx,
            cancel_token,
        }
    }

    async fn refresh_discovery_file(&self, network_name: &str) -> Result<Option<Discovery>> {
        if Discovery::path_is_stale(self.config_path.as_path(), network_name)? {
            let discovery = Discovery::fetch(network_name).await?;
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

    async fn run(self, mut network: Network) {
        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        let mut checked_consistency = false;

        self.cancel_token
            .run_until_cancelled(async {
                // no initial tick outside the loop, since we're only running in Connected state,
                // we want to try to refresh asap and make the most of this iteration
                loop {
                    interval.tick().await;
                    if !checked_consistency {
                        match network.check_consistency().await {
                            // This is probably because of network unavailability, so consistency can't be checked yet
                            Err(e) => tracing::warn!("Could not check consistency: {e:?}"),
                            Ok(false) => {
                                tracing::error!("Inconsistent network");
                                self.discovery_refresher_tx
                                    .send(Err(Error::InconsistentNetwork))
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
                            self.discovery_refresher_tx
                                .send(Err(Error::RefreshDiscoveryFile))
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
                                    trace_err_chain!(
                                        err,
                                        "Failed to parse refreshed discovery file"
                                    );
                                    self.discovery_refresher_tx
                                        .send(Err(Error::ParseDiscoveryFile))
                                        .await
                                        .ok();
                                }
                                Ok(new_network) => {
                                    network = new_network.clone();
                                    self.discovery_refresher_tx.send(Ok(new_network)).await.ok();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .await;
    }
}

// Ideally we only refresh the discovery file when the tunnel is up
pub fn start_background_file_refresh(
    config_path: PathBuf,
    network: Network,
    discovery_refresher_tx: tokio::sync::mpsc::Sender<Result<Network>>,
    cancel_token: CancellationToken,
) -> JoinHandle<()> {
    let refresher = FileRefresher::new(config_path, discovery_refresher_tx, cancel_token);
    tokio::spawn(refresher.run(network))
}
