// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
};

use nym_gateway_directory::split_ips;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Gateway {
    pub identity_key: String,
    pub moniker: String,
    pub location: Option<Location>,
    pub last_probe: Option<Probe>,
    pub mixnet_score: Option<Score>,
    pub wg_score: Option<Score>,
    pub exit_ipv4s: Vec<Ipv4Addr>,
    pub exit_ipv6s: Vec<Ipv6Addr>,
    pub build_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.two_letter_iso_country_code)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "last_updated_utc: {}", self.last_updated_utc)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Score {
    High,
    Medium,
    Low,
    None,
}

impl Score {
    pub fn from_i32(value: i32) -> Self {
        if value == 3 {
            Self::High
        } else if value == 2 {
            Self::Medium
        } else if value == 1 {
            Self::Low
        } else {
            Self::None
        }
    }
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

impl From<nym_gateway_directory::Country> for Country {
    fn from(country: nym_gateway_directory::Country) -> Self {
        Self {
            iso_code: country.iso_code().to_string(),
        }
    }
}

impl From<nym_validator_client::models::NymNodeDescription> for Gateway {
    fn from(node_description: nym_validator_client::models::NymNodeDescription) -> Self {
        let build_version = Some(node_description.version().to_owned());
        let (exit_ipv4s, exit_ipv6s) =
            split_ips(node_description.description.host_information.ip_address);
        Self {
            identity_key: node_description
                .description
                .host_information
                .keys
                .ed25519
                .to_string(),
            moniker: String::new(),
            location: None,
            last_probe: None,
            wg_score: None,
            mixnet_score: None,
            exit_ipv4s,
            exit_ipv6s,
            build_version,
        }
    }
}

impl From<nym_gateway_directory::Location> for Location {
    fn from(location: nym_gateway_directory::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: Some(location.latitude),
            longitude: Some(location.longitude),
        }
    }
}

impl From<nym_gateway_directory::Score> for Score {
    fn from(score: nym_gateway_directory::Score) -> Self {
        match score {
            nym_gateway_directory::Score::High(_) => Score::High,
            nym_gateway_directory::Score::Medium(_) => Score::Medium,
            nym_gateway_directory::Score::Low(_) => Score::Low,
            nym_gateway_directory::Score::None => Score::None,
        }
    }
}

impl From<nym_gateway_directory::Entry> for Entry {
    fn from(entry: nym_gateway_directory::Entry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

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

impl From<nym_gateway_directory::ProbeOutcome> for ProbeOutcome {
    fn from(outcome: nym_gateway_directory::ProbeOutcome) -> Self {
        Self {
            as_entry: Entry::from(outcome.as_entry),
            as_exit: outcome.as_exit.map(Exit::from),
        }
    }
}

impl From<nym_gateway_directory::Probe> for Probe {
    fn from(probe: nym_gateway_directory::Probe) -> Self {
        Self {
            last_updated_utc: probe.last_updated_utc,
            outcome: ProbeOutcome::from(probe.outcome),
        }
    }
}

impl From<nym_gateway_directory::Gateway> for Gateway {
    fn from(gateway: nym_gateway_directory::Gateway) -> Self {
        let (exit_ipv4s, exit_ipv6s) = gateway.split_ips();
        Self {
            identity_key: gateway.identity.to_string(),
            moniker: gateway.moniker,
            location: gateway.location.map(Location::from),
            last_probe: gateway.last_probe.map(Probe::from),
            wg_score: gateway.wg_score.map(Score::from),
            mixnet_score: gateway.mixnet_score.map(Score::from),
            exit_ipv4s,
            exit_ipv6s,
            build_version: gateway.version,
        }
    }
}
