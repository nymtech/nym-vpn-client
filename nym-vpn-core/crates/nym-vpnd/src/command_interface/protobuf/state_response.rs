// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{ConnectedStateDetails, VpnServiceStatusResult};
use nym_vpn_proto::{
    connected_state_details, ConnectionStatus, Error as ProtoError, MixConnectedStateDetails,
    StatusResponse, WgConnectedStateDetails,
};

impl From<ConnectedStateDetails> for connected_state_details::ConnectedStateDetails {
    fn from(value: ConnectedStateDetails) -> Self {
        match value {
            ConnectedStateDetails::Mix(details) => {
                connected_state_details::ConnectedStateDetails::Mix(MixConnectedStateDetails {
                    nym_address: Some(nym_vpn_proto::Address {
                        nym_address: details.nym_address.to_string(),
                    }),
                    exit_ipr: Some(nym_vpn_proto::Address {
                        nym_address: details.exit_ipr.to_string(),
                    }),
                    ipv4: details.ipv4.to_string(),
                    ipv6: details.ipv6.to_string(),
                })
            }
            ConnectedStateDetails::Wg(details) => {
                connected_state_details::ConnectedStateDetails::Wg(WgConnectedStateDetails {
                    entry_ipv4: details.entry_ipv4.to_string(),
                    exit_ipv4: details.exit_ipv4.to_string(),
                })
            }
        }
    }
}

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
                    entry_gateway: Some(nym_vpn_proto::Gateway {
                        id: conn_details.entry_gateway.to_string(),
                    }),
                    exit_gateway: Some(nym_vpn_proto::Gateway {
                        id: conn_details.exit_gateway.to_string(),
                    }),
                    protocol_details: Some(nym_vpn_proto::ConnectedStateDetails {
                        connected_state_details: Some(
                            connected_state_details::ConnectedStateDetails::from(
                                conn_details.specific_details,
                            ),
                        ),
                    }),
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
