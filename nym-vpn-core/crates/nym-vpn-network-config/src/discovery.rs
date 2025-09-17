// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use nym_common::trace_err_chain;
use nym_sdk::UserAgent;
use url::Url;

use nym_vpn_api_client::{
    VpnApiClient,
    response::{NymWellknownDiscoveryItem, NymWellknownDiscoveryItemResponse},
};

use nym_api_requests::NymNetworkDetailsResponse;
use nym_validator_client::nym_api::NymApiClientExt;

use crate::{
    AccountManagement, ApiUrl, Error, FeatureFlags, Result, SystemMessages,
    system_configuration::SystemConfiguration,
};

use super::{MAX_FILE_AGE, NETWORKS_SUBDIR, nym_network::NymNetwork};

// TODO: integrate with nym-vpn-api-client

const DISCOVERY_FILE: &str = "discovery.json";

static MAINNET_DISCOVERY_JSON: &[u8] = include_bytes!("../default/mainnet_discovery.json");
static SANDBOX_DISCOVERY_JSON: &[u8] = include_bytes!("../default/sandbox_discovery.json");
static CANARY_DISCOVERY_JSON: &[u8] = include_bytes!("../default/canary_discovery.json");

static DEFAULT_VPN_API_URL: LazyLock<Url> =
    LazyLock::new(|| Discovery::default_mainnet().nym_vpn_api_url);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Discovery {
    // Base network setup
    pub(super) network_name: String,
    pub(super) nym_api_url: Url,
    pub(super) nym_api_urls: Vec<ApiUrl>,
    pub(super) nym_vpn_api_url: Url,
    pub(super) nym_vpn_api_urls: Vec<ApiUrl>,

    // Additional context
    pub(super) account_management: Option<AccountManagement>,
    pub(super) feature_flags: Option<FeatureFlags>,
    pub(super) system_configuration: Option<SystemConfiguration>,

    #[serde(default)]
    pub(super) system_messages: SystemMessages,
}

impl Discovery {
    /// Default VPN API URL
    pub fn defaul_vpn_api_url() -> Url {
        DEFAULT_VPN_API_URL.clone()
    }

    /// Default mainnet discovery
    pub fn default_mainnet() -> Self {
        #[allow(clippy::expect_used)]
        serde_json::from_slice(MAINNET_DISCOVERY_JSON)
            .expect("failed to parse default mainnet discovery")
    }

    /// Default sandbox discovery
    pub fn default_sandbox() -> Self {
        #[allow(clippy::expect_used)]
        serde_json::from_slice(SANDBOX_DISCOVERY_JSON)
            .expect("failed to parse default sandbox discovery")
    }

    /// Default canary discovery
    pub fn default_canary() -> Self {
        #[allow(clippy::expect_used)]
        serde_json::from_slice(CANARY_DISCOVERY_JSON)
            .expect("failed to parse default canary discovery")
    }

    fn path(config_dir: &Path, network_name: &str) -> PathBuf {
        config_dir
            .join(NETWORKS_SUBDIR)
            .join(format!("{network_name}_{DISCOVERY_FILE}"))
    }

    pub(super) fn path_is_stale(config_dir: &Path, network_name: &str) -> Result<bool> {
        let path = Self::path(config_dir, network_name);

        crate::filetime::is_stale_file(&path, MAX_FILE_AGE)
            .map_err(|source| Error::GetFileStaleness { path, source })
    }

    pub async fn fetch(network_name: &str) -> Result<Self> {
        // allow panic because a broken bootstrap url means everything will fail anyways.
        let default_url = DEFAULT_VPN_API_URL.clone();
        let client = VpnApiClient::new(default_url, empty_user_agent())
            .map_err(Error::CreateVpnApiClient)?;

        tracing::debug!("Fetching nym network discovery");
        let discovery = client
            .get_wellknown_discovery(network_name)
            .await
            .map_err(Error::GetWellKnownDiscovery)?;

        tracing::trace!("Discovery response: {:#?}", discovery);
        if discovery.network_name == network_name {
            tracing::trace!("Fetched nym network discovery: {:#?}", discovery);
            Self::try_from(discovery).map_err(Error::ConvertWellKnownDiscovery)
        } else {
            Err(Error::NetworkNameMismatch {
                expected: network_name.to_owned(),
                actual: discovery.network_name.clone(),
            })
        }
    }

    pub(super) fn read_from_file(config_dir: &Path, network_name: &str) -> Result<Self> {
        let path = Self::path(config_dir, network_name);
        tracing::debug!("Reading discovery file from: {}", path.display());

        crate::serialization::deserialize_from_json_file(path)
    }

    pub(super) fn write_to_file(&self, config_dir: &Path) -> Result<()> {
        let path = Self::path(config_dir, &self.network_name);
        tracing::debug!("Writing discovery file to: {}", path.display());

        crate::serialization::serialize_to_json_file(path, self)
    }

    pub(super) async fn ensure_exists(config_dir: &Path, network_name: &str) -> Result<Self> {
        match Self::read_from_file(config_dir, network_name) {
            Ok(discovery) => Ok(discovery),
            Err(e) if e.should_overwrite_file() => {
                if e.is_file_not_found() {
                    tracing::debug!("No discovery file found, creating a new discovery file");
                } else {
                    trace_err_chain!(e, "Failed to read discovery file");
                }

                let discovery = Self::fetch(network_name).await.or_else(|e| {
                    match Self::default_discovery(network_name) {
                        Some(default_discovery) => {
                            tracing::warn!(
                                "Failed to fetch remote discovery file: {e}, creating a default one"
                            );
                            Ok(default_discovery)
                        }
                        None => {
                            tracing::error!(
                                "No default discovery available for {network_name} environment"
                            );
                            Err(e)
                        }
                    }
                })?;

                discovery.write_to_file(config_dir).inspect_err(|err| {
                    trace_err_chain!(err, "Failed to write discovery file");
                })?;

                Ok(discovery)
            }
            Err(e) => {
                trace_err_chain!(e, "Failed to read discovery file");
                Err(e)
            }
        }
    }

    pub async fn fetch_nym_network_details(&self) -> Result<NymNetwork> {
        tracing::debug!("Fetching nym network details");
        let client = nym_http_api_client::Client::builder(self.nym_api_url.clone())
            .map_err(Box::new)?
            .build()
            .map_err(Box::new)?;
        let network_details = client
            .get_network_details()
            .await
            .map_err(Box::new)
            .map_err(Error::GetNetworkDetails)?;

        if network_details.network.network_name == self.network_name {
            Ok(NymNetwork {
                network: network_details.network,
            })
        } else {
            Err(Error::NetworkNameMismatch {
                expected: self.network_name.clone(),
                actual: network_details.network.network_name,
            })
        }
    }

    pub async fn update_nym_network_file(&self, config_dir: &Path) -> Result<()> {
        self.fetch_nym_network_details()
            .await?
            .write_to_file(config_dir)
    }

    fn default_discovery(network_name: &str) -> Option<Self> {
        Some(match network_name {
            "mainnet" => Self::default_mainnet(),
            "sandbox" => Self::default_sandbox(),
            "canary" => Self::default_canary(),
            _ => None?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryFromNymWellknownDiscoveryError {
    #[error("Failed to parse nym api url: {value}")]
    ParseNymApiUrl {
        value: String,
        source: url::ParseError,
    },

    #[error("Failed to parse nym vpn api url: {value}")]
    ParseNymVpnApiUrl {
        value: String,
        source: url::ParseError,
    },
}

impl TryFrom<NymWellknownDiscoveryItemResponse> for Discovery {
    type Error = DiscoveryFromNymWellknownDiscoveryError;

    fn try_from(discovery: NymWellknownDiscoveryItemResponse) -> Result<Self, Self::Error> {
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

        let nym_api_url = discovery.nym_api_url.parse().map_err(|source| {
            DiscoveryFromNymWellknownDiscoveryError::ParseNymApiUrl {
                value: discovery.nym_api_url,
                source,
            }
        })?;

        let nym_api_urls = discovery
            .nym_api_urls
            .into_iter()
            .map(|api_url| ApiUrl {
                url: api_url.url,
                fronts: api_url.fronts,
            })
            .collect::<Vec<_>>();

        let nym_vpn_api_url = discovery.nym_vpn_api_url.parse().map_err(|source| {
            DiscoveryFromNymWellknownDiscoveryError::ParseNymVpnApiUrl {
                value: discovery.nym_vpn_api_url,
                source,
            }
        })?;
        let nym_vpn_api_urls = discovery
            .nym_vpn_api_urls
            .into_iter()
            .map(|api_url| ApiUrl {
                url: api_url.url,
                fronts: api_url.fronts,
            })
            .collect::<Vec<_>>();

        Ok(Self {
            network_name: discovery.network_name,
            nym_api_url,
            nym_api_urls,
            nym_vpn_api_url,
            nym_vpn_api_urls,
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
    nym_api_url: Url,
) -> Result<NymNetworkDetailsResponse> {
    tracing::debug!("Fetching nym network details");
    let client = nym_http_api_client::Client::builder(nym_api_url)
        .map_err(Box::new)?
        .build()
        .map_err(Box::new)?;

    client
        .get_network_details()
        .await
        .map_err(Box::new)
        .map_err(Error::GetNetworkDetails)
}

pub(crate) async fn fetch_nym_vpn_network_details(
    nym_vpn_api_url: Url,
) -> Result<NymWellknownDiscoveryItem> {
    tracing::debug!("Fetching nym vpn network details");
    VpnApiClient::new(nym_vpn_api_url, empty_user_agent())
        .map_err(Error::CreateVpnApiClient)?
        .get_wellknown_current_env()
        .await
        .map_err(Error::GetWellKnownCurrentEnv)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use crate::{
        SystemMessage, account_management::AccountManagementPaths, feature_flags::FlagValue,
        system_messages::Properties,
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
        for discovery in [
            Discovery::default_mainnet(),
            Discovery::default_sandbox(),
            Discovery::default_canary(),
        ] {
            let fetched = Discovery::fetch(&discovery.network_name)
                .await
                .expect(&format!(
                    "failed to fetch discovery for {}",
                    discovery.network_name
                ));

            // Only compare the base fields
            assert_eq!(discovery.network_name, fetched.network_name);
            assert_eq!(discovery.nym_api_url, fetched.nym_api_url);
            assert_eq!(discovery.nym_vpn_api_url, fetched.nym_vpn_api_url);
        }
    }

    #[test]
    fn test_parse_discovery_response() {
        let json = r#"{
            "network_name": "qa",
            "nym_api_url": "https://foo.ch/api/",
            "nym_api_urls": [
                {
                    "url": "https://foo.ch/api/",
                    "fronts": ["foobar.ch", "qux.baz"]
                }
            ],
            "nym_vpn_api_url": "https://bar.ch/api/",
            "nym_vpn_api_urls": [
                {
                    "url": "https://bar.ch/api/",
                    "fronts": ["quxbar.ch", "qux.baz"]
                }
            ],
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
            nym_api_urls: vec![ApiUrl {
                url: "https://foo.ch/api/".parse().unwrap(),
                fronts: Some(vec!["foobar.ch".to_owned(), "qux.baz".to_owned()]),
            }],
            nym_vpn_api_url: "https://bar.ch/api/".parse().unwrap(),
            nym_vpn_api_urls: vec![ApiUrl {
                url: "https://bar.ch/api/".parse().unwrap(),
                fronts: Some(vec!["quxbar.ch".to_owned(), "qux.baz".to_owned()]),
            }],
            account_management: Some(AccountManagement {
                url: "https://foobar.ch/".parse().unwrap(),
                paths: AccountManagementPaths {
                    sign_up: "{locale}/account/create".to_owned(),
                    sign_in: "{locale}/account/login".to_owned(),
                    account: "{locale}/account/{account_id}".to_owned(),
                },
            }),
            feature_flags: Some(FeatureFlags::from(HashMap::from([
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
            ]))),
            system_messages: SystemMessages::from(vec![SystemMessage {
                name: "test_message".to_owned(),
                display_from: Some(
                    OffsetDateTime::parse("2024-11-05T12:00:00.000Z", &Rfc3339).unwrap(),
                ),
                display_until: None,
                message: "This is a test message, no need to panic!".to_owned(),
                properties: Some(Properties::from(HashMap::from([(
                    "modal".to_owned(),
                    "true".to_owned(),
                )]))),
            }]),
            system_configuration: None,
        };
        assert_eq!(network, expected_network);
    }
}
