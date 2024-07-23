// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{VpnServiceInfoResult, VpnServiceStateChange, VpnServiceStatusResult};
use nym_vpn_proto::{
    ConnectionStateChange, ConnectionStatus, Error as ProtoError, InfoResponse, StatusResponse,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

impl From<VpnServiceStatusResult> for StatusResponse {
    fn from(status: VpnServiceStatusResult) -> Self {
        let mut details = None;
        let mut error = None;
        let status = match status {
            VpnServiceStatusResult::NotConnected => ConnectionStatus::NotConnected,
            VpnServiceStatusResult::Connecting => ConnectionStatus::Connecting,
            VpnServiceStatusResult::Connected(conn_details) => {
                let timestamp = prost_types::Timestamp {
                    seconds: conn_details.since.unix_timestamp(),
                    nanos: conn_details.since.nanosecond() as i32,
                };
                details = Some(nym_vpn_proto::ConnectionDetails {
                    nym_address: Some(nym_vpn_proto::Address {
                        nym_address: conn_details.nym_address.to_string(),
                    }),
                    entry_gateway: Some(nym_vpn_proto::Gateway {
                        id: conn_details.entry_gateway.to_string(),
                    }),
                    exit_gateway: Some(nym_vpn_proto::Gateway {
                        id: conn_details.exit_gateway.to_string(),
                    }),
                    exit_ipr: Some(nym_vpn_proto::Address {
                        nym_address: conn_details.exit_ipr.to_string(),
                    }),
                    ipv4: conn_details.ipv4.to_string(),
                    ipv6: conn_details.ipv6.to_string(),
                    since: Some(timestamp),
                });
                ConnectionStatus::Connected
            }
            VpnServiceStatusResult::Disconnecting => ConnectionStatus::Disconnecting,
            VpnServiceStatusResult::ConnectionFailed(reason) => {
                error = Some(ProtoError::from(reason));
                ConnectionStatus::ConnectionFailed
            }
        } as i32;

        StatusResponse {
            status,
            details,
            error,
        }
    }
}

impl From<VpnServiceInfoResult> for InfoResponse {
    fn from(info: VpnServiceInfoResult) -> Self {
        let build_timestamp = info.build_timestamp.map(|ts| prost_types::Timestamp {
            seconds: ts.unix_timestamp(),
            nanos: ts.nanosecond() as i32,
        });
        InfoResponse {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            git_commit: info.git_commit,
        }
    }
}

impl From<VpnServiceStateChange> for ConnectionStateChange {
    fn from(status: VpnServiceStateChange) -> Self {
        let mut error = None;
        let status = match status {
            VpnServiceStateChange::NotConnected => ConnectionStatus::NotConnected,
            VpnServiceStateChange::Connecting => ConnectionStatus::Connecting,
            VpnServiceStateChange::Connected => ConnectionStatus::Connected,
            VpnServiceStateChange::Disconnecting => ConnectionStatus::Disconnecting,
            VpnServiceStateChange::ConnectionFailed(reason) => {
                error = Some(ProtoError::from(reason));
                ConnectionStatus::ConnectionFailed
            }
        } as i32;

        ConnectionStateChange { status, error }
    }
}

pub(super) fn entry_gateway_from_vpn_api(
    gateway: nym_vpn_api_client::Gateway,
) -> nym_vpn_proto::EntryGateway {
    let last_updated = gateway
        .last_probe.clone()
        .and_then(|probe| OffsetDateTime::parse(&probe.last_updated_utc, &Rfc3339).ok());
    let last_updated_utc = last_updated.map(|timestamp| prost_types::Timestamp {
        seconds: timestamp.unix_timestamp(),
        nanos: timestamp.nanosecond() as i32,
    });

    let as_entry = gateway.last_probe.clone().map(|probe| {
        nym_vpn_proto::AsEntry {
            can_connect: probe.outcome.as_entry.can_connect,
            can_route: probe.outcome.as_entry.can_route,
        }
    });

    let as_exit = gateway.last_probe.clone().and_then(|probe| {
        probe.outcome.as_exit.map(|as_exit| {
            nym_vpn_proto::AsExit {
                can_connect: as_exit.can_connect,
                can_route_ip_v4: as_exit.can_route_ip_v4,
                can_route_ip_v6: as_exit.can_route_ip_v6,
                can_route_ip_external_v4: as_exit.can_route_ip_external_v4,
                can_route_ip_external_v6: as_exit.can_route_ip_external_v6,
            }
        })
    });

    let last_probe = gateway.last_probe.map(|p| {
        nym_vpn_proto::Probe {
            last_updated_utc,
            outcome: Some(nym_vpn_proto::ProbeOutcome {
                as_entry,
                as_exit,
            }),
        }
    });

    nym_vpn_proto::EntryGateway {
        id: Some(nym_vpn_proto::Gateway {
            id: gateway.identity_key.to_string(),
        }),
        location: Some(nym_vpn_proto::Location {
            two_letter_iso_country_code: gateway.location.two_letter_iso_country_code,
        }),
        // last_probe: Some(nym_vpn_proto::Probe {
        //     last_updated_utc,
        //     outcome: Some(nym_vpn_proto::ProbeOutcome {
        //         as_entry,
        //         as_exit,
        //     }),
        // }),
        last_probe,
    }
}
