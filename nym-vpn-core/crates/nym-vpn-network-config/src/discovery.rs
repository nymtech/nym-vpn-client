// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use anyhow::Context;
use nym_sdk::UserAgent;
use url::Url;

use nym_vpn_api_client::{
    response::{NymWellknownDiscoveryItem, NymWellknownDiscoveryItemResponse},
    BootstrapVpnApiClient, VpnApiClient,
};

use nym_api_requests::NymNetworkDetailsResponse;
use nym_validator_client::nym_api::{Client as NymApiClient, NymApiClientExt};

use crate::{
    system_configuration::SystemConfiguration, AccountManagement, FeatureFlags, SystemMessages,
};

use super::{nym_network::NymNetwork, MAX_FILE_AGE, NETWORKS_SUBDIR};

// TODO: integrate with nym-vpn-api-client

const DISCOVERY_FILE: &str = "discovery.json";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Discovery {
    // Base network setup
    pub(super) network_name: String,
    pub(super) nym_api_url: Url,
    pub(super) nym_vpn_api_url: Url,

    // Additional context
    pub(super) account_management: Option<AccountManagement>,
    pub(super) feature_flags: Option<FeatureFlags>,
    pub(super) system_configuration: Option<SystemConfiguration>,
    pub(super) system_messages: SystemMessages,
}

// Include the generated Default implementation
include!(concat!(env!("OUT_DIR"), "/default_discovery.rs"));

impl Discovery {
    fn path(config_dir: &Path, network_name: &str) -> PathBuf {
        config_dir
            .join(NETWORKS_SUBDIR)
            .join(format!("{}_{}", network_name, DISCOVERY_FILE))
    }

    pub(super) fn path_is_stale(config_dir: &Path, network_name: &str) -> anyhow::Result<bool> {
        if let Some(age) = crate::util::get_age_of_file(&Self::path(config_dir, network_name))? {
            Ok(age > MAX_FILE_AGE)
        } else {
            Ok(true)
        }
    }

    pub async fn fetch(network_name: &str) -> anyhow::Result<Self> {
        // allow panic because a broken bootstrap url means everything will fail anyways.
        #[allow(clippy::expect_used)]
        let default_url = Self::DEFAULT_VPN_API_URL
            .parse()
            .expect("Failed to parse NYM VPN API URL");
        let client = BootstrapVpnApiClient::new(default_url)?;

        tracing::debug!("Fetching nym network discovery");
        let discovery = client.get_wellknown_discovery(network_name).await?;

        tracing::debug!("Discovery response: {:#?}", discovery);

        if discovery.network_name != network_name {
            anyhow::bail!("Network name mismatch between requested and fetched discovery")
        }

        tracing::debug!("Fetched nym network discovery: {:#?}", discovery);
        discovery.try_into()
    }

    pub(super) fn read_from_file(config_dir: &Path, network_name: &str) -> anyhow::Result<Self> {
        let path = Self::path(config_dir, network_name);
        tracing::debug!("Reading discovery file from: {}", path.display());

        let file_str = std::fs::read_to_string(path)?;
        let network: Discovery = serde_json::from_str(&file_str)?;
        Ok(network)
    }

    pub(super) fn write_to_file(&self, config_dir: &Path) -> anyhow::Result<()> {
        let path = Self::path(config_dir, &self.network_name);
        tracing::debug!("Writing discovery file to: {}", path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for {:?}", path))?;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .with_context(|| format!("Failed to open discovery file at {:?}", path))?;

        serde_json::to_writer_pretty(&file, self)
            .with_context(|| format!("Failed to write discovery file at {:?}", path))?;

        Ok(())
    }

    async fn update_file(config_dir: &Path, network_name: &str) -> anyhow::Result<()> {
        Self::fetch(network_name).await?.write_to_file(config_dir)
    }

    pub(super) async fn ensure_exists(
        config_dir: &Path,
        network_name: &str,
    ) -> anyhow::Result<Self> {
        if !Self::path(config_dir, network_name).exists() && network_name == "mainnet" {
            tracing::info!("No discovery file found, writing creating a new discovery file");
            Self::fetch(network_name)
                .await
                .inspect_err(|err| {
                    tracing::warn!(
                        "Failed to fetch remote discovery file: {err}, creating a default one"
                    )
                })
                .unwrap_or_default()
                .write_to_file(config_dir)
                .inspect_err(|err| tracing::warn!("Failed to write discovery file: {err}"))
                .ok();
        } else {
            // Download the file if it doesn't exists, or refresh it.
            // TODO: in the future, we should only refresh the discovery file when the tunnel is up.
            // Probably in a background task.

            Self::update_file(config_dir, network_name)
                .await
                .inspect_err(|err| {
                    tracing::warn!("Failed to refresh discovery file: {err}");
                    tracing::warn!("Attempting to use existing discovery file");
                })
                .ok();
        }

        Self::read_from_file(config_dir, network_name)
    }

    pub async fn fetch_nym_network_details(&self) -> anyhow::Result<NymNetwork> {
        tracing::debug!("Fetching nym network details");
        // Spawn the root task
        let network_details =
            NymApiClient::builder::<Url, anyhow::Error>(self.nym_api_url.clone())?
                .build::<anyhow::Error>()?
                .get_network_details()
                .await?;

        if network_details.network.network_name != self.network_name {
            anyhow::bail!("Network name mismatch between requested and fetched network details")
        }
        // resolve_nym_network_details(&mut network_details.network);
        Ok(NymNetwork {
            network: network_details.network,
        })
    }
}

impl TryFrom<NymWellknownDiscoveryItemResponse> for Discovery {
    type Error = anyhow::Error;

    fn try_from(discovery: NymWellknownDiscoveryItemResponse) -> anyhow::Result<Self> {
        let account_management = discovery.account_management.and_then(|am| {
            AccountManagement::try_from(am)
                .inspect_err(|err| tracing::warn!("Failed to parse account management: {err}"))
                .ok()
        });

        let feature_flags = discovery.feature_flags.and_then(|ff| {
            FeatureFlags::try_from(ff)
                .inspect_err(|err| tracing::warn!("Failed to parse feature flags: {err}"))
                .ok()
        });

        let system_configuration = discovery
            .system_configuration
            .map(SystemConfiguration::from);

        let system_messages = discovery
            .system_messages
            .map(SystemMessages::from)
            .unwrap_or_default();

        Ok(Self {
            network_name: discovery.network_name,
            nym_api_url: discovery.nym_api_url.parse()?,
            nym_vpn_api_url: discovery.nym_vpn_api_url.parse()?,
            account_management,
            feature_flags,
            system_configuration,
            system_messages,
        })
    }
}

fn empty_user_agent() -> UserAgent {
    UserAgent {
        application: String::new(),
        version: String::new(),
        platform: String::new(),
        git_commit: String::new(),
    }
}

pub(crate) async fn fetch_nym_network_details(
    nym_api_url: &Url,
) -> anyhow::Result<NymNetworkDetailsResponse> {
    tracing::debug!("Fetching nym network details");
    NymApiClient::builder::<Url, anyhow::Error>(nym_api_url.clone())?
        .build::<anyhow::Error>()?
        .get_network_details()
        .await
        .with_context(|| "Discovery endpoint returned error response".to_owned())
}

pub(crate) async fn fetch_nym_vpn_network_details(
    nym_vpn_api_url: &Url,
) -> anyhow::Result<NymWellknownDiscoveryItem> {
    tracing::debug!("Fetching nym vpn network details");
    VpnApiClient::new(nym_vpn_api_url.clone(), empty_user_agent())?
        .get_wellknown_current_env()
        .await
        .with_context(|| "Discovery endpoint returned error response".to_owned())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    use crate::{
        account_management::AccountManagementPaths, feature_flags::FlagValue,
        system_messages::Properties, SystemMessage,
    };

    use super::*;

    #[tokio::test]
    async fn test_discovery_fetch() {
        let network_name = "mainnet";
        let discovery = Discovery::fetch(network_name).await.unwrap();
        assert_eq!(discovery.network_name, network_name);
    }

    #[tokio::test]
    async fn test_discovery_default_same_as_fetched() {
        let default = Discovery::default();
        let fetched = Discovery::fetch(&default.network_name).await.unwrap();

        // Only compare the base fields
        assert_eq!(default.network_name, fetched.network_name);
        assert_eq!(default.nym_api_url, fetched.nym_api_url);
        assert_eq!(default.nym_vpn_api_url, fetched.nym_vpn_api_url);
    }

    #[test]
    fn test_parse_discovery_response() {
        let json = r#"{
            "network_name": "qa",
            "nym_api_url": "https://foo.ch/api/",
            "nym_vpn_api_url": "https://bar.ch/api/",
            "account_management": {
                "url": "https://foobar.ch/",
                "paths": {
                    "sign_up": "{locale}/account/create",
                    "sign_in": "{locale}/account/login",
                    "account": "{locale}/account/{account_id}"
                }
            },
            "feature_flags": {
                "website": {
                    "showAccounts": "true"
                },
                "zkNyms": {
                    "credentialMode": "false"
                }
            },
            "system_messages": [
                {
                    "name": "test_message",
                    "displayFrom": "2024-11-05T12:00:00.000Z",
                    "displayUntil": "",
                    "message": "This is a test message, no need to panic!",
                    "properties": {
                        "modal": "true"
                    }
                }
            ],
            "network_compatibility": {
                "core": "1.1.1",
                "ios": "1.1.1",
                "macos": "1.1.1",
                "tauri": "1.1.1",
                "android": "1.1.1"
            }
        }"#;
        let discovery: NymWellknownDiscoveryItemResponse = serde_json::from_str(json).unwrap();
        let network: Discovery = discovery.try_into().unwrap();

        let expected_network = Discovery {
            network_name: "qa".to_owned(),
            nym_api_url: "https://foo.ch/api/".parse().unwrap(),
            nym_vpn_api_url: "https://bar.ch/api/".parse().unwrap(),
            account_management: Some(AccountManagement {
                url: "https://foobar.ch/".parse().unwrap(),
                paths: AccountManagementPaths {
                    sign_up: "{locale}/account/create".to_owned(),
                    sign_in: "{locale}/account/login".to_owned(),
                    account: "{locale}/account/{account_id}".to_owned(),
                },
            }),
            feature_flags: Some(FeatureFlags {
                flags: HashMap::from([
                    (
                        "website".to_owned(),
                        FlagValue::Group(HashMap::from([(
                            "showAccounts".to_owned(),
                            "true".to_owned(),
                        )])),
                    ),
                    (
                        "zkNyms".to_owned(),
                        FlagValue::Group(HashMap::from([(
                            "credentialMode".to_owned(),
                            "false".to_owned(),
                        )])),
                    ),
                ]),
            }),
            system_messages: SystemMessages::from(vec![SystemMessage {
                name: "test_message".to_owned(),
                display_from: Some(
                    OffsetDateTime::parse("2024-11-05T12:00:00.000Z", &Rfc3339).unwrap(),
                ),
                display_until: None,
                message: "This is a test message, no need to panic!".to_owned(),
                properties: Properties::from(HashMap::from([(
                    "modal".to_owned(),
                    "true".to_owned(),
                )])),
            }]),
            system_configuration: None,
        };
        assert_eq!(network, expected_network);
    }
}
