// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Helper functions for refreshing caches when network environment changes.

use std::sync::Arc;

use nym_endpoint_health::EndpointHealthTracker;
use nym_gateway_directory::{Config as GatewayConfig, GatewayClient};
use nym_http_api_client::UserAgent;
use nym_vpn_network_config::{Network, merge_and_order_api_urls_by_health};

use crate::{VpnTopologyServiceHandle, gateway_directory::GatewayCacheHandle};

/// Update gateway cache and topology cache for a new network environment.
/// This is called when the discovery refresher detects an environment change.
///
/// `endpoint_health` orders the nym-api URL list by health before the
/// gateway-directory client is built from it; pass `None` only for call
/// paths that genuinely have no tracker available.
pub async fn update_caches_for_network(
    network: &Network,
    gateway_cache_handle: &GatewayCacheHandle,
    topology_service_handle: &VpnTopologyServiceHandle,
    user_agent: &UserAgent,
    endpoint_health: Option<&Arc<EndpointHealthTracker>>,
) {
    let network_name = &network.nym_network.network_name;
    tracing::info!(
        network = %network_name,
        "Updating gateway cache and topology cache for network environment change"
    );

    // Clear the gateway cache
    if let Err(e) = gateway_cache_handle.clear_cache() {
        tracing::warn!(
            network = %network_name,
            error = %e,
            "Failed to clear gateway cache on environment change"
        );
    }

    // Clear the topology cache
    topology_service_handle.clear_cache().await;

    // Create new gateway client for the new environment
    let nyxd_url = network.nyxd_url();
    let mut nym_api_urls = network.nym_api_urls().unwrap_or_default();
    if let Some(tracker) = endpoint_health {
        nym_api_urls = merge_and_order_api_urls_by_health(nym_api_urls, tracker);
    }
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
        None,
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
            "Gateway cache and topology cache successfully updated for new environment"
        );
    }
}
