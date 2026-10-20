// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use nym_sdk::UserAgent;
use nym_vpn_network_config::NetworkCache;

pub async fn setup_environment(
    config_dir: &Path,
    network_name: &str,
    user_agent: UserAgent,
) -> anyhow::Result<NetworkCache> {
    tracing::info!("Setting up environment for {network_name}");

    let vpn_api_override = option_env!("NYM_VPN_API_URL_OVERRIDE").map(|s| s.to_owned());
    let mut network_cache = NetworkCache::new(
        config_dir.to_path_buf(),
        network_name,
        Some(user_agent),
        vpn_api_override,
    )
    .await?;

    if let Ok(network) = network_cache.network() {
        network.export_to_env();
    } else {
        network_cache.fetch_if_stale().await?;
        network_cache.network()?.export_to_env();
    }

    Ok(network_cache)
}
