// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

/// Represents the nym network environment together with the environment specific to nym-vpn. These
/// need to be exported to the environment (for now, until it's refactored internally in the nym
/// crates) so that the client can have access to the necessary information.
///
/// The list is as of today:
///
/// NETWORK_NAME = nym_network::network_name
///
/// BECH32_PREFIX = nym_network::chain_details::bech32_account_prefix
/// MIX_DENOM = nym_network::chain_details::mix_denom::base
/// MIX_DENOM_DISPLAY = nym_network::chain_details::mix_denom::display
/// STAKE_DENOM = nym_network::chain_details::stake_denom::base
/// STAKE_DENOM_DISPLAY = nym_network::chain_details::stake_denom::display
/// DENOMS_EXPONENT = nym_network::chain_details::mix_denom::display_exponent
///
/// MIXNET_CONTRACT_ADDRESS = nym_network::contracts::mixnet_contract_address
/// VESTING_CONTRACT_ADDRESS = nym_network::contracts::vesting_contract_address
/// GROUP_CONTRACT_ADDRESS = nym_network::contracts::group_contract_address
/// ECASH_CONTRACT_ADDRESS = nym_network::contracts::ecash_contract_address
/// MULTISIG_CONTRACT_ADDRESS = nym_network::contracts::multisig_contract_address
/// COCONUT_DKG_CONTRACT_ADDRESS = nym_network::contracts::coconut_dkg_contract_address
///
/// NYXD = nym_network::endpoints[0]::nyxd_url
/// NYM_API = nym_network::endpoints[0]::api_url
/// NYXD_WS = nym_network::endpoints[0]::websocket_url
///
/// NYM_VPN_API = nym_vpn_network::nym_vpn_api_url
#[derive(uniffi::Record)]
pub struct NetworkEnvironment {
    pub nym_network: NymNetworkDetails,
    pub nym_vpn_network: NymVpnNetwork,
    pub feature_flags: Option<FeatureFlags>,
}

impl From<nym_vpn_lib_types::Network> for NetworkEnvironment {
    fn from(value: nym_vpn_lib_types::Network) -> Self {
        NetworkEnvironment {
            nym_network: value.nym_network.into(),
            nym_vpn_network: value.nym_vpn_network.into(),
            feature_flags: value.feature_flags.map(FeatureFlags::from),
        }
    }
}

impl From<nym_vpn_network_config::Network> for NetworkEnvironment {
    fn from(network: nym_vpn_network_config::Network) -> Self {
        NetworkEnvironment {
            nym_network: NymNetworkDetails::from(network.nym_network.network),
            nym_vpn_network: NymVpnNetwork::from(network.nym_vpn_network),
            feature_flags: network.feature_flags.map(FeatureFlags::from),
        }
    }
}

#[derive(uniffi::Record)]
pub struct NymNetworkDetails {
    pub network_name: String,
    pub chain_details: ChainDetails,
    pub endpoints: Vec<ValidatorDetails>,
    pub contracts: NymContracts,
}

impl From<nym_vpn_lib_types::NymNetworkDetails> for NymNetworkDetails {
    fn from(value: nym_vpn_lib_types::NymNetworkDetails) -> Self {
        NymNetworkDetails {
            network_name: value.network_name,
            chain_details: value.chain_details.into(),
            endpoints: value
                .endpoints
                .into_iter()
                .map(ValidatorDetails::from)
                .collect(),
            contracts: value.contracts.into(),
        }
    }
}

impl From<nym_config::defaults::NymNetworkDetails> for NymNetworkDetails {
    fn from(value: nym_config::defaults::NymNetworkDetails) -> Self {
        NymNetworkDetails {
            network_name: value.network_name,
            chain_details: value.chain_details.into(),
            endpoints: value
                .endpoints
                .into_iter()
                .map(ValidatorDetails::from)
                .collect(),
            contracts: value.contracts.into(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct ChainDetails {
    pub bech32_account_prefix: String,
    pub mix_denom: DenomDetails,
    pub stake_denom: DenomDetails,
}

impl From<nym_vpn_lib_types::ChainDetails> for ChainDetails {
    fn from(value: nym_vpn_lib_types::ChainDetails) -> Self {
        ChainDetails {
            bech32_account_prefix: value.bech32_account_prefix,
            mix_denom: value.mix_denom.into(),
            stake_denom: value.stake_denom.into(),
        }
    }
}

impl From<nym_config::defaults::ChainDetails> for ChainDetails {
    fn from(value: nym_config::defaults::ChainDetails) -> Self {
        ChainDetails {
            bech32_account_prefix: value.bech32_account_prefix,
            mix_denom: value.mix_denom.into(),
            stake_denom: value.stake_denom.into(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct DenomDetails {
    pub base: String,
    pub display: String,
    pub display_exponent: u32,
}

impl From<nym_vpn_lib_types::DenomDetailsOwned> for DenomDetails {
    fn from(value: nym_vpn_lib_types::DenomDetailsOwned) -> Self {
        DenomDetails {
            base: value.base,
            display: value.display,
            display_exponent: value.display_exponent,
        }
    }
}

impl From<nym_config::defaults::DenomDetailsOwned> for DenomDetails {
    fn from(value: nym_config::defaults::DenomDetailsOwned) -> Self {
        DenomDetails {
            base: value.base,
            display: value.display,
            display_exponent: value.display_exponent,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ValidatorDetails {
    pub nyxd_url: String,
    pub websocket_url: Option<String>,
    pub api_url: Option<String>,
}

impl From<nym_vpn_lib_types::ValidatorDetails> for ValidatorDetails {
    fn from(value: nym_vpn_lib_types::ValidatorDetails) -> Self {
        ValidatorDetails {
            nyxd_url: value.nyxd_url,
            websocket_url: value.websocket_url,
            api_url: value.api_url,
        }
    }
}

impl From<nym_config::defaults::ValidatorDetails> for ValidatorDetails {
    fn from(value: nym_config::defaults::ValidatorDetails) -> Self {
        ValidatorDetails {
            nyxd_url: value.nyxd_url,
            websocket_url: value.websocket_url,
            api_url: value.api_url,
        }
    }
}

#[derive(uniffi::Record)]
pub struct NymContracts {
    pub mixnet_contract_address: Option<String>,
    pub vesting_contract_address: Option<String>,
    pub ecash_contract_address: Option<String>,
    pub group_contract_address: Option<String>,
    pub multisig_contract_address: Option<String>,
    pub coconut_dkg_contract_address: Option<String>,
}

impl From<nym_vpn_lib_types::NymContracts> for NymContracts {
    fn from(value: nym_vpn_lib_types::NymContracts) -> Self {
        NymContracts {
            mixnet_contract_address: value.mixnet_contract_address,
            vesting_contract_address: value.vesting_contract_address,
            ecash_contract_address: value.ecash_contract_address,
            group_contract_address: value.group_contract_address,
            multisig_contract_address: value.multisig_contract_address,
            coconut_dkg_contract_address: value.coconut_dkg_contract_address,
        }
    }
}

impl From<nym_config::defaults::NymContracts> for NymContracts {
    fn from(value: nym_config::defaults::NymContracts) -> Self {
        NymContracts {
            mixnet_contract_address: value.mixnet_contract_address,
            vesting_contract_address: value.vesting_contract_address,
            ecash_contract_address: value.ecash_contract_address,
            group_contract_address: value.group_contract_address,
            multisig_contract_address: value.multisig_contract_address,
            coconut_dkg_contract_address: value.coconut_dkg_contract_address,
        }
    }
}

#[derive(uniffi::Record)]
pub struct NymVpnNetwork {
    pub nym_vpn_api_url: String,
    // todo: missing fields
}

impl From<nym_vpn_lib_types::NymVpnNetwork> for NymVpnNetwork {
    fn from(value: nym_vpn_lib_types::NymVpnNetwork) -> Self {
        NymVpnNetwork {
            nym_vpn_api_url: value.nym_vpn_api_url.to_string(),
        }
    }
}

impl From<nym_vpn_network_config::NymVpnNetwork> for NymVpnNetwork {
    fn from(value: nym_vpn_network_config::NymVpnNetwork) -> Self {
        NymVpnNetwork {
            nym_vpn_api_url: value.nym_vpn_api_url.to_string(),
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct FeatureFlags {
    // todo: hold as nym_vpn_network_config::FeatureFlags
    pub flags: HashMap<String, FlagValue>,
}

impl FeatureFlags {
    /// If domain fronting is enabled or not, if set
    #[uniffi::method]
    pub fn domain_fronting_enabled(&self) -> Option<bool> {
        // nym_vpn_network_config::FeatureFlags::from(self.clone()).domain_fronting_enabled()
        todo!()
    }

    /// If quic is enabled or not, if set
    #[uniffi::method]
    pub fn quic_enabled(&self) -> Option<bool> {
        // nym_vpn_network_config::FeatureFlags::from(self.clone()).quic_enabled()
        todo!()
    }
}

#[derive(Clone, uniffi::Enum)]
pub enum FlagValue {
    Value(String),
    Group(HashMap<String, String>),
}

impl From<nym_vpn_network_config::FeatureFlags> for FeatureFlags {
    fn from(value: nym_vpn_network_config::FeatureFlags) -> Self {
        FeatureFlags {
            flags: value
                .into_hash_map()
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<nym_vpn_network_config::FlagValue> for FlagValue {
    fn from(value: nym_vpn_network_config::FlagValue) -> Self {
        match value {
            nym_vpn_network_config::FlagValue::Value(v) => FlagValue::Value(v),
            nym_vpn_network_config::FlagValue::Group(g) => FlagValue::Group(g),
        }
    }
}

impl From<nym_vpn_lib_types::FeatureFlags> for FeatureFlags {
    fn from(value: nym_vpn_lib_types::FeatureFlags) -> Self {
        FeatureFlags {
            flags: value
                .flags
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<FeatureFlags> for nym_vpn_lib_types::FeatureFlags {
    fn from(value: FeatureFlags) -> Self {
        nym_vpn_lib_types::FeatureFlags {
            flags: value
                .flags
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<HashMap<_, _>>(),
        }
    }
}

impl From<nym_vpn_lib_types::FlagValue> for FlagValue {
    fn from(value: nym_vpn_lib_types::FlagValue) -> Self {
        match value {
            nym_vpn_lib_types::FlagValue::Value(v) => FlagValue::Value(v),
            nym_vpn_lib_types::FlagValue::Group(g) => FlagValue::Group(g),
        }
    }
}

impl From<FlagValue> for nym_vpn_lib_types::FlagValue {
    fn from(value: FlagValue) -> Self {
        match value {
            FlagValue::Value(v) => nym_vpn_lib_types::FlagValue::Value(v),
            FlagValue::Group(g) => nym_vpn_lib_types::FlagValue::Group(g),
        }
    }
}

#[derive(uniffi::Record, Clone, PartialEq)]
pub struct SystemMessage {
    pub name: String,
    pub message: String,
    pub properties: HashMap<String, String>,
}

impl From<nym_vpn_lib_types::SystemMessage> for SystemMessage {
    fn from(value: nym_vpn_lib_types::SystemMessage) -> Self {
        SystemMessage {
            name: value.name,
            message: value.message,
            properties: value.properties.unwrap_or_default(),
        }
    }
}

impl From<nym_vpn_network_config::SystemMessage> for SystemMessage {
    fn from(value: nym_vpn_network_config::SystemMessage) -> Self {
        SystemMessage {
            name: value.name,
            message: value.message,
            properties: value.properties.map(|v| v.into_inner()).unwrap_or_default(),
        }
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ParsedAccountLinks {
    pub sign_up: String,
    pub sign_in: String,
    pub account: Option<String>,
}

impl From<nym_vpn_network_config::ParsedAccountLinks> for ParsedAccountLinks {
    fn from(value: nym_vpn_network_config::ParsedAccountLinks) -> Self {
        Self {
            sign_up: value.sign_up.to_string(),
            sign_in: value.sign_in.to_string(),
            account: value.account.map(|s| s.to_string()),
        }
    }
}
impl From<nym_vpn_lib_types::ParsedAccountLinks> for ParsedAccountLinks {
    fn from(value: nym_vpn_lib_types::ParsedAccountLinks) -> Self {
        Self {
            sign_up: value.sign_up,
            sign_in: value.sign_in,
            account: value.account,
        }
    }
}
