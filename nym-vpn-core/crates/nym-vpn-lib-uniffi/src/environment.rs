// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_common::trace_err_chain;
use nym_vpn_lib_types::{
    Network, NetworkCompatibility, ParsedAccountLinks, SystemMessage, UserAgent,
};
use nym_vpn_network_config::NetworkCache;

use super::{NETWORK_ENVIRONMENT, error::VpnError};

pub(crate) async fn init_environment(
    cache_dir: String,
    network_name: &str,
    user_agent: UserAgent,
) -> Result<(), VpnError> {
    let mut network_cache = NetworkCache::new(
        PathBuf::from(cache_dir),
        network_name,
        Some(user_agent.into()),
        None,
    )
    .await
    .map_err(VpnError::internal)?;

    if let Err(err) = network_cache.fetch_if_stale().await {
        trace_err_chain!(err, "failed to fetch network environment");
    }

    let network = network_cache
        .network()
        .map_err(|err| VpnError::internal(err.to_string()))?;

    // To bridge with old code, export to environment. New code should not rely on this.
    network.export_to_env();

    let mut guard = NETWORK_ENVIRONMENT.lock().await;
    *guard = Some(*network);

    Ok(())
}

pub(crate) async fn init_fallback_mainnet_environment() -> Result<(), VpnError> {
    let network = nym_vpn_network_config::Network::mainnet_default().map_err(|_err| {
        VpnError::InternalError {
            details: "mainnet is not consistent".to_string(),
        }
    })?;
    network.export_to_env();

    let mut guard = NETWORK_ENVIRONMENT.lock().await;
    *guard = Some(network);

    Ok(())
}

pub(crate) async fn current_environment() -> Result<Network, VpnError> {
    current_environment_details().await.map(Network::from)
}

pub(super) async fn current_environment_details()
-> Result<nym_vpn_network_config::Network, VpnError> {
    NETWORK_ENVIRONMENT
        .lock()
        .await
        .clone()
        .ok_or(VpnError::InvalidStateError {
            details: "Network environment not set".to_string(),
        })
}

pub(crate) async fn get_system_messages() -> Result<Vec<SystemMessage>, VpnError> {
    current_environment_details().await.map(|network| {
        network
            .nym_vpn_network
            .system_messages
            .into_current_iter()
            .map(SystemMessage::from)
            .collect()
    })
}

pub(crate) async fn get_network_compatibility() -> Result<Option<NetworkCompatibility>, VpnError> {
    current_environment_details().await.map(|network| {
        network
            .system_configuration
            .and_then(|sc| sc.min_supported_app_versions)
            .map(NetworkCompatibility::from)
    })
}

pub(crate) async fn get_account_links(locale: &str) -> Result<ParsedAccountLinks, VpnError> {
    let account_id = super::account::get_account_id().await?;
    current_environment_details()
        .await
        .and_then(|network| {
            network
                .nym_vpn_network
                .try_into_parsed_links(locale, account_id.as_deref())
                .map_err(VpnError::internal)
        })
        .map(ParsedAccountLinks::from)
}

pub(crate) async fn get_account_links_raw(
    path: &str,
    locale: &str,
) -> Result<ParsedAccountLinks, VpnError> {
    // If the account ID is not found, we are not logged in, so we don't need to pass it to the
    // API. But we can still get the links that don't require an account ID.
    let account_id = super::account::raw::get_account_id_raw(path).await.ok();

    current_environment_details()
        .await
        .and_then(|network| {
            network
                .nym_vpn_network
                .try_into_parsed_links(locale, account_id.as_deref())
                .map_err(VpnError::internal)
        })
        .map(ParsedAccountLinks::from)
}
