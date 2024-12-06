// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_tunnel_provider::error::VpnError;

use super::NETWORK_ENVIRONMENT;
use crate::uniffi_custom_impls::{AccountLinks, NetworkEnvironment, SystemMessage};

pub(crate) async fn init_environment(network_name: &str) -> Result<(), VpnError> {
    let network = nym_vpn_network_config::Network::fetch(network_name).map_err(|err| {
        VpnError::InternalError {
            details: err.to_string(),
        }
    })?;

    // To bridge with old code, export to environment. New code should not rely on this.
    network.export_to_env();

    let mut guard = NETWORK_ENVIRONMENT.lock().await;
    *guard = Some(network);

    Ok(())
}

pub(crate) async fn init_fallback_mainnet_environment() -> Result<(), VpnError> {
    let network = nym_vpn_network_config::Network::mainnet_default();
    network.export_to_env();

    let mut guard = NETWORK_ENVIRONMENT.lock().await;
    *guard = Some(network);

    Ok(())
}

pub(crate) async fn current_environment() -> Result<NetworkEnvironment, VpnError> {
    current_environment_details()
        .await
        .map(NetworkEnvironment::from)
}

pub(super) async fn current_environment_details(
) -> Result<nym_vpn_network_config::Network, VpnError> {
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

pub(crate) async fn get_account_links(path: &str, locale: &str) -> Result<AccountLinks, VpnError> {
    let account_id = super::account::get_account_id(path).await.ok();
    current_environment_details()
        .await
        .and_then(|network| {
            network
                .nym_vpn_network
                .try_into_parsed_links(locale, account_id.as_deref())
                .map_err(|err| VpnError::InternalError {
                    details: err.to_string(),
                })
        })
        .map(AccountLinks::from)
}

pub(crate) async fn get_feature_flag_credential_mode() -> Result<bool, VpnError> {
    let current_environment = NETWORK_ENVIRONMENT.lock().await.clone();
    current_environment
        .as_ref()
        .map(get_credential_mode)
        .ok_or(VpnError::InvalidStateError {
            details: "Network environment not set".to_string(),
        })
}

fn get_credential_mode(network: &nym_vpn_network_config::Network) -> bool {
    network
        .get_feature_flag("zkNym", "credentialMode")
        .unwrap_or(false)
}

pub(crate) async fn fetch_environment(network_name: &str) -> Result<NetworkEnvironment, VpnError> {
    nym_vpn_network_config::Network::fetch(network_name)
        .map(NetworkEnvironment::from)
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}

pub(crate) async fn fetch_system_messages(
    network_name: &str,
) -> Result<Vec<SystemMessage>, VpnError> {
    nym_vpn_network_config::Network::fetch(network_name)
        .map(|network| {
            network
                .nym_vpn_network
                .system_messages
                .into_current_iter()
                .map(SystemMessage::from)
                .collect()
        })
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}

pub(crate) async fn fetch_account_links(
    path: &str,
    network_name: &str,
    locale: &str,
) -> Result<AccountLinks, VpnError> {
    let account_id = super::account::get_account_id(path).await.ok();
    nym_vpn_network_config::Network::fetch(network_name)
        .and_then(|network| {
            network
                .nym_vpn_network
                .try_into_parsed_links(locale, account_id.as_deref())
        })
        .map(AccountLinks::from)
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}
