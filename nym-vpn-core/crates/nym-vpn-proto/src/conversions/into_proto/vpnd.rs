// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpnd_types::gateway::Score;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

impl From<nym_vpnd_types::gateway::Location> for crate::Location {
    fn from(location: nym_vpnd_types::gateway::Location) -> Self {
        crate::Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<Score> for crate::Score {
    fn from(score: Score) -> Self {
        match score {
            Score::High => crate::Score::High,
            Score::Medium => crate::Score::Medium,
            Score::Low => crate::Score::Low,
            Score::None => crate::Score::None,
        }
    }
}

impl From<nym_vpnd_types::gateway::Entry> for crate::AsEntry {
    fn from(entry: nym_vpnd_types::gateway::Entry) -> Self {
        crate::AsEntry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpnd_types::gateway::Exit> for crate::AsExit {
    fn from(exit: nym_vpnd_types::gateway::Exit) -> Self {
        crate::AsExit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<nym_vpnd_types::gateway::ProbeOutcome> for crate::ProbeOutcome {
    fn from(outcome: nym_vpnd_types::gateway::ProbeOutcome) -> Self {
        let as_entry = Some(crate::AsEntry::from(outcome.as_entry));
        let as_exit = outcome.as_exit.map(crate::AsExit::from);
        let wg = None;
        crate::ProbeOutcome {
            as_entry,
            as_exit,
            wg,
        }
    }
}

impl From<nym_vpnd_types::gateway::Probe> for crate::Probe {
    fn from(probe: nym_vpnd_types::gateway::Probe) -> Self {
        let last_updated = OffsetDateTime::parse(&probe.last_updated_utc, &Rfc3339).ok();
        let last_updated_utc = last_updated.map(|timestamp| prost_types::Timestamp {
            seconds: timestamp.unix_timestamp(),
            nanos: timestamp.nanosecond() as i32,
        });
        let outcome = Some(crate::ProbeOutcome::from(probe.outcome));
        crate::Probe {
            last_updated_utc,
            outcome,
        }
    }
}

impl From<nym_vpnd_types::gateway::Gateway> for crate::GatewayResponse {
    fn from(gateway: nym_vpnd_types::gateway::Gateway) -> Self {
        let id = Some(crate::Gateway {
            id: gateway.identity_key.to_string(),
        });
        let location = gateway.location.map(crate::Location::from);
        let last_probe = gateway.last_probe.map(crate::Probe::from);
        let moniker = gateway.moniker;
        crate::GatewayResponse {
            id,
            location,
            last_probe,
            wg_score: gateway
                .wg_score
                .map(|score| crate::Score::from(score) as i32),
            mixnet_score: gateway
                .mixnet_score
                .map(|score| crate::Score::from(score) as i32),
            moniker,
        }
    }
}

impl From<nym_vpnd_types::gateway::Country> for crate::Location {
    fn from(country: nym_vpnd_types::gateway::Country) -> Self {
        crate::Location {
            two_letter_iso_country_code: country.iso_code().to_string(),
            latitude: None,
            longitude: None,
        }
    }
}
