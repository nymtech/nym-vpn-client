// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{VpnServiceStateChange, VpnServiceStatusResult};
use nym_vpn_proto::{ConnectionStateChange, ConnectionStatus, Error as ProtoError, StatusResponse};

impl From<VpnServiceStatusResult> for StatusResponse {
    fn from(status: VpnServiceStatusResult) -> Self {
        let mut error = None;
        let status = match status {
            VpnServiceStatusResult::NotConnected => ConnectionStatus::NotConnected,
            VpnServiceStatusResult::Connecting => ConnectionStatus::Connecting,
            VpnServiceStatusResult::Connected => ConnectionStatus::Connected,
            VpnServiceStatusResult::Disconnecting => ConnectionStatus::Disconnecting,
            VpnServiceStatusResult::ConnectionFailed(reason) => {
                error = Some(ProtoError::from(reason));
                ConnectionStatus::ConnectionFailed
            }
        } as i32;

        StatusResponse { status, error }
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
