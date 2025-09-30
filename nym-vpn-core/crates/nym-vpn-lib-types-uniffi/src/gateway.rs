// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use nym_vpn_lib_types::{NodeIdentity, Recipient};

use super::error::UniffiConversionError;

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

#[derive(uniffi::Enum)]
pub enum EntryPoint {
    Gateway { identity: NodeIdentity },
    Country { two_letter_iso_country_code: String },
    Region { region: String },
    Random,
}

impl From<EntryPoint> for nym_vpn_lib_types::EntryPoint {
    fn from(value: EntryPoint) -> Self {
        match value {
            EntryPoint::Gateway { identity } => nym_vpn_lib_types::EntryPoint::Gateway { identity },
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => nym_vpn_lib_types::EntryPoint::Country {
                two_letter_iso_country_code,
            },
            EntryPoint::Region { region } => nym_vpn_lib_types::EntryPoint::Region { region },
            EntryPoint::Random => nym_vpn_lib_types::EntryPoint::Random,
        }
    }
}

impl From<nym_vpn_lib_types::EntryPoint> for EntryPoint {
    fn from(value: nym_vpn_lib_types::EntryPoint) -> Self {
        match value {
            nym_vpn_lib_types::EntryPoint::Gateway { identity } => EntryPoint::Gateway { identity },
            nym_vpn_lib_types::EntryPoint::Country {
                two_letter_iso_country_code,
            } => EntryPoint::Country {
                two_letter_iso_country_code,
            },
            nym_vpn_lib_types::EntryPoint::Region { region } => EntryPoint::Region { region },
            nym_vpn_lib_types::EntryPoint::Random => EntryPoint::Random,
        }
    }
}

#[derive(uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum ExitPoint {
    Address { address: Recipient },
    Gateway { identity: NodeIdentity },
    Country { two_letter_iso_country_code: String },
    Region { region: String },
    Random,
}

impl From<ExitPoint> for nym_vpn_lib_types::ExitPoint {
    fn from(value: ExitPoint) -> Self {
        match value {
            ExitPoint::Address { address } => nym_vpn_lib_types::ExitPoint::Address {
                address: Box::new(address),
            },
            ExitPoint::Gateway { identity } => nym_vpn_lib_types::ExitPoint::Gateway { identity },
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => nym_vpn_lib_types::ExitPoint::Country {
                two_letter_iso_country_code,
            },
            ExitPoint::Region { region } => nym_vpn_lib_types::ExitPoint::Region { region },
            ExitPoint::Random => nym_vpn_lib_types::ExitPoint::Random,
        }
    }
}

impl From<nym_vpn_lib_types::ExitPoint> for ExitPoint {
    fn from(value: nym_vpn_lib_types::ExitPoint) -> Self {
        match value {
            nym_vpn_lib_types::ExitPoint::Address { address } => {
                ExitPoint::Address { address: *address }
            }
            nym_vpn_lib_types::ExitPoint::Gateway { identity } => ExitPoint::Gateway { identity },
            nym_vpn_lib_types::ExitPoint::Country {
                two_letter_iso_country_code,
            } => ExitPoint::Country {
                two_letter_iso_country_code,
            },
            nym_vpn_lib_types::ExitPoint::Region { region } => ExitPoint::Region { region },
            nym_vpn_lib_types::ExitPoint::Random => ExitPoint::Random,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Enum, Clone)]
pub enum Score {
    High,
    Medium,
    Low,
    Offline,
}

impl From<nym_vpn_lib_types::Score> for Score {
    fn from(value: nym_vpn_lib_types::Score) -> Self {
        match value {
            nym_vpn_lib_types::Score::High => Score::High,
            nym_vpn_lib_types::Score::Medium => Score::Medium,
            nym_vpn_lib_types::Score::Low => Score::Low,
            nym_vpn_lib_types::Score::Offline => Score::Offline,
        }
    }
}

impl From<nym_gateway_directory::Score> for Score {
    fn from(value: nym_gateway_directory::Score) -> Self {
        match value {
            nym_gateway_directory::Score::High(_) => Score::High,
            nym_gateway_directory::Score::Medium(_) => Score::Medium,
            nym_gateway_directory::Score::Low(_) => Score::Low,
            nym_gateway_directory::Score::None => Score::Offline,
        }
    }
}

impl From<nym_gateway_directory::ScoreValue> for Score {
    fn from(value: nym_gateway_directory::ScoreValue) -> Self {
        match value {
            nym_gateway_directory::ScoreValue::High => Score::High,
            nym_gateway_directory::ScoreValue::Medium => Score::Medium,
            nym_gateway_directory::ScoreValue::Low => Score::Low,
            nym_gateway_directory::ScoreValue::Offline => Score::Offline,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct Performance {
    pub last_updated_utc: String,
    pub score: Score,
    pub load: Score,
    pub uptime_percentage_last_24_hours: f32,
}

impl From<nym_vpn_lib_types::Performance> for Performance {
    fn from(value: nym_vpn_lib_types::Performance) -> Self {
        Performance {
            last_updated_utc: value.last_updated_utc,
            score: Score::from(value.score),
            load: Score::from(value.load),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
        }
    }
}

impl From<nym_gateway_directory::Performance> for Performance {
    fn from(value: nym_gateway_directory::Performance) -> Self {
        Performance {
            last_updated_utc: value.last_updated_utc,
            score: Score::from(value.score),
            load: Score::from(value.load),
            uptime_percentage_last_24_hours: value.uptime_percentage_last_24_hours,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct Gateway {
    pub id: String,
    pub moniker: String,
    pub location: Option<Location>,
    pub mixnet_score: Option<Score>,
    pub wg_performance: Option<Performance>,
    pub exit_ipv4s: Vec<Ipv4Addr>,
    pub exit_ipv6s: Vec<Ipv6Addr>,
    pub build_version: Option<String>,
}

impl From<nym_vpn_lib_types::Gateway> for Gateway {
    fn from(value: nym_vpn_lib_types::Gateway) -> Self {
        Gateway {
            moniker: value.moniker,
            location: value.location.map(Location::from),
            id: value.identity_key,
            mixnet_score: value.mixnet_score.map(Score::from),
            wg_performance: value.wg_performance.map(Performance::from),
            exit_ipv4s: value.exit_ipv4s,
            exit_ipv6s: value.exit_ipv6s,
            build_version: value.build_version,
        }
    }
}

impl From<nym_gateway_directory::Gateway> for Gateway {
    fn from(value: nym_gateway_directory::Gateway) -> Self {
        let mut ipv4_ips = vec![];
        let mut ipv6_ips = vec![];

        for ip in value.ips {
            match ip {
                IpAddr::V4(ip) => ipv4_ips.push(ip),
                IpAddr::V6(ip) => ipv6_ips.push(ip),
            }
        }
        Gateway {
            moniker: value.moniker,
            location: value.location.map(Location::from),
            id: value.identity.to_base58_string(),
            mixnet_score: value.mixnet_score.map(Score::from),
            wg_performance: value.wg_performance.map(Performance::from),
            exit_ipv4s: ipv4_ips,
            exit_ipv6s: ipv6_ips,
            build_version: value.version,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Enum, Clone)]
pub enum AsnKind {
    Residential,
    Other,
}

impl From<nym_gateway_directory::AsnKind> for AsnKind {
    fn from(value: nym_gateway_directory::AsnKind) -> Self {
        match value {
            nym_gateway_directory::AsnKind::Residential => AsnKind::Residential,
            nym_gateway_directory::AsnKind::Other => AsnKind::Other,
        }
    }
}

impl From<nym_vpn_lib_types::AsnKind> for AsnKind {
    fn from(value: nym_vpn_lib_types::AsnKind) -> Self {
        match value {
            nym_vpn_lib_types::AsnKind::Residential => AsnKind::Residential,
            nym_vpn_lib_types::AsnKind::Other => AsnKind::Other,
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct Asn {
    pub asn: String,
    pub name: String,
    pub kind: AsnKind,
}

impl From<nym_gateway_directory::Asn> for Asn {
    fn from(value: nym_gateway_directory::Asn) -> Self {
        Asn {
            asn: value.asn,
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

impl From<nym_vpn_lib_types::Asn> for Asn {
    fn from(value: nym_vpn_lib_types::Asn) -> Self {
        Asn {
            asn: value.asn,
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

#[derive(Debug, PartialEq, uniffi::Record, Clone)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub region: String,
    pub asn: Option<Asn>,
}

impl From<nym_gateway_directory::Location> for Location {
    fn from(value: nym_gateway_directory::Location) -> Self {
        Location {
            two_letter_iso_country_code: value.two_letter_iso_country_code,
            latitude: value.latitude,
            longitude: value.longitude,
            city: value.city,
            region: value.region,
            asn: value.asn.map(Into::into),
        }
    }
}

impl From<nym_vpn_lib_types::Location> for Location {
    fn from(value: nym_vpn_lib_types::Location) -> Self {
        Location {
            two_letter_iso_country_code: value.two_letter_iso_country_code,
            latitude: value.latitude,
            longitude: value.longitude,
            city: value.city,
            region: value.region,
            asn: value.asn.map(Into::into),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum GatewayType {
    MixnetEntry,
    MixnetExit,
    Wg,
}

impl From<GatewayType> for nym_vpn_lib_types::GatewayType {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MixnetEntry => nym_vpn_lib_types::GatewayType::MixnetEntry,
            GatewayType::MixnetExit => nym_vpn_lib_types::GatewayType::MixnetExit,
            GatewayType::Wg => nym_vpn_lib_types::GatewayType::Wg,
        }
    }
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
