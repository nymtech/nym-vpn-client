// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::platform::error::FFIError;
use crate::{
    MixnetConnectionInfo, MixnetExitConnectionInfo, NodeIdentity, NymVpnStatusMessage, Recipient,
    UniffiCustomTypeConverter,
};
use ipnetwork::IpNetwork;
use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_explorer_client::Location as ExpLocation;
use nym_gateway_directory::{EntryPoint as GwEntryPoint, ExitPoint as GwExitPoint};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent as NymUserAgent;
use nym_vpn_api_client::Country;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use url::Url;

uniffi::custom_type!(Ipv4Addr, String);
uniffi::custom_type!(Ipv6Addr, String);
uniffi::custom_type!(IpAddr, String);
uniffi::custom_type!(PrivateKey, String);
uniffi::custom_type!(PublicKey, String);
uniffi::custom_type!(IpNetwork, String);
uniffi::custom_type!(SocketAddr, String);
uniffi::custom_type!(PresharedKey, String);
uniffi::custom_type!(Url, String);
uniffi::custom_type!(NodeIdentity, String);
uniffi::custom_type!(Recipient, String);
uniffi::custom_type!(PathBuf, String);

impl UniffiCustomTypeConverter for NodeIdentity {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(NodeIdentity::from_base58_string(val)?)
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

impl UniffiCustomTypeConverter for Url {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Url::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for PrivateKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PrivateKey::from(
            *PublicKey::from_base64(&val)
                .map_err(|_| FFIError::InvalidValueUniffi)?
                .as_bytes(),
        ))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base64()
    }
}

impl UniffiCustomTypeConverter for PublicKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PublicKey::from_base64(&val).map_err(|_| FFIError::InvalidValueUniffi)?)
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
        Ok(serde_json::from_str(&val).map_err(|_| FFIError::InvalidValueUniffi)?)
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

impl UniffiCustomTypeConverter for SocketAddr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(SocketAddr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for PresharedKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PresharedKey::from(Box::new(
            PrivateKey::into_custom(val)?.to_bytes(),
        )))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        PrivateKey::from_custom(PrivateKey::from(*obj.as_bytes()))
    }
}

#[derive(uniffi::Record)]
pub struct Location {
    pub two_letter_iso_country_code: String,
}

impl From<ExpLocation> for Location {
    fn from(value: ExpLocation) -> Self {
        Location {
            two_letter_iso_country_code: value.two_letter_iso_country_code,
        }
    }
}

impl From<nym_gateway_directory::Location> for Location {
    fn from(value: nym_gateway_directory::Location) -> Self {
        Location {
            two_letter_iso_country_code: value.two_letter_iso_country_code,
        }
    }
}

impl From<Country> for Location {
    fn from(value: Country) -> Self {
        Location {
            two_letter_iso_country_code: value.iso_code().to_string(),
        }
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

impl From<MixnetConnectionInfo> for MixConnectionInfo {
    fn from(value: MixnetConnectionInfo) -> Self {
        MixConnectionInfo {
            nym_address: value.nym_address,
            entry_gateway: value.entry_gateway,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct MixExitConnectionInfo {
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ips: IpPair,
}

impl From<MixnetExitConnectionInfo> for MixExitConnectionInfo {
    fn from(value: MixnetExitConnectionInfo) -> Self {
        MixExitConnectionInfo {
            exit_gateway: value.exit_gateway,
            exit_ipr: value.exit_ipr,
            ips: value.ips,
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

#[derive(PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StatusEvent {
    Tun(TunStatus),
    Bandwidth(BandwidthStatus),
    Connection(ConnectionStatus),
    NymVpn(NymVpnStatus),
    Exit(ExitStatus),
}

#[derive(uniffi::Enum, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExitStatus {
    Stopped,
    Failed { error: String },
}

#[derive(uniffi::Enum, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
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
    ConnectionInfo {
        mixnet_connection_info: MixConnectionInfo,
        mixnet_exit_connection_info: MixExitConnectionInfo,
    },
}

impl From<NymVpnStatusMessage> for NymVpnStatus {
    fn from(value: NymVpnStatusMessage) -> Self {
        match value {
            NymVpnStatusMessage::MixnetConnectionInfo {
                mixnet_connection_info,
                mixnet_exit_connection_info,
            } => NymVpnStatus::ConnectionInfo {
                mixnet_connection_info: mixnet_connection_info.into(),
                mixnet_exit_connection_info: mixnet_exit_connection_info.into(),
            },
        }
    }
}

#[derive(uniffi::Enum, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
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
#[allow(clippy::large_enum_variant)]
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
