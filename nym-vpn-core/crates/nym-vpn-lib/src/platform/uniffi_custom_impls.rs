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
use nym_sdk::UserAgent as NymUserAgent;
use time::OffsetDateTime;
use url::Url;

use crate::{
    platform::error::{RequestZkNymError, RequestZkNymSuccess, VpnError},
    NodeIdentity, Recipient, UniffiCustomTypeConverter,
};

uniffi::custom_type!(Ipv4Addr, String);
uniffi::custom_type!(Ipv6Addr, String);
uniffi::custom_type!(IpAddr, String);
uniffi::custom_type!(IpNetwork, String);
uniffi::custom_type!(Ipv4Network, String);
uniffi::custom_type!(Ipv6Network, String);
uniffi::custom_type!(SocketAddr, String);
uniffi::custom_type!(Url, String);
uniffi::custom_type!(NodeIdentity, String);
uniffi::custom_type!(Recipient, String);
uniffi::custom_type!(PathBuf, String);
uniffi::custom_type!(OffsetDateTime, i64);

pub type BoxedRecepient = Box<Recipient>;
pub type BoxedNodeIdentity = Box<NodeIdentity>;
uniffi::custom_type!(BoxedRecepient, String);
uniffi::custom_type!(BoxedNodeIdentity, String);

impl UniffiCustomTypeConverter for NodeIdentity {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(NodeIdentity::from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base58_string()
    }
}

impl UniffiCustomTypeConverter for BoxedNodeIdentity {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Box::new(NodeIdentity::from_base58_string(val)?))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base58_string()
    }
}

impl UniffiCustomTypeConverter for Recipient {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Recipient::try_from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl crate::UniffiCustomTypeConverter for BoxedRecepient {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Box::new(Recipient::try_from_base58_string(val)?))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Url {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Url::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for IpAddr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(IpAddr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

uniffi::custom_type!(IpPair, String);
impl UniffiCustomTypeConverter for IpPair {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(
            serde_json::from_str(&val).map_err(|e| VpnError::InternalError {
                details: e.to_string(),
            })?,
        )
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        serde_json::to_string(&obj).expect("Failed to serialize ip pair")
    }
}

impl UniffiCustomTypeConverter for Ipv4Addr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv4Addr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv6Addr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv6Addr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for IpNetwork {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(IpNetwork::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv4Network {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv4Network::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv6Network {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv6Network::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for SocketAddr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(SocketAddr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for OffsetDateTime {
    type Builtin = i64;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(OffsetDateTime::from_unix_timestamp(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.unix_timestamp()
    }
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

impl From<crate::nym_config::defaults::NymNetworkDetails> for NymNetworkDetails {
    fn from(value: crate::nym_config::defaults::NymNetworkDetails) -> Self {
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

impl From<crate::nym_config::defaults::ChainDetails> for ChainDetails {
    fn from(value: crate::nym_config::defaults::ChainDetails) -> Self {
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

impl From<crate::nym_config::defaults::DenomDetailsOwned> for DenomDetails {
    fn from(value: crate::nym_config::defaults::DenomDetailsOwned) -> Self {
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

impl From<crate::nym_config::defaults::ValidatorDetails> for ValidatorDetails {
    fn from(value: crate::nym_config::defaults::ValidatorDetails) -> Self {
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

impl From<crate::nym_config::defaults::NymContracts> for NymContracts {
    fn from(value: crate::nym_config::defaults::NymContracts) -> Self {
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
                .flags
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

#[derive(uniffi::Record)]
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
    type Error = VpnError;

    fn try_from(value: GatewayMinPerformance) -> Result<Self, Self::Error> {
        let mixnet_min_performance = value
            .mixnet_min_performance
            .map(|p| {
                nym_gateway_directory::Percent::from_percentage_value(p).map_err(|_| {
                    VpnError::InternalError {
                        details: "Invalid mixnet min performance percentage".to_string(),
                    }
                })
            })
            .transpose()?;
        let vpn_min_performance = value
            .vpn_min_performance
            .map(|p| {
                nym_gateway_directory::Percent::from_percentage_value(p).map_err(|_| {
                    VpnError::InternalError {
                        details: "Invalid vpn min performance percentage".to_string(),
                    }
                })
            })
            .transpose()?;
        Ok(nym_gateway_directory::GatewayMinPerformance {
            mixnet_min_performance,
            vpn_min_performance,
        })
    }
}

#[derive(uniffi::Record)]
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
    RandomLowLatency,
    Random,
}

impl From<EntryPoint> for GwEntryPoint {
    fn from(value: EntryPoint) -> Self {
        match value {
            EntryPoint::Gateway { identity } => GwEntryPoint::Gateway { identity },
            EntryPoint::Location { location } => GwEntryPoint::Location { location },
            EntryPoint::RandomLowLatency => GwEntryPoint::RandomLowLatency,
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
            ExitPoint::Address { address } => GwExitPoint::Address { address },
            ExitPoint::Gateway { identity } => GwExitPoint::Gateway { identity },
            ExitPoint::Location { location } => GwExitPoint::Location { location },
        }
    }
}

impl UniffiCustomTypeConverter for PathBuf {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PathBuf::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.display().to_string()
    }
}

#[derive(uniffi::Record, Clone, Default, PartialEq)]
pub struct AccountStateSummary {
    pub mnemonic: Option<MnemonicState>,
    pub account_registered: Option<AccountRegistered>,
    pub account_summary: Option<AccountSummary>,
    pub device: Option<DeviceState>,
    pub register_device_result: Option<RegisterDeviceResult>,
    pub request_zk_nym_result: Option<RequestZkNymResult>,
}

impl From<nym_vpn_account_controller::AccountStateSummary> for AccountStateSummary {
    fn from(value: nym_vpn_account_controller::AccountStateSummary) -> Self {
        AccountStateSummary {
            mnemonic: value.mnemonic.map(|m| m.into()),
            account_registered: value.account_registered.map(|a| a.into()),
            account_summary: value.account_summary.map(|a| a.into()),
            device: value.device.map(|d| d.into()),
            register_device_result: value.register_device_result.map(|r| r.into()),
            request_zk_nym_result: value.request_zk_nym_result.map(|r| r.into()),
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum MnemonicState {
    NotStored,
    Stored,
}

impl From<nym_vpn_account_controller::shared_state::MnemonicState> for MnemonicState {
    fn from(value: nym_vpn_account_controller::shared_state::MnemonicState) -> Self {
        match value {
            nym_vpn_account_controller::shared_state::MnemonicState::NotStored => {
                MnemonicState::NotStored
            }
            nym_vpn_account_controller::shared_state::MnemonicState::Stored { .. } => {
                MnemonicState::Stored
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountRegistered {
    Registered,
    NotRegistered,
}

impl From<nym_vpn_account_controller::shared_state::AccountRegistered> for AccountRegistered {
    fn from(value: nym_vpn_account_controller::shared_state::AccountRegistered) -> Self {
        match value {
            nym_vpn_account_controller::shared_state::AccountRegistered::Registered { .. } => {
                AccountRegistered::Registered
            }
            nym_vpn_account_controller::shared_state::AccountRegistered::NotRegistered => {
                AccountRegistered::NotRegistered
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountState {
    Inactive,
    Active,
    DeleteMe,
}

impl From<nym_vpn_account_controller::shared_state::AccountState> for AccountState {
    fn from(value: nym_vpn_account_controller::shared_state::AccountState) -> Self {
        match value {
            nym_vpn_account_controller::shared_state::AccountState::Inactive { .. } => {
                AccountState::Inactive
            }
            nym_vpn_account_controller::shared_state::AccountState::Active { .. } => {
                AccountState::Active
            }
            nym_vpn_account_controller::shared_state::AccountState::DeleteMe { .. } => {
                AccountState::DeleteMe
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum SubscriptionState {
    NotActive,
    Pending,
    Complete,
    Active,
}

impl From<nym_vpn_account_controller::shared_state::SubscriptionState> for SubscriptionState {
    fn from(value: nym_vpn_account_controller::shared_state::SubscriptionState) -> Self {
        match value {
            nym_vpn_account_controller::shared_state::SubscriptionState::NotActive => {
                SubscriptionState::NotActive
            }
            nym_vpn_account_controller::shared_state::SubscriptionState::Pending => {
                SubscriptionState::Pending
            }
            nym_vpn_account_controller::shared_state::SubscriptionState::Complete => {
                SubscriptionState::Complete
            }
            nym_vpn_account_controller::shared_state::SubscriptionState::Active => {
                SubscriptionState::Active
            }
        }
    }
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct DeviceSummary {
    active: u64,
    max: u64,
    remaining: u64,
}

impl From<nym_vpn_account_controller::shared_state::DeviceSummary> for DeviceSummary {
    fn from(value: nym_vpn_account_controller::shared_state::DeviceSummary) -> Self {
        DeviceSummary {
            active: value.active,
            max: value.max,
            remaining: value.remaining,
        }
    }
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct FairUsage {
    pub used_gb: Option<f64>,
    pub limit_gb: Option<f64>,
    pub resets_on_utc: Option<String>,
}

impl From<nym_vpn_account_controller::shared_state::FairUsage> for FairUsage {
    fn from(value: nym_vpn_account_controller::shared_state::FairUsage) -> Self {
        FairUsage {
            used_gb: value.used_gb,
            limit_gb: value.limit_gb,
            resets_on_utc: value.resets_on_utc,
        }
    }
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct AccountSummary {
    account: AccountState,
    subscription: SubscriptionState,
    device_summary: DeviceSummary,
    fair_usage: FairUsage,
}

impl From<nym_vpn_account_controller::shared_state::AccountSummary> for AccountSummary {
    fn from(value: nym_vpn_account_controller::shared_state::AccountSummary) -> Self {
        AccountSummary {
            account: value.account.into(),
            subscription: value.subscription.into(),
            device_summary: value.device_summary.into(),
            fair_usage: value.fair_usage.into(),
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum DeviceState {
    NotRegistered,
    Inactive,
    Active,
    DeleteMe,
}

impl From<nym_vpn_account_controller::shared_state::DeviceState> for DeviceState {
    fn from(value: nym_vpn_account_controller::shared_state::DeviceState) -> Self {
        match value {
            nym_vpn_account_controller::shared_state::DeviceState::NotRegistered => {
                DeviceState::NotRegistered
            }
            nym_vpn_account_controller::shared_state::DeviceState::Inactive => {
                DeviceState::Inactive
            }
            nym_vpn_account_controller::shared_state::DeviceState::Active => DeviceState::Active,
            nym_vpn_account_controller::shared_state::DeviceState::DeleteMe => {
                DeviceState::DeleteMe
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum RegisterDeviceResult {
    InProgress,
    Success,
    Failed {
        message: String,
        message_id: Option<String>,
    },
}

impl From<nym_vpn_account_controller::shared_state::RegisterDeviceResult> for RegisterDeviceResult {
    fn from(result: nym_vpn_account_controller::shared_state::RegisterDeviceResult) -> Self {
        match result {
            nym_vpn_account_controller::shared_state::RegisterDeviceResult::InProgress => {
                RegisterDeviceResult::InProgress
            }
            nym_vpn_account_controller::shared_state::RegisterDeviceResult::Success => {
                RegisterDeviceResult::Success
            }
            nym_vpn_account_controller::shared_state::RegisterDeviceResult::Failed(err) => {
                RegisterDeviceResult::Failed {
                    message: err.message(),
                    message_id: err.message_id(),
                }
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum RequestZkNymResult {
    InProgress,
    Done {
        successes: Vec<RequestZkNymSuccess>,
        failures: Vec<RequestZkNymError>,
    },
    Error(RequestZkNymError),
}

impl From<nym_vpn_account_controller::shared_state::RequestZkNymResult> for RequestZkNymResult {
    fn from(result: nym_vpn_account_controller::shared_state::RequestZkNymResult) -> Self {
        match result {
            nym_vpn_account_controller::shared_state::RequestZkNymResult::InProgress => {
                RequestZkNymResult::InProgress
            }
            nym_vpn_account_controller::shared_state::RequestZkNymResult::Done {
                successes,
                failures,
            } => RequestZkNymResult::Done {
                successes: successes.into_iter().map(|s| s.into()).collect(),
                failures: failures.into_iter().map(|f| f.into()).collect(),
            },
            nym_vpn_account_controller::shared_state::RequestZkNymResult::Error(e) => {
                RequestZkNymResult::Error(e.into())
            }
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
