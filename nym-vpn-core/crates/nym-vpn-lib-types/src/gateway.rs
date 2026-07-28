// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

// The least of evil in terms of dependencies
use nym_crypto::asymmetric::{
    ed25519::{self, Ed25519RecoveryError},
    x25519,
};

use crate::TunnelType;

// Types that duplicate what nym-sdk already offers without pulling the whole nym-sdk into types crate
pub type ClientEncryptionKey = x25519::PublicKey;
pub type ClientIdentity = ed25519::PublicKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeIdentity {
    key: ed25519::PublicKey,
}

impl From<ed25519::PublicKey> for NodeIdentity {
    fn from(key: ed25519::PublicKey) -> Self {
        Self { key }
    }
}

impl NodeIdentity {
    pub fn inner(&self) -> &ed25519::PublicKey {
        &self.key
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Ed25519RecoveryError> {
        ed25519::PublicKey::from_bytes(bytes).map(Self::from)
    }

    pub fn from_base58_string(s: impl AsRef<[u8]>) -> Result<Self, Ed25519RecoveryError> {
        ed25519::PublicKey::from_base58_string(s).map(Self::from)
    }

    pub fn to_base58_string(&self) -> String {
        self.key.to_base58_string()
    }
}

impl FromStr for NodeIdentity {
    type Err = Ed25519RecoveryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_base58_string(s)
    }
}

impl std::fmt::Display for NodeIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base58_string())
    }
}

#[cfg(feature = "serde")]
impl Serialize for NodeIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.key.to_base58_string().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for NodeIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Self::from_base58_string(s).map_err(serde::de::Error::custom)
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum ParseRecipientError {
    #[error("the string address does not contain exactly a single '@' character")]
    InvalidFormat,

    #[error("the string address does not contain exactly a single '.' character")]
    InvalidGatewayHalf,

    #[error("Invalid client identity")]
    InvalidClientIdentity,

    #[error("Invalid client encryption key")]
    InvalidClientEncryptionKey,

    #[error("Invalid gateway key")]
    InvalidGatewayKey,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Recipient {
    client_identity: ClientIdentity,
    client_encryption_key: ClientEncryptionKey,
    gateway: NodeIdentity,
}

impl Recipient {
    pub fn try_from_base58_string<S: Into<String>>(
        full_address: S,
    ) -> Result<Self, ParseRecipientError> {
        let string_address = full_address.into();
        let split: Vec<_> = string_address.split('@').collect();
        if split.len() != 2 {
            return Err(ParseRecipientError::InvalidFormat);
        }
        let client_half = split[0];
        let gateway_half = split[1];

        let split_client: Vec<_> = client_half.split('.').collect();
        if split_client.len() != 2 {
            return Err(ParseRecipientError::InvalidGatewayHalf);
        }

        let client_identity = ClientIdentity::from_base58_string(split_client[0])
            .map_err(|_| ParseRecipientError::InvalidClientIdentity)?;
        let client_encryption_key = ClientEncryptionKey::from_base58_string(split_client[1])
            .map_err(|_| ParseRecipientError::InvalidClientEncryptionKey)?;
        let gateway = NodeIdentity::from_base58_string(gateway_half)
            .map_err(|_| ParseRecipientError::InvalidGatewayKey)?;

        Ok(Recipient {
            client_identity,
            client_encryption_key,
            gateway,
        })
    }

    pub fn gateway(&self) -> &NodeIdentity {
        &self.gateway
    }
}

impl std::str::FromStr for Recipient {
    type Err = ParseRecipientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_base58_string(s)
    }
}

// ADDRESS . ENCRYPTION @ GATEWAY_ID
impl std::fmt::Display for Recipient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}@{}",
            self.client_identity.to_base58_string(),
            self.client_encryption_key.to_base58_string(),
            self.gateway.to_base58_string()
        )
    }
}

impl std::fmt::Debug for Recipient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        // use the Display implementation
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Recipient {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Recipient {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from_base58_string(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "uniffi-bindings")]
pub type BoxedRecepient = Box<Recipient>;
#[cfg(feature = "uniffi-bindings")]
pub type BoxedNodeIdentity = Box<NodeIdentity>;
#[cfg(feature = "uniffi-bindings")]
pub type BoxedGateway = Box<Gateway>;

#[cfg(feature = "uniffi-bindings")]
uniffi::custom_type!(NodeIdentity, String, {
    remote,
    try_lift: |val| Ok(NodeIdentity::from_base58_string(val)?),
    lower: |val| val.to_base58_string()
});

#[cfg(feature = "uniffi-bindings")]
uniffi::custom_type!(BoxedNodeIdentity, String, {
    remote,
    try_lift: |val| Ok(Box::new(NodeIdentity::from_base58_string(val)?)),
    lower: |val| val.to_base58_string()
});

#[cfg(feature = "uniffi-bindings")]
uniffi::custom_type!(Recipient, String, {
    remote,
    try_lift: |val| Ok(Recipient::try_from_base58_string(val)?),
    lower: |val| val.to_string()
});

#[cfg(feature = "uniffi-bindings")]
uniffi::custom_type!(BoxedRecepient, String, {
    remote,
    try_lift: |val| Ok(Box::new(Recipient::try_from_base58_string(val)?)),
    lower: |val| val.to_string()
});

#[cfg(feature = "uniffi-bindings")]
uniffi::custom_type!(BoxedGateway, Gateway, {
    remote,
    try_lift: |val| Ok(Box::new(val)),
    lower: |val| *val
});

#[cfg(feature = "nym-type-conversions")]
impl From<Recipient> for nym_gateway_directory::Recipient {
    fn from(value: Recipient) -> Self {
        Self::new(
            value.client_identity,
            value.client_encryption_key,
            *value.gateway.inner(),
        )
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Recipient> for Recipient {
    fn from(value: nym_gateway_directory::Recipient) -> Self {
        Self {
            client_identity: *value.identity(),
            client_encryption_key: *value.encryption_key(),
            gateway: NodeIdentity::from(value.gateway()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum EntryPoint {
    // An explicit entry gateway identity.
    Gateway {
        #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
        identity: NodeIdentity,
    },
    // Select a random entry gateway in a specific country.
    Country {
        two_letter_iso_country_code: String,
    },
    // Select a random entry gateway in a specific region/state.
    Region {
        region: String,
    },
    // Select an entry gateway at random.
    Random,
}

impl EntryPoint {
    pub fn from_base58_string(identity: &str) -> Result<Self, Ed25519RecoveryError> {
        NodeIdentity::from_base58_string(identity).map(|identity| EntryPoint::Gateway { identity })
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::EntryPoint> for EntryPoint {
    fn from(value: nym_gateway_directory::EntryPoint) -> Self {
        match value {
            nym_gateway_directory::EntryPoint::Gateway { identity } => EntryPoint::Gateway {
                identity: NodeIdentity::from(identity),
            },
            nym_gateway_directory::EntryPoint::Country {
                two_letter_iso_country_code,
            } => EntryPoint::Country {
                two_letter_iso_country_code,
            },
            nym_gateway_directory::EntryPoint::Region { region } => EntryPoint::Region { region },
            nym_gateway_directory::EntryPoint::Random => EntryPoint::Random,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<EntryPoint> for nym_gateway_directory::EntryPoint {
    fn from(value: EntryPoint) -> Self {
        match value {
            EntryPoint::Gateway { identity } => nym_gateway_directory::EntryPoint::Gateway {
                identity: *identity.inner(),
            },
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => nym_gateway_directory::EntryPoint::Country {
                two_letter_iso_country_code,
            },
            EntryPoint::Region { region } => nym_gateway_directory::EntryPoint::Region { region },
            EntryPoint::Random => nym_gateway_directory::EntryPoint::Random,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum ExitPoint {
    // An explicit exit address. This is useful when the exit ip-packet-router is running as a
    // standalone entity (private).
    Address {
        #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
        address: Box<Recipient>,
    },

    // An explicit exit gateway identity. This is useful when the exit ip-packet-router is running
    // embedded on a gateway.
    Gateway {
        #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
        identity: NodeIdentity,
    },

    // Select a random entry gateway in a specific country.
    Country {
        two_letter_iso_country_code: String,
    },

    // Select a random entry gateway in a specific region/state.
    Region {
        region: String,
    },

    // Select an exit gateway at random.
    Random,
}

#[cfg(feature = "nym-type-conversions")]
impl From<ExitPoint> for nym_gateway_directory::ExitPoint {
    fn from(value: ExitPoint) -> Self {
        match value {
            ExitPoint::Address { address } => nym_gateway_directory::ExitPoint::Address {
                address: Box::new(nym_gateway_directory::Recipient::from(*address)),
            },
            ExitPoint::Gateway { identity } => nym_gateway_directory::ExitPoint::Gateway {
                identity: *identity.inner(),
            },
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => nym_gateway_directory::ExitPoint::Country {
                two_letter_iso_country_code,
            },
            ExitPoint::Region { region } => nym_gateway_directory::ExitPoint::Region { region },
            ExitPoint::Random => nym_gateway_directory::ExitPoint::Random,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::ExitPoint> for ExitPoint {
    fn from(value: nym_gateway_directory::ExitPoint) -> Self {
        match value {
            nym_gateway_directory::ExitPoint::Address { address } => ExitPoint::Address {
                address: Box::new(Recipient::from(*address)),
            },
            nym_gateway_directory::ExitPoint::Gateway { identity } => ExitPoint::Gateway {
                identity: NodeIdentity::from(identity),
            },
            nym_gateway_directory::ExitPoint::Country {
                two_letter_iso_country_code,
            } => ExitPoint::Country {
                two_letter_iso_country_code,
            },
            nym_gateway_directory::ExitPoint::Region { region } => ExitPoint::Region { region },
            nym_gateway_directory::ExitPoint::Random => ExitPoint::Random,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum FavoriteSelector {
    // A favorite gateway identity.
    Gateway {
        #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
        identity: NodeIdentity,
    },
    // A favorite country.
    Country {
        two_letter_iso_country_code: String,
    },
    // A favorite region.
    Region {
        region: String,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct FavoriteSelectors {
    pub entry: Vec<FavoriteSelector>,
    pub exit: Vec<FavoriteSelector>,
}

impl std::fmt::Display for FavoriteSelectors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "entry: {:?}", self.entry)?;
        write!(f, "exit: {:?}", self.exit)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum GatewayType {
    MixnetEntry,
    MixnetExit,
    Wg,
}

#[cfg(feature = "nym-type-conversions")]
impl From<GatewayType> for nym_gateway_directory::GatewayType {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MixnetEntry => nym_gateway_directory::GatewayType::MixnetEntry,
            GatewayType::MixnetExit => nym_gateway_directory::GatewayType::MixnetExit,
            GatewayType::Wg => nym_gateway_directory::GatewayType::Wg,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum GatewayFilter {
    MinScore(Score),
    Country(String),
    Region(String),
    Residential,
    QuicEnabled,
    Exit,
    Vpn,
}

#[cfg(feature = "nym-type-conversions")]
impl From<GatewayFilter> for nym_gateway_directory::GatewayFilter {
    fn from(value: GatewayFilter) -> Self {
        match value {
            GatewayFilter::MinScore(score) => nym_gateway_directory::GatewayFilter::MinScore(
                nym_gateway_directory::ScoreValue::from(score),
            ),
            GatewayFilter::Country(country) => {
                nym_gateway_directory::GatewayFilter::Country(country)
            }
            GatewayFilter::Region(region) => nym_gateway_directory::GatewayFilter::Region(region),
            GatewayFilter::Residential => nym_gateway_directory::GatewayFilter::Residential,
            GatewayFilter::QuicEnabled => nym_gateway_directory::GatewayFilter::QuicEnabled,
            GatewayFilter::Exit => nym_gateway_directory::GatewayFilter::Exit,
            GatewayFilter::Vpn => nym_gateway_directory::GatewayFilter::Vpn,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct LookupGatewayFilters {
    pub gw_type: GatewayType,
    pub filters: Vec<GatewayFilter>,
}

#[cfg(feature = "nym-type-conversions")]
impl From<Score> for nym_gateway_directory::ScoreValue {
    fn from(value: Score) -> Self {
        match value {
            Score::Offline => nym_gateway_directory::ScoreValue::Offline,
            Score::Low => nym_gateway_directory::ScoreValue::Low,
            Score::Medium => nym_gateway_directory::ScoreValue::Medium,
            Score::High => nym_gateway_directory::ScoreValue::High,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<LookupGatewayFilters> for nym_gateway_directory::LookupGatewayFilters {
    fn from(value: LookupGatewayFilters) -> Self {
        let LookupGatewayFilters { gw_type, filters } = value;
        let filters: Vec<nym_gateway_directory::GatewayFilter> = filters
            .into_iter()
            .map(nym_gateway_directory::GatewayFilter::from)
            .collect();
        nym_gateway_directory::LookupGatewayFilters {
            gw_type: nym_gateway_directory::GatewayType::from(gw_type),
            filters: nym_gateway_directory::GatewayFilters::from(&filters),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Gateway {
    pub identity_key: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<Location>,
    pub last_probe: Option<Probe>,
    // todo: remove since it's unused?
    pub mixnet_performance: Option<u8>,
    pub bridge_params: Option<BridgeInformation>,
    pub performance: Option<Performance>,
    pub exit_ipv4s: Vec<Ipv4Addr>,
    pub exit_ipv6s: Vec<Ipv6Addr>,
    pub build_version: Option<String>,
    pub lewes_protocol_details: Option<LewesProtocolDetails>,
    pub node_family_name: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum TentativeGateways {
    Selected {
        entry: Box<Gateway>,
        exit: Box<Gateway>,
    },
    NeedsRelaxedIndependenceCriteria,
    NoGatewaysAvailable,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct GetRecentGatewaysParams {
    pub tunnel_type: TunnelType,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct RecentGateways {
    pub entry: Vec<Gateway>,
    pub exit: Vec<Gateway>,
}

pub use nym_bridges_types::ClientConfig as BridgeParameters;
pub use nym_bridges_types::PersistedClientConfig as BridgeInformation;
pub use nym_bridges_types::quic::ClientOptions as QuicClientOptions;
pub use nym_bridges_types::tls::ClientOptions as TlsClientOptions;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Performance {
    pub last_updated_utc: String,
    pub score: Score,
    pub mixnet_score: Score,
    pub load: Score,
    pub uptime_percentage_last_24_hours: f32,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Performance> for Performance {
    fn from(value: nym_gateway_directory::Performance) -> Self {
        Performance {
            last_updated_utc: value.last_updated_utc,
            score: value.score.into(),
            mixnet_score: value.mixnet_score.into(),
            load: value.load.into(),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum AsnKind {
    Residential,
    Other,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Asn {
    pub asn: String,
    pub name: String,
    pub kind: AsnKind,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub region: String,
    pub asn: Option<Asn>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.two_letter_iso_country_code)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "last_updated_utc: {}", self.last_updated_utc)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum Score {
    High,
    Medium,
    Low,
    Offline,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::ScoreValue> for Score {
    fn from(value: nym_gateway_directory::ScoreValue) -> Self {
        match value {
            nym_gateway_directory::ScoreValue::Offline => Score::Offline,
            nym_gateway_directory::ScoreValue::Low => Score::Low,
            nym_gateway_directory::ScoreValue::Medium => Score::Medium,
            nym_gateway_directory::ScoreValue::High => Score::High,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
    pub socks5: Option<Socks5>,
    pub lp: Option<Lp>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Entry {
    pub can_connect: bool,
    pub can_route: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Socks5 {
    pub can_proxy_https: bool,
    pub score: Option<Score>,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Lp {
    pub can_connect: bool,
    pub can_handshake: bool,
    pub can_register: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct LewesProtocolDetails {
    pub content: LewesProtocolDetailsData,
    pub signature: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct LewesProtocolDetailsData {
    pub enabled: bool,
    pub control_port: u16,
    pub data_port: u16,
    pub x25519: String,
    pub kem_keys: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Exit {
    pub can_connect: bool,
    pub can_route_ip_v4: bool,
    pub can_route_ip_external_v4: bool,
    pub can_route_ip_v6: bool,
    pub can_route_ip_external_v6: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Country {
    pub iso_code: String,
}

impl Country {
    pub fn iso_code(&self) -> &str {
        &self.iso_code
    }
}

impl fmt::Display for Gateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let location = self
            .location
            .as_ref()
            .map(|l| l.to_string())
            .unwrap_or("not set".to_string());
        let last_probe = self
            .last_probe
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or("not set".to_string());

        write!(f, "{}, {}, {}", self.identity_key, location, last_probe)
    }
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso_code)
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Country> for Country {
    fn from(country: nym_gateway_directory::Country) -> Self {
        Self {
            iso_code: country.iso_code().to_string(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_validator_client::models::described::v1::NymNodeDescriptionV1> for Gateway {
    fn from(
        node_description: nym_validator_client::models::described::v1::NymNodeDescriptionV1,
    ) -> Self {
        let build_version = Some(node_description.version().to_owned());
        let (exit_ipv4s, exit_ipv6s) = nym_gateway_directory::split_ips(
            node_description.description.host_information.ip_address,
        );
        Self {
            identity_key: node_description
                .description
                .host_information
                .keys
                .ed25519
                .to_string(),
            name: "".to_owned(),
            description: None,
            location: None,
            last_probe: None,
            mixnet_performance: None,
            bridge_params: None,
            performance: None,
            exit_ipv4s,
            exit_ipv6s,
            build_version,
            // v1 has no lp details
            lewes_protocol_details: None,
            node_family_name: None,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_validator_client::models::described::v2::NymNodeDescriptionV2> for Gateway {
    fn from(
        node_description: nym_validator_client::models::described::v2::NymNodeDescriptionV2,
    ) -> Self {
        let build_version = Some(node_description.version().to_owned());
        let (exit_ipv4s, exit_ipv6s) = nym_gateway_directory::split_ips(
            node_description.description.host_information.ip_address,
        );
        Self {
            identity_key: node_description
                .description
                .host_information
                .keys
                .ed25519
                .to_string(),
            name: "".to_owned(),
            description: None,
            location: None,
            last_probe: None,
            mixnet_performance: None,
            bridge_params: None,
            performance: None,
            exit_ipv4s,
            exit_ipv6s,
            build_version,
            lewes_protocol_details: node_description.description.lewes_protocol.map(Into::into),
            node_family_name: None,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::AsnKind> for AsnKind {
    fn from(value: nym_gateway_directory::AsnKind) -> Self {
        match value {
            nym_gateway_directory::AsnKind::Residential => AsnKind::Residential,
            nym_gateway_directory::AsnKind::Other => AsnKind::Other,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Asn> for Asn {
    fn from(value: nym_gateway_directory::Asn) -> Self {
        Asn {
            asn: value.asn,
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Location> for Location {
    fn from(location: nym_gateway_directory::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(Into::into),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Entry> for Entry {
    fn from(entry: nym_gateway_directory::Entry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Socks5> for Socks5 {
    fn from(socks5: nym_gateway_directory::Socks5) -> Self {
        Self {
            can_proxy_https: socks5.can_proxy_https,
            score: socks5.score.map(Score::from),
            errors: socks5.errors,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Lp> for Lp {
    fn from(lp: nym_gateway_directory::Lp) -> Self {
        Self {
            can_connect: lp.can_connect,
            can_handshake: lp.can_handshake,
            can_register: lp.can_register,
            error: lp.error,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_validator_client::models::described::type_translation::LewesProtocolDetailsV1>
    for LewesProtocolDetails
{
    fn from(
        value: nym_validator_client::models::described::type_translation::LewesProtocolDetailsV1,
    ) -> Self {
        Self {
            content: value.content.into(),
            signature: value.signature.to_base58_string(),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_validator_client::models::described::type_translation::LewesProtocolDetailsDataV1>
    for LewesProtocolDetailsData
{
    fn from(
        value: nym_validator_client::models::described::type_translation::LewesProtocolDetailsDataV1,
    ) -> Self {
        let kem_keys = value
            .kem_keys
            .into_iter()
            .map(|(kem, digests)| {
                let digests_str: HashMap<String, String> = digests
                    .into_iter()
                    .map(|(hash_fn, value)| (hash_fn.to_string(), value))
                    .collect();

                (kem.to_string(), digests_str)
            })
            .collect();
        Self {
            enabled: value.enabled,
            control_port: value.control_port,
            data_port: value.data_port,
            x25519: x25519::PublicKey::from(value.x25519).to_base58_string(),
            kem_keys,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Exit> for Exit {
    fn from(exit: nym_gateway_directory::Exit) -> Self {
        Self {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::ProbeOutcome> for ProbeOutcome {
    fn from(outcome: nym_gateway_directory::ProbeOutcome) -> Self {
        Self {
            as_entry: Entry::from(outcome.as_entry),
            as_exit: outcome.as_exit.map(Exit::from),
            socks5: outcome.socks5.map(Socks5::from),
            lp: outcome.lp.map(Lp::from),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Probe> for Probe {
    fn from(probe: nym_gateway_directory::Probe) -> Self {
        Self {
            last_updated_utc: probe.last_updated_utc,
            outcome: ProbeOutcome::from(probe.outcome),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_gateway_directory::Gateway> for Gateway {
    fn from(gateway: nym_gateway_directory::Gateway) -> Self {
        let (exit_ipv4s, exit_ipv6s) = gateway.split_ips();

        Self {
            identity_key: gateway.identity.to_string(),
            name: gateway.name,
            description: gateway.description,
            location: gateway.location.map(Location::from),
            last_probe: gateway.last_probe.map(Probe::from),
            mixnet_performance: gateway.mixnet_performance.map(|p| p.round_to_integer()),
            bridge_params: gateway.bridge_params,
            performance: gateway.performance.map(Performance::from),
            exit_ipv4s,
            exit_ipv6s,
            build_version: gateway.version,
            lewes_protocol_details: gateway
                .lewes_protocol_details
                .map(LewesProtocolDetails::from),
            node_family_name: gateway.family_data.map(|family| family.name),
        }
    }
}
