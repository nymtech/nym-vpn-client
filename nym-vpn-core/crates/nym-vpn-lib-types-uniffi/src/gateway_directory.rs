// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{Ipv4Addr, Ipv6Addr};

use nym_sdk::mixnet::{NodeIdentity, Recipient};

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
    Location { location: String },
    Random,
}

impl From<EntryPoint> for nym_gateway_directory::EntryPoint {
    fn from(value: EntryPoint) -> Self {
        match value {
            EntryPoint::Gateway { identity } => {
                nym_gateway_directory::EntryPoint::Gateway { identity }
            }
            EntryPoint::Location { location } => {
                nym_gateway_directory::EntryPoint::Location { location }
            }
            EntryPoint::Random => nym_gateway_directory::EntryPoint::Random,
        }
    }
}

#[derive(uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum ExitPoint {
    Address { address: Recipient },
    Gateway { identity: NodeIdentity },
    Location { location: String },
    Random,
}

impl From<ExitPoint> for nym_gateway_directory::ExitPoint {
    fn from(value: ExitPoint) -> Self {
        match value {
            ExitPoint::Address { address } => nym_gateway_directory::ExitPoint::Address {
                address: Box::new(address),
            },
            ExitPoint::Gateway { identity } => {
                nym_gateway_directory::ExitPoint::Gateway { identity }
            }
            ExitPoint::Location { location } => {
                nym_gateway_directory::ExitPoint::Location { location }
            }
            ExitPoint::Random => nym_gateway_directory::ExitPoint::Random,
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
    pub exit_ipv4s: Vec<Ipv4Addr>,
    pub exit_ipv6s: Vec<Ipv6Addr>,
    pub build_version: Option<String>,
}

impl From<nym_gateway_directory::Gateway> for GatewayInfo {
    fn from(value: nym_gateway_directory::Gateway) -> Self {
        let (exit_ipv4s, exit_ipv6s) = value.split_ips();
        GatewayInfo {
            moniker: value.moniker,
            location: value.location.map(Location::from),
            id: value.identity,
            mixnet_score: value.mixnet_score.map(Score::from),
            wg_score: value.wg_score.map(Score::from),
            exit_ipv4s,
            exit_ipv6s,
            build_version: value.version,
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
