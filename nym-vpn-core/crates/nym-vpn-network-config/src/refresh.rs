// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::discovery::Discovery;

struct DiscoveryRefresher {
    config_path: PathBuf,
    network_name: String,
    cancel_token: CancellationToken,
}

impl DiscoveryRefresher {
    fn new(config_path: PathBuf, network_name: String, cancel_token: CancellationToken) -> Self {
        Self {
            config_path,
            network_name,
            cancel_token,
        }
    }

    async fn refresh_discovery_file(&self) -> anyhow::Result<()> {
        if !Discovery::path_is_stale(self.config_path.as_path(), &self.network_name)? {
            return Ok(());
        }
        Discovery::fetch(&self.network_name)
            .await?
            .write_to_file(self.config_path.as_path())?;
        Ok(())
    }

    async fn run(self) {
        // Check once an hour
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        interval.tick().await; // initial tick

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(Err(err)) = self.cancel_token.run_until_cancelled(self.refresh_discovery_file()).await {
                        tracing::error!("Failed to refresh discovery file: {:?}", err);
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    break;
                }
            }
        }
    }
}

// Ideally we only refresh the discovery file when the tunnel is up
#[allow(unused)]
pub fn start_background_discovery_refresh(
    config_path: PathBuf,
    network_name: String,
    cancel_token: CancellationToken,
) -> JoinHandle<()> {
    let refresher = DiscoveryRefresher::new(config_path, network_name, cancel_token);
    tokio::spawn(refresher.run())
}
