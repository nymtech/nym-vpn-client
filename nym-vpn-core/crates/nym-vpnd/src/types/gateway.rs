// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Gateway {
    pub identity_key: String,
    pub location: Option<Location>,
    pub last_probe: Option<Probe>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub can_connect: bool,
    pub can_route: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exit {
    pub can_connect: bool,
    pub can_route_ip_v4: bool,
    pub can_route_ip_external_v4: bool,
    pub can_route_ip_v6: bool,
    pub can_route_ip_external_v6: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country(String);

impl From<String> for Country {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<nym_vpn_api_client::responses::Country> for Country {
    fn from(country: nym_vpn_api_client::responses::Country) -> Self {
        Self(country.into_string())
    }
}

impl From<nym_validator_client::models::DescribedGateway> for Gateway {
    fn from(gateway: nym_validator_client::models::DescribedGateway) -> Self {
        Self {
            identity_key: gateway.bond.identity().clone(),
            location: None,
            last_probe: None,
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::Location> for Location {
    fn from(location: nym_vpn_lib::gateway_directory::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::Entry> for Entry {
    fn from(entry: nym_vpn_lib::gateway_directory::Entry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::Exit> for Exit {
    fn from(exit: nym_vpn_lib::gateway_directory::Exit) -> Self {
        Self {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::ProbeOutcome> for ProbeOutcome {
    fn from(outcome: nym_vpn_lib::gateway_directory::ProbeOutcome) -> Self {
        Self {
            as_entry: Entry::from(outcome.as_entry),
            as_exit: outcome.as_exit.map(Exit::from),
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::Probe> for Probe {
    fn from(probe: nym_vpn_lib::gateway_directory::Probe) -> Self {
        Self {
            last_updated_utc: probe.last_updated_utc,
            outcome: ProbeOutcome::from(probe.outcome),
        }
    }
}

impl From<nym_vpn_lib::gateway_directory::Gateway> for Gateway {
    fn from(gateway: nym_vpn_lib::gateway_directory::Gateway) -> Self {
        Self {
            identity_key: gateway.identity.to_string(),
            location: gateway.location.map(Location::from),
            last_probe: gateway.last_probe.map(Probe::from),
        }
    }
}
