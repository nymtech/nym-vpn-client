// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_lib_types::{Network, NetworkCompatibility, ParsedAccountLinks, SystemMessage};

use crate::error::VpnError;

#[derive(Clone, uniffi::Object)]
pub struct NymEnvironment {
    network: nym_vpn_network_config::Network,
}

impl NymEnvironment {
    pub fn inner(&self) -> &nym_vpn_network_config::Network {
        &self.network
    }

    pub fn export_to_env(&self) {
        // To bridge with old code, export to environment. New code should not rely on this.
        self.network.export_to_env();
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NymEnvironment {
    /// Fetches the network environment details from the network name
    #[uniffi::constructor]
    pub async fn new(cache_dir: PathBuf, network_name: &str) -> Result<Self, VpnError> {
        nym_vpn_network_config::discover_env(
            PathBuf::from(cache_dir)
                .parent()
                .ok_or(VpnError::internal("cache directory can't be root"))?,
            network_name,
        )
        .await
        .map(|network| Self { network })
        .map_err(VpnError::internal)
    }

    /// Sets up mainnet defaults without making any network calls. This means no system messages or
    /// account links will be available.
    #[uniffi::constructor]
    pub fn new_with_mainnet_fallback() -> Result<Self, VpnError> {
        nym_vpn_network_config::Network::mainnet_default()
            .map(|network| Self { network })
            .ok_or(VpnError::InternalError {
                details: "mainnet is not consistent".to_string(),
            })
    }

    pub async fn __stub_to_keep_compiler_happy(&self) {
        // todo: remove after updating to uniffi 0.31
    }

    /// Returns the currently set network environment
    pub fn current(&self) -> Network {
        Network::from(self.network.clone())
    }

    pub fn system_messages(&self) -> Vec<SystemMessage> {
        self.network
            .nym_vpn_network
            .system_messages
            .current_iter()
            .cloned()
            .map(SystemMessage::from)
            .collect()
    }

    pub fn network_compatibility(&self) -> Option<NetworkCompatibility> {
        self.network
            .system_configuration
            .as_ref()
            .and_then(|sc| sc.min_supported_app_versions.clone())
            .map(NetworkCompatibility::from)
    }

    pub fn account_links(
        &self,
        locale: &str,
        account_id: Option<String>,
    ) -> Result<ParsedAccountLinks, VpnError> {
        self.network
            .nym_vpn_network
            .clone()
            .try_into_parsed_links(locale, account_id.as_deref())
            .map_err(VpnError::internal)
            .map(ParsedAccountLinks::from)
    }
}
