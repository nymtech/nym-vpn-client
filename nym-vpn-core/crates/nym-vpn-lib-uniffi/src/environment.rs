// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, sync::Arc};

use nym_vpn_lib_types::{
    FeatureFlags, Network, NetworkCompatibility, ParsedAccountLinks, SystemMessage, UserAgent,
};
use nym_vpn_network_config::NetworkCache;

use crate::error::VpnError;

#[derive(Clone, uniffi::Object)]
pub struct NymEnvironment {
    network: Box<nym_vpn_network_config::Network>,
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
    pub async fn new_with_cache_dir(
        cache_dir: PathBuf,
        network_name: &str,
        user_agent: UserAgent,
    ) -> Result<Self, VpnError> {
        let mut network_cache =
            NetworkCache::new(cache_dir, network_name, Some(user_agent.into()), None)
                .await
                .map_err(VpnError::internal)?;

        let network = if let Ok(network) = network_cache.network() {
            network
        } else {
            network_cache
                .fetch_if_stale()
                .await
                .map_err(|err| VpnError::FetchEnvironment {
                    details: err.to_string(),
                })?;
            network_cache.network().map_err(VpnError::internal)?
        };

        Ok(Self { network })
    }

    /// Sets up mainnet defaults without making any network calls. This means no system messages or
    /// account links will be available.
    #[uniffi::constructor]
    pub fn new_with_mainnet_fallback() -> Result<Self, VpnError> {
        nym_vpn_network_config::Network::mainnet_default()
            .map(|network| Self {
                network: Box::new(network),
            })
            .map_err(VpnError::internal)
    }

    pub fn network_name(&self) -> String {
        self.network.nym_network.network_name.clone()
    }

    /// Returns the currently set network environment
    pub fn current(&self) -> Network {
        Network::from(*self.network.clone())
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

    pub fn feature_flags(&self) -> Option<Arc<FeatureFlags>> {
        self.network
            .feature_flags
            .clone()
            .map(|feature_flags| Arc::new(FeatureFlags::from(feature_flags)))
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
