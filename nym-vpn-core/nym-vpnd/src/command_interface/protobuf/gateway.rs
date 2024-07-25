// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::types::gateway;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

impl From<gateway::Location> for nym_vpn_proto::Location {
    fn from(location: gateway::Location) -> Self {
        nym_vpn_proto::Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
        }
    }
}

impl From<gateway::Entry> for nym_vpn_proto::AsEntry {
    fn from(entry: gateway::Entry) -> Self {
        nym_vpn_proto::AsEntry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<gateway::Exit> for nym_vpn_proto::AsExit {
    fn from(exit: gateway::Exit) -> Self {
        nym_vpn_proto::AsExit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<gateway::ProbeOutcome> for nym_vpn_proto::ProbeOutcome {
    fn from(outcome: gateway::ProbeOutcome) -> Self {
        let as_entry = Some(nym_vpn_proto::AsEntry::from(outcome.as_entry));
        let as_exit = outcome.as_exit.map(nym_vpn_proto::AsExit::from);
        nym_vpn_proto::ProbeOutcome { as_entry, as_exit }
    }
}

impl From<gateway::Probe> for nym_vpn_proto::Probe {
    fn from(probe: gateway::Probe) -> Self {
        let last_updated = OffsetDateTime::parse(&probe.last_updated_utc, &Rfc3339).ok();
        let last_updated_utc = last_updated.map(|timestamp| prost_types::Timestamp {
            seconds: timestamp.unix_timestamp(),
            nanos: timestamp.nanosecond() as i32,
        });
        let outcome = Some(nym_vpn_proto::ProbeOutcome::from(probe.outcome));
        nym_vpn_proto::Probe {
            last_updated_utc,
            outcome,
        }
    }
}

impl From<gateway::Gateway> for nym_vpn_proto::EntryGateway {
    fn from(gateway: gateway::Gateway) -> Self {
        let id = Some(nym_vpn_proto::Gateway {
            id: gateway.identity_key.to_string(),
        });
        let location = gateway.location.map(nym_vpn_proto::Location::from);
        let last_probe = gateway.last_probe.map(nym_vpn_proto::Probe::from);
        nym_vpn_proto::EntryGateway {
            id,
            location,
            last_probe,
        }
    }
}
