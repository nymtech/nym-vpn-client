// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(uniffi::Record)]
pub struct Gateway {
    pub identity_key: String,
    pub moniker: String,
    pub location: Option<GatewayLocation>,
    pub mixnet_score: Option<GatewayScore>,
    pub wg_performance: Option<GatewayPerformance>,
}

#[derive(uniffi::Enum)]
pub enum GatewayScore {
    High,
    Medium,
    Low,
    Offline,
}

#[derive(uniffi::Record)]
pub struct GatewayPerformance {
    pub last_updated_utc: String,
    pub score: GatewayScore,
    pub load: GatewayScore,
    pub uptime_percentage_last_24_hours: f32,
}

#[derive(uniffi::Enum)]
pub enum AsnKind {
    Residential,
    Other,
}

#[derive(uniffi::Record)]
pub struct Asn {
    pub asn: String,
    pub name: String,
    pub kind: AsnKind,
}

#[derive(uniffi::Record)]
pub struct GatewayLocation {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,

    pub city: String,
    pub region: String,

    pub asn: Option<Asn>,
}

#[derive(uniffi::Record)]
pub struct GatewayCountry {
    pub iso_code: String,
}

impl From<nym_vpnd_types::gateway::AsnKind> for AsnKind {
    fn from(value: nym_vpnd_types::gateway::AsnKind) -> Self {
        match value {
            nym_vpnd_types::gateway::AsnKind::Residential => AsnKind::Residential,
            nym_vpnd_types::gateway::AsnKind::Other => AsnKind::Other,
        }
    }
}

impl From<nym_vpnd_types::gateway::Asn> for Asn {
    fn from(value: nym_vpnd_types::gateway::Asn) -> Self {
        Asn {
            asn: value.asn,
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

impl From<nym_vpnd_types::gateway::Location> for GatewayLocation {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        GatewayLocation {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: location.asn.map(Into::into),
        }
    }
}

impl From<nym_vpnd_types::gateway::Score> for GatewayScore {
    fn from(score: nym_vpnd_types::gateway::Score) -> Self {
        match score {
            nym_vpnd_types::gateway::Score::High => GatewayScore::High,
            nym_vpnd_types::gateway::Score::Medium => GatewayScore::Medium,
            nym_vpnd_types::gateway::Score::Low => GatewayScore::Low,
            nym_vpnd_types::gateway::Score::Offline => GatewayScore::Offline,
        }
    }
}

impl From<nym_vpnd_types::gateway::Performance> for GatewayPerformance {
    fn from(gateway: nym_vpnd_types::gateway::Performance) -> Self {
        GatewayPerformance {
            last_updated_utc: gateway.last_updated_utc,
            score: gateway.score.into(),
            load: gateway.load.into(),
            uptime_percentage_last_24_hours: gateway.uptime_percentage_last_24_hours,
        }
    }
}

impl From<nym_vpnd_types::gateway::Gateway> for Gateway {
    fn from(gateway: nym_vpnd_types::gateway::Gateway) -> Self {
        Gateway {
            identity_key: gateway.identity_key,
            moniker: gateway.moniker,
            location: gateway.location.map(GatewayLocation::from),
            mixnet_score: gateway.mixnet_score.map(GatewayScore::from),
            wg_performance: gateway.wg_performance.map(GatewayPerformance::from),
        }
    }
}

impl From<nym_vpnd_types::gateway::Country> for GatewayCountry {
    fn from(country: nym_vpnd_types::gateway::Country) -> Self {
        GatewayCountry {
            iso_code: country.iso_code,
        }
    }
}
