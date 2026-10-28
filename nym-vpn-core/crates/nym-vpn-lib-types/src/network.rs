// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct NymNetworkDetails {
    #[cfg_attr(feature = "cbor", n(0))]
    pub network_name: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub chain_details: ChainDetails,
    #[cfg_attr(feature = "cbor", n(2))]
    pub endpoints: Vec<ValidatorDetails>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub contracts: NymContracts,
    #[cfg_attr(feature = "cbor", n(4))]
    pub networking: NymNetworkingSpecifics,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct NymNetworkingSpecifics {
    #[cfg_attr(feature = "cbor", n(0))]
    pub nym_api_urls: Vec<ApiUrl>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub nym_vpn_api_urls: Vec<ApiUrl>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub dns_fallbacks: Vec<DnsFallback>,
    // pub internal_nameservers: std::any::Any,
    // pub covert channels: std::any::Any,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DnsFallback {
    #[cfg_attr(feature = "cbor", n(0))]
    pub url: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub addresses: Vec<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ChainDetails {
    #[cfg_attr(feature = "cbor", n(0))]
    pub bech32_account_prefix: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub mix_denom: DenomDetailsOwned,
    #[cfg_attr(feature = "cbor", n(2))]
    pub stake_denom: DenomDetailsOwned,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct DenomDetailsOwned {
    #[cfg_attr(feature = "cbor", n(0))]
    pub base: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub display: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub display_exponent: u32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ValidatorDetails {
    #[cfg_attr(feature = "cbor", n(0))]
    pub nyxd_url: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub websocket_url: Option<String>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub api_url: Option<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct NymContracts {
    #[cfg_attr(feature = "cbor", n(0))]
    pub mixnet_contract_address: Option<String>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub vesting_contract_address: Option<String>,
    #[cfg_attr(feature = "cbor", n(2))]
    pub performance_contract_address: Option<String>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub ecash_contract_address: Option<String>,
    #[cfg_attr(feature = "cbor", n(4))]
    pub group_contract_address: Option<String>,
    #[cfg_attr(feature = "cbor", n(5))]
    pub multisig_contract_address: Option<String>,
    #[cfg_attr(feature = "cbor", n(6))]
    pub coconut_dkg_contract_address: Option<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ApiUrl {
    #[cfg_attr(feature = "cbor", n(0))]
    pub url: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub front_hosts: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct NymVpnNetwork {
    #[cfg_attr(feature = "cbor", n(0))]
    pub nym_vpn_api_urls: Vec<ApiUrl>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct Network {
    #[cfg_attr(feature = "cbor", n(0))]
    pub nym_network: NymNetworkDetails,
    #[cfg_attr(feature = "cbor", n(1))]
    pub nyxd_url: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub nym_vpn_network: NymVpnNetwork,
    #[cfg_attr(feature = "cbor", n(3))]
    #[cfg_attr(feature = "cbor", cbor(with = "crate::cbor::option_arc", has_nil))]
    pub feature_flags: Option<Arc<FeatureFlags>>,
    #[cfg_attr(feature = "cbor", n(4))]
    pub system_configuration: Option<SystemConfiguration>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Object))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct FeatureFlags {
    #[cfg_attr(feature = "cbor", n(0))]
    pub flags: HashMap<String, FlagValue>,
}

#[cfg(feature = "uniffi-bindings")]
#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
impl FeatureFlags {
    /// If domain fronting is enabled or not, if set
    #[uniffi::method]
    pub fn is_domain_fronting_enabled(&self) -> Option<bool> {
        // todo: harmonize with nym-vpn-network-config/src/feature_flags.rs
        self.get_group_flag("domain_fronting", "enabled")
    }

    /// If quic is enabled or not, if set
    #[uniffi::method]
    pub fn is_quic_enabled(&self) -> Option<bool> {
        // todo: harmonize with nym-vpn-network-config/src/feature_flags.rs
        self.get_group_flag("quic", "enabled")
    }

    /// If privy is enabled or not, if set
    #[uniffi::method]
    pub fn is_privy_enabled(&self) -> Option<bool> {
        // todo: harmonize with nym-vpn-network-config/src/feature_flags.rs
        self.get_group_flag("privy", "enabled")
    }

    fn get_group_flag(&self, group_name: &str, flag_name: &str) -> Option<bool> {
        if let Some(FlagValue::Group(group)) = self.flags.get(group_name)
            && let Some(value) = group.get(flag_name)
        {
            Some(value == "true")
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum FlagValue {
    #[cfg_attr(feature = "cbor", n(0))]
    Value(#[cfg_attr(feature = "cbor", n(0))] String),
    #[cfg_attr(feature = "cbor", n(1))]
    Group(#[cfg_attr(feature = "cbor", n(0))] HashMap<String, String>),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ParsedAccountLinks {
    #[cfg_attr(feature = "cbor", n(0))]
    pub sign_up: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub sign_in: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub account: Option<String>,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ScoreThresholds {
    #[cfg_attr(feature = "cbor", n(0))]
    pub high: u8,
    #[cfg_attr(feature = "cbor", n(1))]
    pub medium: u8,
    #[cfg_attr(feature = "cbor", n(2))]
    pub low: u8,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct SystemConfiguration {
    #[cfg_attr(feature = "cbor", n(0))]
    pub mix_thresholds: ScoreThresholds,
    #[cfg_attr(feature = "cbor", n(1))]
    pub wg_thresholds: ScoreThresholds,
    #[cfg_attr(feature = "cbor", n(2))]
    pub statistics_api: Option<String>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub min_supported_app_versions: Option<NetworkCompatibility>,
    #[cfg_attr(feature = "cbor", n(4))]
    pub app_update_policy: String,
    #[cfg_attr(feature = "cbor", n(5))]
    pub app_update_policies: HashMap<String, String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct SystemMessage {
    #[cfg_attr(feature = "cbor", n(0))]
    pub name: String,
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601::option"))]
    #[cfg_attr(feature = "cbor", n(1))]
    #[cfg_attr(
        feature = "cbor",
        cbor(with = "crate::cbor::option_offset_date_time", has_nil)
    )]
    pub display_from: Option<OffsetDateTime>,
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601::option"))]
    #[cfg_attr(feature = "cbor", n(2))]
    #[cfg_attr(
        feature = "cbor",
        cbor(with = "crate::cbor::option_offset_date_time", has_nil)
    )]
    pub display_until: Option<OffsetDateTime>,
    #[cfg_attr(feature = "cbor", n(3))]
    pub message: String,
    #[cfg_attr(feature = "cbor", n(4))]
    pub properties: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct NetworkCompatibility {
    #[cfg_attr(feature = "cbor", n(0))]
    pub core: String,
    #[cfg_attr(feature = "cbor", n(1))]
    pub ios: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub macos: String,
    #[cfg_attr(feature = "cbor", n(3))]
    pub tauri: String,
    #[cfg_attr(feature = "cbor", n(4))]
    pub android: String,
}

/// Everything an app needs to decide whether to ask for, or require, an update.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct AppCompatibility {
    #[cfg_attr(feature = "cbor", n(0))]
    pub network: Option<NetworkCompatibility>,
    #[cfg_attr(feature = "cbor", n(1))]
    pub app_update_policy: String,
    #[cfg_attr(feature = "cbor", n(2))]
    pub app_update_policies: HashMap<String, String>,
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
            app_update_policy: value.app_update_policy,
            app_update_policies: value.app_update_policies,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<&nym_vpn_network_config::SystemConfiguration> for AppCompatibility {
    fn from(value: &nym_vpn_network_config::SystemConfiguration) -> Self {
        Self {
            network: value
                .min_supported_app_versions
                .clone()
                .map(NetworkCompatibility::from),
            app_update_policy: value.app_update_policy.clone(),
            app_update_policies: value.app_update_policies.clone(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::SystemConfiguration> for SystemConfiguration {
    fn from(value: nym_vpn_network_config::SystemConfiguration) -> Self {
        SystemConfiguration {
            wg_thresholds: ScoreThresholds::from(value.wg_thresholds),
            mix_thresholds: ScoreThresholds::from(value.mix_thresholds),
            statistics_api: value.statistics_api.map(|url| url.to_string()),
            min_supported_app_versions: value
                .min_supported_app_versions
                .map(NetworkCompatibility::from),
            app_update_policy: value.app_update_policy,
            app_update_policies: value.app_update_policies,
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
impl From<nym_vpn_network_config::ScoreThresholds> for ScoreThresholds {
    fn from(value: nym_vpn_network_config::ScoreThresholds) -> Self {
        Self {
            high: value.high,
            medium: value.medium,
            low: value.low,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_network_config::Network> for Network {
    fn from(value: nym_vpn_network_config::Network) -> Self {
        Self {
            nym_network: NymNetworkDetails::from(value.nym_network),
            nyxd_url: value.nyxd_url.to_string(),
            nym_vpn_network: NymVpnNetwork::from(value.nym_vpn_network),
            feature_flags: value.feature_flags.map(FeatureFlags::from).map(Arc::from),
            system_configuration: value.system_configuration.map(SystemConfiguration::from),
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
impl From<nym_vpn_network_config::NymNetworkDetails> for NymNetworkDetails {
    fn from(value: nym_vpn_network_config::NymNetworkDetails) -> Self {
        Self {
            network_name: value.network_name,
            chain_details: ChainDetails::from(value.chain_details),
            endpoints: value
                .endpoints
                .into_iter()
                .map(ValidatorDetails::from)
                .collect(),
            contracts: NymContracts::from(value.contracts),
            networking: value.networking.clone().into(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::v2::NetworkingSpecifics> for NymNetworkingSpecifics {
    fn from(value: nym_network_defaults::v2::NetworkingSpecifics) -> Self {
        Self {
            nym_api_urls: value.nym_api_urls.into_iter().map(ApiUrl::from).collect(),
            nym_vpn_api_urls: value
                .nym_vpn_api_urls
                .into_iter()
                .map(ApiUrl::from)
                .collect(),
            dns_fallbacks: value
                .dns_fallbacks
                .into_iter()
                .map(DnsFallback::from)
                .collect(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_network_defaults::v2::DnsFallback> for DnsFallback {
    fn from(value: nym_network_defaults::v2::DnsFallback) -> Self {
        Self {
            url: value.url,
            addresses: value.addresses,
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
