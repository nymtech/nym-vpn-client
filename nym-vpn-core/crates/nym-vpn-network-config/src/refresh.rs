// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::discovery::Discovery;

fn refresh_discovery_file(config_dir: &Path, network_name: &str) -> anyhow::Result<()> {
    if !Discovery::path_is_stale(config_dir, network_name)? {
        return Ok(());
    }
    Discovery::fetch(network_name)?.write_to_file(config_dir)?;
    Ok(())
}

// Ideally we only refresh the discovery file when the tunnel is up
#[allow(unused)]
pub(crate) async fn start_background_discovery_refresh(
    config_path: PathBuf,
    network_name: String,
    cancel_token: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // Check once an hour
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
        interval.tick().await; // initial tick

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = refresh_discovery_file(&config_path, &network_name) {
                        tracing::error!("Failed to refresh discovery file: {:?}", err);
                    }
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }
    })
}
