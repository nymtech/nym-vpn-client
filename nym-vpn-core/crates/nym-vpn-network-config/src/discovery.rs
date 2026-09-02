// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AccountManagement, FeatureFlags, Result, SystemMessages,
    system_configuration::SystemConfiguration,
};
use nym_vpn_api_client::response::{ApiUrl, NymWellknownDiscoveryItemResponse};

static MAINNET_DISCOVERY_JSON: &[u8] = include_bytes!("../default/mainnet_discovery.json");
static SANDBOX_DISCOVERY_JSON: &[u8] = include_bytes!("../default/sandbox_discovery.json");
static CANARY_DISCOVERY_JSON: &[u8] = include_bytes!("../default/canary_discovery.json");

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Discovery {
    // Base network setup
    pub network_name: String,

    // Use the getters!
    nym_api_url: url::Url,
    nym_api_urls: Vec<ApiUrl>,
    nym_vpn_api_url: url::Url,
    nym_vpn_api_urls: Vec<ApiUrl>,
    #[serde(default)]
    nyxd_urls: Vec<url::Url>,

    // Additional context
    pub account_management: Option<AccountManagement>,
    pub feature_flags: Option<FeatureFlags>,
    pub system_configuration: Option<SystemConfiguration>,

    #[serde(default)]
    pub system_messages: SystemMessages,
}

impl Discovery {
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

    pub fn default_discovery(network_name: &str) -> Option<Self> {
        Some(match network_name {
            "mainnet" => Self::default_mainnet(),
            "sandbox" => Self::default_sandbox(),
            "canary" => Self::default_canary(),
            _ => None?,
        })
    }

    pub fn nym_api_urls(&self) -> Vec<nym_network_defaults::ApiUrl> {
        if self.nym_api_urls.is_empty() {
            vec![nym_network_defaults::ApiUrl {
                url: self.nym_api_url.to_string(),
                front_hosts: None,
            }]
        } else {
            self.nym_api_urls
                .iter()
                .map(|api_url| nym_network_defaults::ApiUrl {
                    url: api_url.url.clone(),
                    front_hosts: api_url.fronts.clone(),
                })
                .collect()
        }
    }

    pub fn nym_vpn_api_urls(&self) -> Vec<nym_network_defaults::ApiUrl> {
        if self.nym_vpn_api_urls.is_empty() {
            vec![nym_network_defaults::ApiUrl {
                url: self.nym_vpn_api_url.to_string(),
                front_hosts: None,
            }]
        } else {
            self.nym_vpn_api_urls
                .iter()
                .map(|api_url| nym_network_defaults::ApiUrl {
                    url: api_url.url.clone(),
                    front_hosts: api_url.fronts.clone(),
                })
                .collect()
        }
    }

    /// Nyxd RPC endpoint pool. Falls back to the bundled defaults for this
    /// network when the stored discovery doesn't carry any (e.g. a remote
    /// discovery response predating this field).
    pub fn nyxd_urls(&self) -> Vec<url::Url> {
        if !self.nyxd_urls.is_empty() {
            return self.nyxd_urls.clone();
        }
        Self::default_discovery(&self.network_name)
            .map(|d| d.nyxd_urls)
            .unwrap_or_default()
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

        let nym_api_urls = discovery.nym_api_urls.clone();

        let nym_vpn_api_url = discovery.nym_vpn_api_url.parse().map_err(|source| {
            DiscoveryFromNymWellknownDiscoveryError::ParseNymVpnApiUrl {
                value: discovery.nym_vpn_api_url,
                source,
            }
        })?;
        let nym_vpn_api_urls = discovery.nym_vpn_api_urls.clone();

        let nyxd_urls = discovery
            .nyxd_urls
            .unwrap_or_default()
            .iter()
            .filter_map(|raw| {
                raw.parse::<url::Url>()
                    .inspect_err(|err| {
                        tracing::warn!("Skipping invalid nyxd url in discovery: {raw}: {err}")
                    })
                    .ok()
            })
            .collect();

        Ok(Self {
            network_name: discovery.network_name,
            nym_api_url,
            nym_api_urls,
            nym_vpn_api_url,
            nym_vpn_api_urls,
            nyxd_urls,
            account_management,
            feature_flags,
            system_configuration,
            system_messages,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::*;
    use crate::{
        SystemMessage,
        account_management::{
            AccountManagementAutologinPaths, AccountManagementPaths, AccountManagementPrivyPaths,
        },
        feature_flags::FlagValue,
        fetcher::Fetcher,
        system_messages::Properties,
    };

    #[tokio::test]
    async fn test_mainnet_discovery_same_as_fetched() {
        test_discovery_equality(Discovery::default_mainnet()).await;
    }

    #[tokio::test]
    async fn test_sandbox_discovery_same_as_fetched() {
        test_discovery_equality(Discovery::default_sandbox()).await;
    }

    #[tokio::test]
    async fn test_canary_discovery_same_as_fetched() {
        test_discovery_equality(Discovery::default_canary()).await;
    }

    async fn test_discovery_equality(discovery: Discovery) {
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None, None).unwrap();
        let fetched = fetcher
            .fetch_discovery(&discovery.network_name)
            .await
            .unwrap();

        // Only compare the base fields
        assert_eq!(discovery.network_name, fetched.network_name);
        assert_eq!(discovery.nym_api_url, fetched.nym_api_url);
        assert_eq!(discovery.nym_vpn_api_url, fetched.nym_vpn_api_url);
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
                    "account": "{locale}/account/{account_id}",
                    "privy": {
                        "mobile": "{locale}/auth/privy",
                        "desktop": "{locale}/auth/privy",
                        "web": "{locale}/auth/privy"
                    },
                    "autologin": {
                        "mobile": "{locale}/account/login/autologin/mobile",
                        "desktop": "{locale}/account/login/autologin/desktop",
                        "web": "{locale}/account/login/autologin/web"
                    },
                    "pricing": "{locale}/pricing"
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
            nyxd_urls: vec![],
            account_management: Some(AccountManagement {
                url: "https://foobar.ch/".parse().unwrap(),
                paths: AccountManagementPaths {
                    sign_up: "{locale}/account/create".to_owned(),
                    sign_in: "{locale}/account/login".to_owned(),
                    account: "{locale}/account/{account_id}".to_owned(),
                    privy: AccountManagementPrivyPaths {
                        mobile: "{locale}/auth/privy".to_owned(),
                        desktop: "{locale}/auth/privy".to_owned(),
                        web: "{locale}/auth/privy".to_owned(),
                    },
                    autologin: AccountManagementAutologinPaths {
                        mobile: "{locale}/account/login/autologin/mobile".to_owned(),
                        desktop: "{locale}/account/login/autologin/desktop".to_owned(),
                        web: "{locale}/account/login/autologin/web".to_owned(),
                    },
                    pricing: "{locale}/pricing".to_owned(),
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

    #[test]
    fn test_mainnet_default_nym_api_pool_includes_signers() {
        let urls = Discovery::default_mainnet().nym_api_urls();
        assert_eq!(urls.len(), 15);
        assert_eq!(urls[0].url, "https://validator.nymtech.net/api/");
        assert!(urls.iter().any(|u| u.url == "https://nym-api.nodes.guru/"));
    }

    #[test]
    fn test_mainnet_default_has_nyxd_pool() {
        let discovery = Discovery::default_mainnet();
        let urls = discovery.nyxd_urls();
        assert_eq!(urls.len(), 4);
        assert_eq!(urls[0].as_str(), "https://rpc.nymtech.net/");
    }

    #[test]
    fn test_nyxd_urls_fall_back_to_bundled_default_when_absent() {
        // Simulates a remote discovery response that doesn't carry nyxd_urls:
        // the bundled mainnet pool must still apply.
        let json = r#"{
            "network_name": "mainnet",
            "nym_api_url": "https://validator.nymtech.net/api/",
            "nym_api_urls": [],
            "nym_vpn_api_url": "https://nymvpn.com/api/",
            "nym_vpn_api_urls": []
        }"#;
        let response: NymWellknownDiscoveryItemResponse = serde_json::from_str(json).unwrap();
        let discovery: Discovery = response.try_into().unwrap();
        assert_eq!(discovery.nyxd_urls().len(), 4);
    }

    #[test]
    fn test_nyxd_urls_from_response_override_default() {
        let json = r#"{
            "network_name": "mainnet",
            "nym_api_url": "https://validator.nymtech.net/api/",
            "nym_api_urls": [],
            "nym_vpn_api_url": "https://nymvpn.com/api/",
            "nym_vpn_api_urls": [],
            "nyxd_urls": ["https://rpc.example.com/", "not a url"]
        }"#;
        let response: NymWellknownDiscoveryItemResponse = serde_json::from_str(json).unwrap();
        let discovery: Discovery = response.try_into().unwrap();
        // invalid entries skipped with a warning, valid ones kept
        assert_eq!(
            discovery.nyxd_urls(),
            vec!["https://rpc.example.com/".parse::<url::Url>().unwrap()]
        );
    }
}
