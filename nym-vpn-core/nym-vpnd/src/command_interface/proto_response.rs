// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{VpnServiceInfoResult, VpnServiceStateChange, VpnServiceStatusResult};
use nym_vpn_proto::{
    ConnectionStateChange, ConnectionStatus, Error as ProtoError, InfoResponse, StatusResponse,
};

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
