// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use nym_common::trace_err_chain;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{Network, NymNetwork, Result, discovery::Discovery, envs::RegisteredNetworks};

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);

struct FileRefresher {
    config_path: PathBuf,
    network: Network,
    background_error_tx: tokio::sync::mpsc::Sender<()>,
    cancel_token: CancellationToken,
}

impl FileRefresher {
    fn new(
        config_path: PathBuf,
        network: Network,
        background_error_tx: tokio::sync::mpsc::Sender<()>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            config_path,
            network,
            background_error_tx,
            cancel_token,
        }
    }

    async fn refresh_discovery_file(&self) -> Result<Option<Discovery>> {
        if Discovery::path_is_stale(
            self.config_path.as_path(),
            &self.network.nym_network.network.network_name,
        )? {
            let discovery =
                Discovery::fetch(&self.network.nym_network.network.network_name).await?;
            discovery.write_to_file(self.config_path.as_path())?;
            Ok(Some(discovery))
        } else {
            Ok(None)
        }
    }

    async fn refresh_nym_network_file(&self, discovery: Discovery) -> Result<()> {
        if NymNetwork::path_is_stale(
            self.config_path.as_path(),
            &self.network.nym_network.network.network_name,
        )? {
            discovery.update_nym_network_file(&self.config_path).await?;
        }

        Ok(())
    }

    async fn refresh_envs_file(&self) -> Result<()> {
        RegisteredNetworks::try_update_file(&self.config_path).await
    }

    async fn run(self) {
        let mut interval = tokio::time::interval(CHECK_INTERVAL);
        let mut checked_consistency = false;

        self.cancel_token
            .run_until_cancelled(async {
                // no initial tick, since we're only running in Connected state, we have to make the most of it
                loop {
                    interval.tick().await;
                    if !checked_consistency {
                        match self.network.check_consistency().await {
                            // This is probably because of network unavailability, so consistency can't be checked yet
                            Err(e) => tracing::warn!("Could not check consistency: {e:?}"),
                            Ok(false) => {
                                tracing::error!("Inconsistent network");
                                self.background_error_tx.send(()).await.ok();
                                return;
                            }
                            Ok(true) => {
                                checked_consistency = true;
                            }
                        }
                    }

                    if let Err(err) = self.refresh_envs_file().await {
                        trace_err_chain!(err, "Failed to refresh envs file");
                    }

                    match self.refresh_discovery_file().await {
                        Err(err) => {
                            trace_err_chain!(err, "Failed to refresh discovery file");
                        }
                        Ok(Some(discovery)) => {
                            if let Err(err) = self.refresh_nym_network_file(discovery).await {
                                trace_err_chain!(err, "Failed to refresh nym network file");
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
    background_error_tx: tokio::sync::mpsc::Sender<()>,
    cancel_token: CancellationToken,
) -> JoinHandle<()> {
    let refresher = FileRefresher::new(config_path, network, background_error_tx, cancel_token);
    tokio::spawn(refresher.run())
}
