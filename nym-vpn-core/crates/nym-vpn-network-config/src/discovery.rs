// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use anyhow::Context;
use url::Url;

use crate::{
    resolve_nym_network_details,
    response::{DiscoveryResponse, NymNetworkDetailsResponse, NymWellknownDiscoveryItem},
    AccountManagement, FeatureFlags, SystemMessages,
};

use super::{nym_network::NymNetwork, MAX_FILE_AGE, NETWORKS_SUBDIR};

// TODO: integrate with nym-vpn-api-client

const DISCOVERY_FILE: &str = "discovery.json";
const DISCOVERY_WELLKNOWN: &str = "https://nymvpn.com/api/public/v1/.wellknown";

const NYM_NETWORK_DETAILS_PATH: &str = "/v1/network/details";
const NYM_VP_NETWORK_CURRENT_ENV_PATH: &str = "/public/v1/.wellknown/current-env.json";

fn nym_network_details_endpoint(nym_api_url: &Url) -> String {
    format!("{}/{}", nym_api_url, NYM_NETWORK_DETAILS_PATH)
}

fn nym_vpn_network_details_endpoint(nym_vpn_api_url: &Url) -> String {
    format!("{}/{}", nym_vpn_api_url, NYM_VP_NETWORK_CURRENT_ENV_PATH)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Discovery {
    // Base network setup
    pub(super) network_name: String,
    pub(super) nym_api_url: Url,
    pub(super) nym_vpn_api_url: Url,

    // Additional context
    pub(super) account_management: Option<AccountManagement>,
    pub(super) feature_flags: Option<FeatureFlags>,
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

    fn endpoint(network_name: &str) -> anyhow::Result<Url> {
        format!(
            "{}/{}/{}",
            DISCOVERY_WELLKNOWN, network_name, DISCOVERY_FILE
        )
        .parse()
        .map_err(Into::into)
    }

    pub fn fetch(network_name: &str) -> anyhow::Result<Self> {
        let discovery: DiscoveryResponse = {
            let url = Self::endpoint(network_name)?;

            tracing::debug!("Fetching nym network discovery from: {}", url);
            let response = reqwest::blocking::get(url.clone())
                .inspect_err(|err| tracing::warn!("{}", err))
                .with_context(|| format!("Failed to fetch discovery from {}", url))?
                .error_for_status()
                .inspect_err(|err| tracing::warn!("{}", err))
                .with_context(|| "Discovery endpoint returned error response".to_owned())?;

            let text_response = response
                .text()
                .inspect_err(|err| tracing::warn!("{}", err))
                .with_context(|| "Failed to read response text")?;
            tracing::debug!("Discovery response: {:#?}", text_response);

            serde_json::from_str(&text_response)
                .with_context(|| "Failed to parse discovery response")
        }?;
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

    fn try_update_file(config_dir: &Path, network_name: &str) -> anyhow::Result<()> {
        if Self::path_is_stale(config_dir, network_name)? {
            Self::fetch(network_name)?.write_to_file(config_dir)?;
        }
        Ok(())
    }

    pub(super) fn ensure_exists(config_dir: &Path, network_name: &str) -> anyhow::Result<Self> {
        if !Self::path(config_dir, network_name).exists() && network_name == "mainnet" {
            tracing::info!("No discovery file found, writing default discovery file");
            Self::default()
                .write_to_file(config_dir)
                .inspect_err(|err| tracing::warn!("Failed to write default discovery file: {err}"))
                .ok();
        }

        // Download the file if it doesn't exists, or if the file is too old, refresh it.
        // TODO: in the future, we should only refresh the discovery file when the tunnel is up.
        // Probably in a background task.

        Self::try_update_file(config_dir, network_name)
            .inspect_err(|err| {
                tracing::warn!("Failed to refresh discovery file: {err}");
                tracing::warn!("Attempting to use existing discovery file");
            })
            .ok();

        Self::read_from_file(config_dir, network_name)
    }

    pub fn fetch_nym_network_details(&self) -> anyhow::Result<NymNetwork> {
        // TODO: integrate with validator-client and/or nym-vpn-api-client
        let url = nym_network_details_endpoint(&self.nym_api_url);
        tracing::debug!("Fetching nym network details from: {}", url);
        let mut network_details: NymNetworkDetailsResponse = reqwest::blocking::get(url.clone())
            .with_context(|| format!("Failed to fetch network details from {}", url))?
            .json()
            .with_context(|| "Failed to parse network details")?;
        if network_details.network.network_name != self.network_name {
            anyhow::bail!("Network name mismatch between requested and fetched network details")
        }
        resolve_nym_network_details(&mut network_details.network);
        Ok(NymNetwork {
            network: network_details.network,
        })
    }
}

impl TryFrom<DiscoveryResponse> for Discovery {
    type Error = anyhow::Error;

    fn try_from(discovery: DiscoveryResponse) -> anyhow::Result<Self> {
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
            system_messages,
        })
    }
}

pub(crate) async fn fetch_nym_network_details(
    nym_api_url: &Url,
) -> anyhow::Result<NymNetworkDetailsResponse> {
    // TODO: integrate with validator-client and/or nym-vpn-api-client
    let url = nym_network_details_endpoint(nym_api_url);
    tracing::debug!("Fetching nym network details from: {}", url);
    reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch network details from {}", url))?
        .json()
        .await
        .with_context(|| "Failed to parse network details")
}

pub(crate) async fn fetch_nym_vpn_network_details(
    nym_vpn_api_url: &Url,
) -> anyhow::Result<NymWellknownDiscoveryItem> {
    // TODO: integrate with nym-vpn-api-client
    let url = nym_vpn_network_details_endpoint(nym_vpn_api_url);
    tracing::debug!("Fetching nym vpn network details from: {}", url);
    reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch vpn network details from {url}"))?
        .json()
        .await
        .with_context(|| "Failed to parse vpn network details")
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

    #[test]
    fn test_discovery_endpoint() {
        let network_name = "mainnet";
        let url = Discovery::endpoint(network_name).unwrap();
        assert_eq!(
            url,
            "https://nymvpn.com/api/public/v1/.wellknown/mainnet/discovery.json"
                .parse()
                .unwrap()
        );
    }

    #[test]
    fn test_discovery_fetch() {
        let network_name = "mainnet";
        let discovery = Discovery::fetch(network_name).unwrap();
        assert_eq!(discovery.network_name, network_name);
    }

    #[test]
    fn test_discovery_default_same_as_fetched() {
        let default = Discovery::default();
        let fetched = Discovery::fetch(&default.network_name).unwrap();

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
            ]
        }"#;
        let discovery: DiscoveryResponse = serde_json::from_str(json).unwrap();
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
        };
        assert_eq!(network, expected_network);
    }
}
