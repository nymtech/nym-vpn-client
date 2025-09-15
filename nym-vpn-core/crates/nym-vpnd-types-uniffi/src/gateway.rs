// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(uniffi::Record)]
pub struct Gateway {
    pub identity_key: String,
    pub moniker: String,
    pub location: Option<GatewayLocation>,
    pub mixnet_score: Option<GatewayScore>,
    pub wg_score: Option<GatewayScore>,
}

#[derive(uniffi::Enum)]
pub enum GatewayScore {
    High,
    Medium,
    Low,
    None,
}

#[derive(uniffi::Record)]
pub struct GatewayLocation {
    pub two_letter_iso_country_code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(uniffi::Record)]
pub struct GatewayCountry {
    pub iso_code: String,
}

impl From<nym_vpnd_types::gateway::Location> for GatewayLocation {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        GatewayLocation {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<nym_vpnd_types::gateway::Score> for GatewayScore {
    fn from(score: nym_vpnd_types::gateway::Score) -> Self {
        match score {
            nym_vpnd_types::gateway::Score::High => GatewayScore::High,
            nym_vpnd_types::gateway::Score::Medium => GatewayScore::Medium,
            nym_vpnd_types::gateway::Score::Low => GatewayScore::Low,
            nym_vpnd_types::gateway::Score::None => GatewayScore::None,
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
            wg_score: gateway.wg_score.map(GatewayScore::from),
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
