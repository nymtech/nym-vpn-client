// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_gateway_directory::{EntryPoint as GwEntryPoint, ExitPoint as GwExitPoint};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent as NymUserAgent;
use nym_wg_go::PublicKey;
use time::OffsetDateTime;
use url::Url;

use crate::{platform::error::VpnError, NodeIdentity, Recipient, UniffiCustomTypeConverter};

uniffi::custom_type!(Ipv4Addr, String);
uniffi::custom_type!(Ipv6Addr, String);
uniffi::custom_type!(IpAddr, String);
uniffi::custom_type!(PublicKey, String);
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

impl UniffiCustomTypeConverter for PublicKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(
            PublicKey::from_base64(&val).ok_or_else(|| VpnError::InternalError {
                details: "Invalid public key".to_owned(),
            })?,
        )
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base64()
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

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct MixConnectionInfo {
    pub nym_address: Recipient,
    pub entry_gateway: NodeIdentity,
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct MixExitConnectionInfo {
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ips: IpPair,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct WireguardConnectionInfo {
    pub gateway_id: NodeIdentity,
    pub public_key: String,
    pub private_ipv4: Ipv4Addr,
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

#[derive(uniffi::Enum, Clone, PartialEq)]
pub enum ExitStatus {
    Failure { error: VpnError },
    Stopped,
}

#[derive(uniffi::Enum, Clone, PartialEq)]
pub enum TunStatus {
    Up,
    Down,
    InitializingClient,
    EstablishingConnection,
    Disconnecting,
}

#[derive(uniffi::Enum, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum NymVpnStatus {
    MixConnectInfo {
        mix_connection_info: MixConnectionInfo,
        mix_exit_connection_info: MixExitConnectionInfo,
    },
    WgConnectInfo {
        entry_connection_info: WireguardConnectionInfo,
        exit_connection_info: WireguardConnectionInfo,
    },
}

#[derive(uniffi::Enum, Clone, PartialEq)]
pub enum BandwidthStatus {
    NoBandwidth,
    RemainingBandwidth { bandwidth: i64 },
}

impl From<&BandwidthStatusMessage> for BandwidthStatus {
    fn from(value: &BandwidthStatusMessage) -> Self {
        match value {
            BandwidthStatusMessage::RemainingBandwidth(bandwidth) => {
                BandwidthStatus::RemainingBandwidth {
                    bandwidth: *bandwidth,
                }
            }
            BandwidthStatusMessage::NoBandwidth => BandwidthStatus::NoBandwidth,
        }
    }
}

#[derive(uniffi::Enum, Clone, PartialEq)]
pub enum ConnectionStatus {
    EntryGatewayDown,
    ExitGatewayDownIpv4,
    ExitGatewayDownIpv6,
    ExitGatewayRoutingErrorIpv4,
    ExitGatewayRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
}

impl From<ConnectionMonitorStatus> for ConnectionStatus {
    fn from(value: ConnectionMonitorStatus) -> Self {
        match value {
            ConnectionMonitorStatus::EntryGatewayDown => ConnectionStatus::EntryGatewayDown,
            ConnectionMonitorStatus::ExitGatewayDownIpv4 => ConnectionStatus::ExitGatewayDownIpv4,
            ConnectionMonitorStatus::ExitGatewayDownIpv6 => ConnectionStatus::ExitGatewayDownIpv6,
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv4 => {
                ConnectionStatus::ExitGatewayRoutingErrorIpv4
            }
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv6 => {
                ConnectionStatus::ExitGatewayRoutingErrorIpv6
            }
            ConnectionMonitorStatus::ConnectedIpv4 => ConnectionStatus::ConnectedIpv4,
            ConnectionMonitorStatus::ConnectedIpv6 => ConnectionStatus::ConnectedIpv6,
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
