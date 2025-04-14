// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, time::Duration};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::NymNetwork;

use super::discovery::Discovery;

struct FileRefresher {
    config_path: PathBuf,
    network_name: String,
    cancel_token: CancellationToken,
}

impl FileRefresher {
    fn new(config_path: PathBuf, network_name: String, cancel_token: CancellationToken) -> Self {
        Self {
            config_path,
            network_name,
            cancel_token,
        }
    }

    async fn refresh_discovery_file(&self) -> anyhow::Result<Option<Discovery>> {
        if Discovery::path_is_stale(self.config_path.as_path(), &self.network_name)? {
            let discovery = Discovery::fetch(&self.network_name).await?;
            discovery.write_to_file(self.config_path.as_path())?;
            Ok(Some(discovery))
        } else {
            Ok(None)
        }
    }

    async fn refresh_nym_network_file(&self, discovery: Discovery) -> anyhow::Result<()> {
        if NymNetwork::path_is_stale(self.config_path.as_path(), &self.network_name)? {
            discovery.update_nym_network_file(&self.config_path).await?;
        }

        Ok(())
    }

    async fn run(self) {
        // Check once an hour
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));

        self.cancel_token
            .run_until_cancelled(async {
                interval.tick().await; // initial tick
                loop {
                    interval.tick().await;
                    match self.refresh_discovery_file().await {
                        Err(err) => {
                            tracing::error!("Failed to refresh discovery file: {:?}", err)
                        }
                        Ok(Some(discovery)) => {
                            if let Err(err) = self.refresh_nym_network_file(discovery).await {
                                tracing::error!("Failed to refresh nym network file: {:?}", err);
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
#[allow(unused)]
pub fn start_background_file_refresh(
    config_path: PathBuf,
    network_name: String,
    cancel_token: CancellationToken,
) -> JoinHandle<()> {
    let refresher = FileRefresher::new(config_path, network_name, cancel_token);
    tokio::spawn(refresher.run())
}
