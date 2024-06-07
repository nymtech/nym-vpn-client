// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{VpnServiceStateChange, VpnServiceStatusResult};
use nym_vpn_proto::{ConnectionStateChange, ConnectionStatus, Error as ProtoError, StatusResponse};

impl From<VpnServiceStatusResult> for StatusResponse {
    fn from(status: VpnServiceStatusResult) -> Self {
        let mut details = None;
        let mut error = None;
        let status = match status {
            VpnServiceStatusResult::NotConnected => ConnectionStatus::NotConnected,
            VpnServiceStatusResult::Connecting => ConnectionStatus::Connecting,
            VpnServiceStatusResult::Connected {
                nym_address,
                entry_gateway,
                exit_gateway,
                exit_ipr,
                ipv4,
                ipv6,
                since,
            } => {
                let timestamp = prost_types::Timestamp {
                    seconds: since.unix_timestamp(),
                    nanos: since.nanosecond() as i32,
                };
                details = Some(nym_vpn_proto::ConnectionDetails {
                    nym_address: Some(nym_vpn_proto::Address {
                        nym_address: nym_address.to_string(),
                    }),
                    entry_gateway,
                    exit_gateway,
                    exit_ipr: Some(nym_vpn_proto::Address {
                        nym_address: exit_ipr.to_string(),
                    }),
                    ipv4: ipv4.to_string(),
                    ipv6: ipv6.to_string(),
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
