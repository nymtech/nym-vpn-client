// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{conversions::ConversionError, Score};

impl From<crate::Location> for nym_vpnd_types::gateway::Location {
    fn from(location: crate::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<crate::Score> for nym_vpnd_types::gateway::Score {
    fn from(score: Score) -> Self {
        match score {
            Score::None => nym_vpnd_types::gateway::Score::None,
            Score::Low => nym_vpnd_types::gateway::Score::Low,
            Score::Medium => nym_vpnd_types::gateway::Score::Medium,
            Score::High => nym_vpnd_types::gateway::Score::High,
        }
    }
}

impl From<crate::AsEntry> for nym_vpnd_types::gateway::Entry {
    fn from(entry: crate::AsEntry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<crate::AsExit> for nym_vpnd_types::gateway::Exit {
    fn from(exit: crate::AsExit) -> Self {
        Self {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl TryFrom<crate::ProbeOutcome> for nym_vpnd_types::gateway::ProbeOutcome {
    type Error = ConversionError;

    fn try_from(outcome: crate::ProbeOutcome) -> Result<Self, Self::Error> {
        let as_entry = outcome
            .as_entry
            .map(nym_vpnd_types::gateway::Entry::from)
            .ok_or(ConversionError::generic("missing as entry"))?;
        let as_exit = outcome.as_exit.map(nym_vpnd_types::gateway::Exit::from);
        Ok(Self { as_entry, as_exit })
    }
}

impl TryFrom<crate::Probe> for nym_vpnd_types::gateway::Probe {
    type Error = ConversionError;

    fn try_from(probe: crate::Probe) -> Result<Self, Self::Error> {
        let last_updated_utc = probe
            .last_updated_utc
            .ok_or(ConversionError::generic("missing last updated timestamp"))
            .map(|timestamp| timestamp.to_string())?;
        let outcome = probe
            .outcome
            .ok_or(ConversionError::generic("missing probe outcome"))
            .and_then(nym_vpnd_types::gateway::ProbeOutcome::try_from)?;
        Ok(Self {
            last_updated_utc,
            outcome,
        })
    }
}

impl TryFrom<crate::GatewayResponse> for nym_vpnd_types::gateway::Gateway {
    type Error = ConversionError;
    fn try_from(gateway: crate::GatewayResponse) -> Result<Self, Self::Error> {
        let identity_key = gateway
            .id
            .map(|id| id.id)
            .ok_or_else(|| ConversionError::generic("missing gateway id"))?;
        let moniker = gateway.moniker;
        let location = gateway
            .location
            .map(nym_vpnd_types::gateway::Location::from);
        let last_probe = gateway
            .last_probe
            .map(nym_vpnd_types::gateway::Probe::try_from)
            .transpose()?;
        let mixnet_score = gateway
            .mixnet_score
            .map(nym_vpnd_types::gateway::Score::from_i32);
        let wg_score = gateway
            .wg_score
            .map(nym_vpnd_types::gateway::Score::from_i32);
        Ok(Self {
            identity_key,
            moniker,
            location,
            last_probe,
            wg_score,
            mixnet_score,
        })
    }
}

impl From<crate::Location> for nym_vpnd_types::gateway::Country {
    fn from(location: crate::Location) -> Self {
        Self {
            iso_code: location.two_letter_iso_country_code,
        }
    }
}
