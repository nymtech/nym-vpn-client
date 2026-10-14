// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Helper functions for refreshing caches when network environment changes.

use nym_gateway_directory::{Config as GatewayConfig, GatewayClient, GatewayMinPerformance};
use nym_http_api_client::UserAgent;
use nym_vpn_network_config::Network;

use crate::gateway_directory::GatewayCacheHandle;

/// Point the gateway cache at the API endpoints of an updated network.
/// Discovery only refreshes the running environment (a network switch needs a restart), so the
/// cached gateway lists and topology stay valid and are not cleared. The cache clears itself when
/// the replacement client has other min-performance thresholds, so the current ones are kept.
pub async fn update_caches_for_network(
    network: &Network,
    gateway_cache_handle: &GatewayCacheHandle,
    min_gateway_performance: Option<GatewayMinPerformance>,
    user_agent: &UserAgent,
) {
    let network_name = &network.nym_network.network_name;
    tracing::info!(
        network = %network_name,
        "Updating gateway client for network environment change"
    );

    // Create new gateway client for the new environment
    let nyxd_url = network.nyxd_url();
    let nym_api_urls = network.nym_api_urls().unwrap_or_default();
    let nym_vpn_api_urls = network.nym_vpn_api_urls().unwrap_or_default();

    // Validate that we have the necessary URLs
    if nym_vpn_api_urls.is_empty() {
        tracing::error!(
            network = %network_name,
            "No VPN API URLs available for new environment, cannot update gateway cache"
        );
        return;
    }

    if nym_api_urls.is_empty() {
        tracing::warn!(
            network = %network_name,
            "No Nym API URLs available for new environment"
        );
    }

    let gateway_config = match GatewayConfig::new(
        nyxd_url,
        nym_api_urls.clone(),
        nym_vpn_api_urls.clone(),
        min_gateway_performance,
    ) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!(
                network = %network_name,
                error = %e,
                vpn_api_urls = ?nym_vpn_api_urls,
                nym_api_urls = ?nym_api_urls,
                "Failed to create gateway config for new environment"
            );
            return;
        }
    };

    let new_gateway_client = match GatewayClient::new(gateway_config, user_agent.clone()) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!(
                network = %network_name,
                error = %e,
                "Failed to create gateway client for new environment"
            );
            return;
        }
    };

    // Replace the gateway client in the cache
    if let Err(e) = gateway_cache_handle.replace_gateway_client(new_gateway_client) {
        tracing::warn!(
            network = %network_name,
            error = %e,
            "Failed to replace gateway client on environment change"
        );
    } else {
        tracing::info!(
            network = %network_name,
            "Gateway client successfully updated for new environment"
        );
    }
}
