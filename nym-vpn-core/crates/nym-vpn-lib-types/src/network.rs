// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct NymNetworkDetails {
    pub network_name: String,
    pub chain_details: ChainDetails,
    pub endpoints: Vec<ValidatorDetails>,
    pub contracts: NymContracts,
    pub nym_vpn_api_url: Option<String>,
    pub nym_api_urls: Option<Vec<ApiUrl>>,
    pub nym_vpn_api_urls: Option<Vec<ApiUrl>>,
}

#[derive(Clone, Debug)]
pub struct ChainDetails {
    pub bech32_account_prefix: String,
    pub mix_denom: DenomDetailsOwned,
    pub stake_denom: DenomDetailsOwned,
}

#[derive(Clone, Debug)]
pub struct DenomDetailsOwned {
    pub base: String,
    pub display: String,
    pub display_exponent: u32,
}

#[derive(Clone, Debug)]
pub struct ValidatorDetails {
    pub nyxd_url: String,
    pub websocket_url: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NymContracts {
    pub mixnet_contract_address: Option<String>,
    pub vesting_contract_address: Option<String>,
    pub performance_contract_address: Option<String>,
    pub ecash_contract_address: Option<String>,
    pub group_contract_address: Option<String>,
    pub multisig_contract_address: Option<String>,
    pub coconut_dkg_contract_address: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ApiUrl {
    pub url: String,
    pub front_hosts: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct NymVpnNetwork {
    pub nym_vpn_api_url: String,
    pub nym_vpn_api_urls: Vec<ApiUrl>,
}

#[derive(Clone, Debug)]
pub struct Network {
    pub nym_network: NymNetworkDetails,
    pub nyxd_url: String,
    pub api_url: String,
    pub nym_vpn_network: NymVpnNetwork,
    pub feature_flags: Option<FeatureFlags>,
    pub system_configuration: Option<SystemConfiguration>,
}

#[derive(Clone, Debug)]
pub struct FeatureFlags {
    pub flags: HashMap<String, FlagValue>,
}

#[derive(Clone, Debug)]
pub enum FlagValue {
    Value(String),
    Group(HashMap<String, String>),
}

#[derive(Clone, Debug)]
pub struct ParsedAccountLinks {
    pub sign_up: String,
    pub sign_in: String,
    pub account: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct ScoreThresholds {
    pub high: u8,
    pub medium: u8,
    pub low: u8,
}

#[derive(Clone, Debug)]
pub struct SystemConfiguration {
    pub mix_thresholds: ScoreThresholds,
    pub wg_thresholds: ScoreThresholds,
    pub statistics_api: Option<String>,
    pub min_supported_app_versions: Option<NetworkCompatibility>,
}

#[derive(Clone, Debug)]
pub struct SystemMessage {
    pub name: String,
    pub display_from: Option<OffsetDateTime>,
    pub display_until: Option<OffsetDateTime>,
    pub message: String,
    pub properties: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkCompatibility {
    pub core: String,
    pub ios: String,
    pub macos: String,
    pub tauri: String,
    pub android: String,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::SystemConfigurationResponse> for SystemConfiguration {
    fn from(value: nym_vpn_api_client::response::SystemConfigurationResponse) -> Self {
        SystemConfiguration {
            wg_thresholds: ScoreThresholds::from(value.wg_thresholds),
            mix_thresholds: ScoreThresholds::from(value.mix_thresholds),
            statistics_api: value.statistics_api,
            min_supported_app_versions: value
                .min_supported_app_versions
                .map(NetworkCompatibility::from),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::ScoreThresholdsResponse> for ScoreThresholds {
    fn from(value: nym_vpn_api_client::response::ScoreThresholdsResponse) -> Self {
        Self {
            high: value.high,
            medium: value.medium,
            low: value.low,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::NetworkCompatibility> for NetworkCompatibility {
    fn from(value: nym_vpn_api_client::NetworkCompatibility) -> Self {
        Self {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::FeatureFlags> for FeatureFlags {
    fn from(value: nym_vpn_network_config::FeatureFlags) -> Self {
        Self {
            flags: value
                .into_hash_map()
                .into_iter()
                .map(|(k, v)| (k, FlagValue::from(v)))
                .collect(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::FlagValue> for FlagValue {
    fn from(value: nym_vpn_network_config::FlagValue) -> Self {
        match value {
            nym_vpn_network_config::FlagValue::Value(v) => FlagValue::Value(v),
            nym_vpn_network_config::FlagValue::Group(v) => FlagValue::Group(v),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::NymNetwork> for NymNetworkDetails {
    fn from(value: nym_vpn_network_config::NymNetwork) -> Self {
        Self {
            network_name: value.network.network_name,
            chain_details: ChainDetails::from(value.network.chain_details),
            endpoints: value
                .network
                .endpoints
                .into_iter()
                .map(ValidatorDetails::from)
                .collect(),
            contracts: NymContracts::from(value.network.contracts),
            nym_vpn_api_url: value.network.nym_vpn_api_url,
            nym_api_urls: value
                .network
                .nym_api_urls
                .map(|v| v.into_iter().map(ApiUrl::from).collect::<Vec<_>>()),
            nym_vpn_api_urls: value
                .network
                .nym_vpn_api_urls
                .map(|v| v.into_iter().map(ApiUrl::from).collect::<Vec<_>>()),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::ChainDetails> for ChainDetails {
    fn from(value: nym_network_defaults::ChainDetails) -> Self {
        Self {
            bech32_account_prefix: value.bech32_account_prefix,
            mix_denom: DenomDetailsOwned::from(value.mix_denom),
            stake_denom: DenomDetailsOwned::from(value.stake_denom),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::DenomDetailsOwned> for DenomDetailsOwned {
    fn from(value: nym_network_defaults::DenomDetailsOwned) -> Self {
        Self {
            base: value.base,
            display: value.display,
            display_exponent: value.display_exponent,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::NymContracts> for NymContracts {
    fn from(value: nym_network_defaults::NymContracts) -> Self {
        Self {
            mixnet_contract_address: value.mixnet_contract_address,
            vesting_contract_address: value.vesting_contract_address,
            performance_contract_address: value.performance_contract_address,
            ecash_contract_address: value.ecash_contract_address,
            group_contract_address: value.group_contract_address,
            multisig_contract_address: value.multisig_contract_address,
            coconut_dkg_contract_address: value.coconut_dkg_contract_address,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::ApiUrl> for ApiUrl {
    fn from(value: nym_network_defaults::ApiUrl) -> Self {
        Self {
            url: value.url,
            front_hosts: value.front_hosts,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::ApiUrl> for ApiUrl {
    fn from(value: nym_vpn_network_config::ApiUrl) -> Self {
        Self {
            url: value.url,
            front_hosts: value.fronts,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::ValidatorDetails> for ValidatorDetails {
    fn from(value: nym_network_defaults::ValidatorDetails) -> Self {
        Self {
            nyxd_url: value.nyxd_url,
            websocket_url: value.websocket_url,
            api_url: value.api_url,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::NymVpnNetwork> for NymVpnNetwork {
    fn from(value: nym_vpn_network_config::NymVpnNetwork) -> Self {
        Self {
            nym_vpn_api_url: value.nym_vpn_api_url.to_string(),
            nym_vpn_api_urls: value
                .nym_vpn_api_urls
                .into_iter()
                .map(ApiUrl::from)
                .collect(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::ParsedAccountLinks> for ParsedAccountLinks {
    fn from(value: nym_vpn_network_config::ParsedAccountLinks) -> Self {
        Self {
            account: value.account.map(|s| s.to_string()),
            sign_in: value.sign_in.to_string(),
            sign_up: value.sign_up.to_string(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::SystemMessage> for SystemMessage {
    fn from(value: nym_vpn_network_config::SystemMessage) -> Self {
        Self {
            name: value.name,
            message: value.message,
            display_from: value.display_from,
            display_until: value.display_until,
            properties: value.properties.map(|v| v.into_inner()),
        }
    }
}
