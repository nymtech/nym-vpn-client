// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_gateway_directory::{EntryPoint as GwEntryPoint, ExitPoint as GwExitPoint};
use nym_ip_packet_requests::IpPair;
use nym_sdk::{
    UserAgent as NymUserAgent,
    mixnet::{NodeIdentity, Recipient},
};
use time::OffsetDateTime;
use url::Url;

uniffi::custom_type!(Ipv4Addr, String, {
    remote,
    try_lift: |val| Ok(Ipv4Addr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Ipv6Addr, String, {
    remote,
    try_lift: |val| Ok(Ipv6Addr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(IpAddr, String, {
    remote,
    try_lift: |val| Ok(IpAddr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(SocketAddr, String, {
    remote,
    try_lift: |val| Ok(SocketAddr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(PathBuf, String, {
    remote,
    try_lift: |val| Ok(PathBuf::from(val)),
    lower: |val| val.display().to_string()
});

uniffi::custom_type!(IpNetwork, String, {
    remote,
    try_lift: |val| Ok(IpNetwork::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Ipv4Network, String, {
    remote,
    try_lift: |val| Ok(Ipv4Network::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Ipv6Network, String, {
    remote,
    try_lift: |val| Ok(Ipv6Network::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Url, String, {
    remote,
    try_lift: |val| Ok(Url::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(OffsetDateTime, i64, {
    remote,
    try_lift: |val| Ok(OffsetDateTime::from_unix_timestamp(val)?),
    lower: |val| val.unix_timestamp()
});

pub type BoxedRecepient = Box<Recipient>;
pub type BoxedNodeIdentity = Box<NodeIdentity>;

uniffi::custom_type!(NodeIdentity, String, {
    remote,
    try_lift: |val| Ok(NodeIdentity::from_base58_string(val)?),
    lower: |val| val.to_base58_string()
});

uniffi::custom_type!(BoxedNodeIdentity, String, {
    remote,
    try_lift: |val| Ok(Box::new(NodeIdentity::from_base58_string(val)?)),
    lower: |val| val.to_base58_string()
});

uniffi::custom_type!(Recipient, String, {
    remote,
    try_lift: |val| Ok(Recipient::try_from_base58_string(val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(BoxedRecepient, String, {
    remote,
    try_lift: |val| Ok(Box::new(Recipient::try_from_base58_string(val)?)),
    lower: |val| val.to_string()
});

uniffi::custom_type!(
    IpPair,
    Vec<u8>, {
        remote,
        try_lift: |val| {
            let bytes: [u8; 20] = val.try_into().expect("Invalid length for IpPair byte representation");
            let ipv4_bytes: [u8; 4] = bytes[0..4].try_into().unwrap();
            let ipv6_bytes: [u8; 16] = bytes[4..20].try_into().unwrap();

            Ok(IpPair {
                ipv4: Ipv4Addr::from(ipv4_bytes),
                ipv6: Ipv6Addr::from(ipv6_bytes)
            })
        },
        lower: |val| {
            val.ipv4.octets().into_iter()
                .chain(val.ipv6.octets().into_iter())
                .collect()
        }
    }
);

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum UniffiConversionError {
    #[error("invalid byte length")]
    InvalidByteLength,

    #[error("invalid mixnet min performance percentage")]
    InvalidMixnetMinPerformancePercentage,

    #[error("invalid vpn min performance percentage")]
    InvalidVpnMinPerformancePercentage,
}

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

impl From<nym_vpn_network_config::Network> for NetworkEnvironment {
    fn from(network: nym_vpn_network_config::Network) -> Self {
        NetworkEnvironment {
            nym_network: network.nym_network.network.into(),
            nym_vpn_network: network.nym_vpn_network.into(),
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

impl From<nym_config::defaults::NymNetworkDetails> for NymNetworkDetails {
    fn from(value: nym_config::defaults::NymNetworkDetails) -> Self {
        NymNetworkDetails {
            network_name: value.network_name,
            chain_details: value.chain_details.into(),
            endpoints: value.endpoints.into_iter().map(|e| e.into()).collect(),
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
}

impl From<nym_vpn_network_config::NymVpnNetwork> for NymVpnNetwork {
    fn from(value: nym_vpn_network_config::NymVpnNetwork) -> Self {
        NymVpnNetwork {
            nym_vpn_api_url: value.nym_vpn_api_url.to_string(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct FeatureFlags {
    pub flags: HashMap<String, FlagValue>,
}

#[derive(uniffi::Enum)]
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

impl From<nym_vpn_network_config::feature_flags::FlagValue> for FlagValue {
    fn from(value: nym_vpn_network_config::feature_flags::FlagValue) -> Self {
        match value {
            nym_vpn_network_config::feature_flags::FlagValue::Value(v) => FlagValue::Value(v),
            nym_vpn_network_config::feature_flags::FlagValue::Group(g) => FlagValue::Group(g),
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Enum, Clone)]
pub enum Score {
    High,
    Medium,
    Low,
    None,
}

impl From<nym_gateway_directory::Score> for Score {
    fn from(value: nym_gateway_directory::Score) -> Self {
        match value {
            nym_gateway_directory::Score::High(_) => Score::High,
            nym_gateway_directory::Score::Medium(_) => Score::Medium,
            nym_gateway_directory::Score::Low(_) => Score::Low,
            nym_gateway_directory::Score::None => Score::None,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct GatewayInfo {
    pub id: NodeIdentity,
    pub moniker: String,
    pub location: Option<Location>,
    pub mixnet_score: Option<Score>,
    pub wg_score: Option<Score>,
}

impl From<nym_gateway_directory::Gateway> for GatewayInfo {
    fn from(value: nym_gateway_directory::Gateway) -> Self {
        GatewayInfo {
            moniker: value.moniker,
            location: value.location.map(Location::from),
            id: value.identity,
            mixnet_score: value.mixnet_score.map(Score::from),
            wg_score: value.wg_score.map(Score::from),
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct Location {
    pub two_letter_iso_country_code: String,
}

impl From<nym_gateway_directory::Location> for Location {
    fn from(value: nym_gateway_directory::Location) -> Self {
        Location {
            two_letter_iso_country_code: value.two_letter_iso_country_code,
        }
    }
}

impl From<nym_gateway_directory::Country> for Location {
    fn from(value: nym_gateway_directory::Country) -> Self {
        Location {
            two_letter_iso_country_code: value.iso_code().to_string(),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum GatewayType {
    MixnetEntry,
    MixnetExit,
    Wg,
}

impl From<GatewayType> for nym_gateway_directory::GatewayType {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MixnetEntry => nym_gateway_directory::GatewayType::MixnetEntry,
            GatewayType::MixnetExit => nym_gateway_directory::GatewayType::MixnetExit,
            GatewayType::Wg => nym_gateway_directory::GatewayType::Wg,
        }
    }
}

#[derive(uniffi::Record)]
pub struct GatewayMinPerformance {
    mixnet_min_performance: Option<u64>,
    vpn_min_performance: Option<u64>,
}

impl TryFrom<GatewayMinPerformance> for nym_gateway_directory::GatewayMinPerformance {
    type Error = UniffiConversionError;

    fn try_from(value: GatewayMinPerformance) -> Result<Self, Self::Error> {
        let mixnet_min_performance = value
            .mixnet_min_performance
            .map(|p| {
                nym_gateway_directory::Percent::from_percentage_value(p)
                    .map_err(|_| UniffiConversionError::InvalidMixnetMinPerformancePercentage)
            })
            .transpose()?;
        let vpn_min_performance = value
            .vpn_min_performance
            .map(|p| {
                nym_gateway_directory::Percent::from_percentage_value(p)
                    .map_err(|_| UniffiConversionError::InvalidVpnMinPerformancePercentage)
            })
            .transpose()?;
        Ok(nym_gateway_directory::GatewayMinPerformance {
            mixnet_min_performance,
            vpn_min_performance,
        })
    }
}

#[derive(uniffi::Record, Clone)]
pub struct UserAgent {
    // The name of the application
    // Example: nym-vpnd
    pub application: String,

    // The version
    pub version: String,

    // The platform triple
    // Example: x86_64-unknown-linux-gnu
    pub platform: String,

    // The git commit hash
    pub git_commit: String,
}

impl From<UserAgent> for NymUserAgent {
    fn from(value: UserAgent) -> Self {
        NymUserAgent {
            application: value.application,
            version: value.version,
            platform: value.platform,
            git_commit: value.git_commit,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum EntryPoint {
    Gateway { identity: NodeIdentity },
    Location { location: String },
    Random,
}

impl From<EntryPoint> for GwEntryPoint {
    fn from(value: EntryPoint) -> Self {
        match value {
            EntryPoint::Gateway { identity } => GwEntryPoint::Gateway { identity },
            EntryPoint::Location { location } => GwEntryPoint::Location { location },
            EntryPoint::Random => GwEntryPoint::Random,
        }
    }
}

#[derive(uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum ExitPoint {
    Address { address: Recipient },
    Gateway { identity: NodeIdentity },
    Location { location: String },
}

impl From<ExitPoint> for GwExitPoint {
    fn from(value: ExitPoint) -> Self {
        match value {
            ExitPoint::Address { address } => GwExitPoint::Address {
                address: Box::new(address),
            },
            ExitPoint::Gateway { identity } => GwExitPoint::Gateway { identity },
            ExitPoint::Location { location } => GwExitPoint::Location { location },
        }
    }
}

#[derive(uniffi::Record, Clone, Default, PartialEq)]
pub struct RegisterAccountResponse {
    pub account_token: String,
}

impl From<nym_vpn_account_controller::RegisterAccountResponse> for RegisterAccountResponse {
    fn from(value: nym_vpn_account_controller::RegisterAccountResponse) -> Self {
        RegisterAccountResponse {
            account_token: value.account_token,
        }
    }
}

#[derive(uniffi::Record, Clone, PartialEq)]
pub struct SystemMessage {
    pub name: String,
    pub message: String,
    pub properties: HashMap<String, String>,
}

impl From<nym_vpn_network_config::SystemMessage> for SystemMessage {
    fn from(value: nym_vpn_network_config::SystemMessage) -> Self {
        SystemMessage {
            name: value.name,
            message: value.message,
            properties: value.properties.into_inner(),
        }
    }
}

#[derive(uniffi::Record, Clone, PartialEq)]
pub struct NetworkCompatibility {
    pub core: String,
    pub ios: String,
    pub macos: String,
    pub tauri: String,
    pub android: String,
}

impl From<nym_vpn_api_client::NetworkCompatibility> for NetworkCompatibility {
    fn from(value: nym_vpn_api_client::NetworkCompatibility) -> Self {
        NetworkCompatibility {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

#[derive(uniffi::Record, Clone, PartialEq)]
pub struct AccountLinks {
    pub sign_up: String,
    pub sign_in: String,
    pub account: Option<String>,
}

impl From<nym_vpn_network_config::ParsedAccountLinks> for AccountLinks {
    fn from(value: nym_vpn_network_config::ParsedAccountLinks) -> Self {
        AccountLinks {
            sign_up: value.sign_up.to_string(),
            sign_in: value.sign_in.to_string(),
            account: value.account.map(|s| s.to_string()),
        }
    }
}
