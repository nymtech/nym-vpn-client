// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AccountManagement, FeatureFlags, SystemMessages, system_configuration::SystemConfiguration,
};
pub use nym_network_defaults::v2::{DnsFallback, NetworkingSpecifics};
use nym_vpn_api_client::response::{ApiUrl as LegacyApiUrl, NymWellknownDiscoveryItemResponse};

static MAINNET_DISCOVERY_JSON: &[u8] = include_bytes!("../default/mainnet_discovery.json");
static SANDBOX_DISCOVERY_JSON: &[u8] = include_bytes!("../default/sandbox_discovery.json");
static CANARY_DISCOVERY_JSON: &[u8] = include_bytes!("../default/canary_discovery.json");

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(from = "DiscoveryOnDisk")]
pub struct Discovery {
    // Base network setup
    pub network_name: String,

    // Use the getters!
    pub networking: NetworkingSpecifics,

    // Additional context
    pub account_management: Option<AccountManagement>,
    pub feature_flags: Option<FeatureFlags>,
    pub system_configuration: Option<SystemConfiguration>,

    #[serde(default)]
    pub system_messages: SystemMessages,
}

/// On-disk shapes accepted when deserializing a [`Discovery`], oldest tried last.
///
/// Prior to the introduction of the nested `networking` field, a persisted `Discovery` stored
/// `nym_api_url`/`nym_api_urls`/`nym_vpn_api_url`/`nym_vpn_api_urls` directly on the struct. This
/// lets clients that persisted the old shape upgrade without hitting a deserialization error and
/// losing/refetching their cached discovery.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum DiscoveryOnDisk {
    Current(CurrentDiscovery),
    LegacyFlatNetworking(LegacyDiscovery),
}

#[derive(serde::Deserialize)]
struct CurrentDiscovery {
    network_name: String,
    networking: NetworkingSpecifics,
    account_management: Option<AccountManagement>,
    feature_flags: Option<FeatureFlags>,
    system_configuration: Option<SystemConfiguration>,
    #[serde(default)]
    system_messages: SystemMessages,
}

#[derive(serde::Deserialize)]
struct LegacyDiscovery {
    network_name: String,
    nym_api_url: url::Url,
    nym_api_urls: Vec<LegacyApiUrl>,
    nym_vpn_api_url: url::Url,
    nym_vpn_api_urls: Vec<LegacyApiUrl>,
    account_management: Option<AccountManagement>,
    feature_flags: Option<FeatureFlags>,
    system_configuration: Option<SystemConfiguration>,
    #[serde(default)]
    system_messages: SystemMessages,
}

impl From<DiscoveryOnDisk> for Discovery {
    fn from(value: DiscoveryOnDisk) -> Self {
        match value {
            DiscoveryOnDisk::Current(current) => Self {
                network_name: current.network_name,
                networking: current.networking,
                account_management: current.account_management,
                feature_flags: current.feature_flags,
                system_configuration: current.system_configuration,
                system_messages: current.system_messages,
            },
            DiscoveryOnDisk::LegacyFlatNetworking(legacy) => legacy.into(),
        }
    }
}

impl From<LegacyDiscovery> for Discovery {
    fn from(legacy: LegacyDiscovery) -> Self {
        let to_api_urls = |single: url::Url, many: Vec<LegacyApiUrl>| {
            if many.is_empty() {
                vec![nym_network_defaults::ApiUrl {
                    url: single.to_string(),
                    front_hosts: None,
                }]
            } else {
                many.into_iter()
                    .map(|api_url| nym_network_defaults::ApiUrl {
                        url: api_url.url,
                        front_hosts: api_url.fronts,
                    })
                    .collect()
            }
        };

        Self {
            network_name: legacy.network_name,
            networking: NetworkingSpecifics {
                nym_api_urls: to_api_urls(legacy.nym_api_url, legacy.nym_api_urls),
                nym_vpn_api_urls: to_api_urls(legacy.nym_vpn_api_url, legacy.nym_vpn_api_urls),
                dns_fallbacks: Vec::new(),
            },
            account_management: legacy.account_management,
            feature_flags: legacy.feature_flags,
            system_configuration: legacy.system_configuration,
            system_messages: legacy.system_messages,
        }
    }
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
        self.networking.nym_api_urls.clone()
    }

    pub fn nym_vpn_api_urls(&self) -> Vec<nym_network_defaults::ApiUrl> {
        self.networking.nym_vpn_api_urls.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryFromNymWellknownDiscoveryError {}

impl From<NymWellknownDiscoveryItemResponse> for Discovery {
    fn from(discovery: NymWellknownDiscoveryItemResponse) -> Self {
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

        let networking = discovery.networking.unwrap_or_else(|| {
            tracing::warn!("Discovery response is missing the networking section");
            NetworkingSpecifics {
                nym_api_urls: Vec::new(),
                nym_vpn_api_urls: Vec::new(),
                dns_fallbacks: Vec::new(),
            }
        });

        Self {
            network_name: discovery.network_name,
            networking,
            account_management,
            feature_flags,
            system_configuration,
            system_messages,
        }
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

    fn assert_valid_dns_fallbacks(dns_fallbacks: &[DnsFallback]) {
        assert!(
            !dns_fallbacks.is_empty(),
            "expected at least one dns fallback entry"
        );
        for fallback in dns_fallbacks {
            assert!(
                !fallback.url.is_empty(),
                "dns fallback url must not be empty"
            );
            assert!(
                !fallback.addresses.is_empty(),
                "dns fallback for '{}' must have at least one address",
                fallback.url
            );
            for address in &fallback.addresses {
                address.parse::<std::net::IpAddr>().unwrap_or_else(|err| {
                    panic!(
                        "invalid dns fallback address '{address}' for '{}': {err}",
                        fallback.url
                    )
                });
            }
        }
    }

    #[test]
    fn test_default_mainnet_has_valid_dns_fallbacks() {
        assert_valid_dns_fallbacks(&Discovery::default_mainnet().networking.dns_fallbacks);
    }

    #[tokio::test]
    async fn test_mainnet_live_discovery_has_valid_dns_fallbacks() {
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None).unwrap();
        let discovery = fetcher.fetch_discovery("mainnet").await.unwrap();
        assert_valid_dns_fallbacks(&discovery.networking.dns_fallbacks);
    }

    async fn test_discovery_equality(discovery: Discovery) {
        let fetcher = Fetcher::new(Discovery::default_mainnet(), None).unwrap();
        let fetched = fetcher
            .fetch_discovery(&discovery.network_name)
            .await
            .unwrap();

        // Only compare the base fields
        assert_eq!(discovery.network_name, fetched.network_name);
    }

    #[test]
    fn test_parse_discovery_response() {
        let json = r#"{
            "network_name": "qa",
            "networking": {
                "nym_api_urls": [
                    {
                        "url": "https://foo.ch/api/",
                        "front_hosts": ["foobar.ch", "qux.baz"]
                    }
                ],
                "nym_vpn_api_urls": [
                    {
                        "url": "https://bar.ch/api/",
                        "front_hosts": ["quxbar.ch", "qux.baz"]
                    }
                ],
                "dns_fallbacks": [
                    {
                        "url": "foo.ch",
                        "addresses": ["1.2.3.4"]
                    }
                ]
            },
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
        let network: Discovery = discovery.into();

        let expected_network = Discovery {
            network_name: "qa".to_owned(),
            networking: NetworkingSpecifics {
                nym_api_urls: vec![nym_network_defaults::ApiUrl {
                    url: "https://foo.ch/api/".to_owned(),
                    front_hosts: Some(vec!["foobar.ch".to_owned(), "qux.baz".to_owned()]),
                }],
                nym_vpn_api_urls: vec![nym_network_defaults::ApiUrl {
                    url: "https://bar.ch/api/".to_owned(),
                    front_hosts: Some(vec!["quxbar.ch".to_owned(), "qux.baz".to_owned()]),
                }],
                dns_fallbacks: vec![DnsFallback {
                    url: "foo.ch".to_owned(),
                    addresses: vec!["1.2.3.4".to_owned()],
                }],
            },
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

    /// Prior to the `networking` field being introduced, a persisted [`Discovery`] stored its
    /// api urls flattened directly on the struct (`nym_api_url`, `nym_api_urls`,
    /// `nym_vpn_api_url`, `nym_vpn_api_urls`). Clients that persisted a cache in that shape must
    /// be able to load it after upgrading instead of hitting a deserialization error.
    #[test]
    fn test_deserialize_legacy_flat_networking_format() {
        let json = r#"{
            "network_name": "mainnet",
            "nym_api_url": "https://api.foo.ch/",
            "nym_api_urls": [
                {
                    "url": "https://api.foo.ch/",
                    "fronts": ["foobar.ch"]
                }
            ],
            "nym_vpn_api_url": "https://vpn-api.foo.ch/",
            "nym_vpn_api_urls": [],
            "account_management": null,
            "feature_flags": null,
            "system_configuration": null,
            "system_messages": []
        }"#;

        let discovery: Discovery = serde_json::from_str(json).unwrap();

        assert_eq!(discovery.network_name, "mainnet");
        assert_eq!(
            discovery.networking.nym_api_urls,
            vec![nym_network_defaults::ApiUrl {
                url: "https://api.foo.ch/".to_owned(),
                front_hosts: Some(vec!["foobar.ch".to_owned()]),
            }]
        );
        // Falls back to the singular field since the plural one is empty, matching the
        // pre-migration getter behaviour.
        assert_eq!(
            discovery.networking.nym_vpn_api_urls,
            vec![nym_network_defaults::ApiUrl {
                url: "https://vpn-api.foo.ch/".to_owned(),
                front_hosts: None,
            }]
        );
        assert!(discovery.networking.dns_fallbacks.is_empty());
        assert_eq!(discovery.account_management, None);
        assert_eq!(discovery.feature_flags, None);
        assert_eq!(discovery.system_configuration, None);
    }

    /// Round-tripping through [`PersistentRecord`] (i.e. the actual on-disk cache format) must
    /// also tolerate the legacy shape, since that's the type that gets serialized to/read from
    /// disk by [`crate::persistent_discovery::PersistentDiscovery`].
    #[test]
    fn test_deserialize_legacy_persistent_record() {
        let json = r#"{
            "updated_at": null,
            "value": {
                "network_name": "sandbox",
                "nym_api_url": "https://api.sandbox.ch/",
                "nym_api_urls": [],
                "nym_vpn_api_url": "https://vpn-api.sandbox.ch/",
                "nym_vpn_api_urls": [],
                "account_management": null,
                "feature_flags": null,
                "system_configuration": null,
                "system_messages": []
            }
        }"#;

        let record: crate::PersistentRecord<Box<Discovery>> = serde_json::from_str(json).unwrap();
        assert_eq!(record.value.network_name, "sandbox");
        assert_eq!(
            record.value.networking.nym_api_urls,
            vec![nym_network_defaults::ApiUrl {
                url: "https://api.sandbox.ch/".to_owned(),
                front_hosts: None,
            }]
        );
    }
}
