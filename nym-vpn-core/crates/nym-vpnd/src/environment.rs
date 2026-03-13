// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sdk::UserAgent;
use nym_vpn_network_config::NetworkCache;

pub async fn setup_environment(
    network_name: &str,
    user_agent: UserAgent,
) -> anyhow::Result<NetworkCache> {
    let cache_dir = crate::paths::config_dir();

    tracing::info!("Setting up environment for {network_name}");

    let mut network_cache = NetworkCache::new(cache_dir, network_name, Some(user_agent)).await?;

    if let Ok(network) = network_cache.network() {
        network.export_to_env();
    } else {
        network_cache.fetch_if_stale().await?;
        network_cache.network()?.export_to_env();
    }

    Ok(network_cache)
}
