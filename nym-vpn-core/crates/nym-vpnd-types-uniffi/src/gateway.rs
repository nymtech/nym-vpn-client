// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(uniffi::Record)]
pub struct Gateway {
    pub identity_key: String,
    pub moniker: String,
    pub location: Option<Location>,
    pub mixnet_score: Option<Score>,
    pub wg_score: Option<Score>,
}

#[derive(uniffi::Enum)]
pub enum Score {
    High,
    Medium,
    Low,
    None,
}

#[derive(uniffi::Record)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(uniffi::Record)]
pub struct Country {
    pub iso_code: String,
}

impl From<nym_vpnd_types::gateway::Location> for Location {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<nym_vpnd_types::gateway::Score> for Score {
    fn from(score: nym_vpnd_types::gateway::Score) -> Self {
        match score {
            nym_vpnd_types::gateway::Score::High => Score::High,
            nym_vpnd_types::gateway::Score::Medium => Score::Medium,
            nym_vpnd_types::gateway::Score::Low => Score::Low,
            nym_vpnd_types::gateway::Score::None => Score::None,
        }
    }
}

impl From<nym_vpnd_types::gateway::Gateway> for Gateway {
    fn from(gateway: nym_vpnd_types::gateway::Gateway) -> Self {
        Gateway {
            identity_key: gateway.identity_key,
            moniker: gateway.moniker,
            location: gateway.location.map(Location::from),
            mixnet_score: gateway.mixnet_score.map(Score::from),
            wg_score: gateway.wg_score.map(Score::from),
        }
    }
}

impl From<nym_vpnd_types::gateway::Country> for Country {
    fn from(country: nym_vpnd_types::gateway::Country) -> Self {
        Country {
            iso_code: country.iso_code,
        }
    }
}
