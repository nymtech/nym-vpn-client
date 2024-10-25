// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

fn refresh_discovery_file(network_name: &str) -> anyhow::Result<()> {
    if !super::bootstrap::is_time_to_refresh_discovery_file(network_name)? {
        return Ok(());
    }
    super::bootstrap::download_discovery_to_file(network_name)?;
    Ok(())
}

// Ideally we only refresh the discovery file when the tunnel is up
#[allow(unused)]
pub(crate) async fn start_background_discovery_refresh(
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
                    if let Err(err) = refresh_discovery_file(&network_name) {
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
