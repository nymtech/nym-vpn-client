// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::VpnServiceStateChange;

impl From<VpnServiceStateChange> for nym_vpn_proto::ConnectionStateChange {
    fn from(status: VpnServiceStateChange) -> Self {
        let mut error = None;

        use nym_vpn_proto::ConnectionStatus;
        let status = match status {
            VpnServiceStateChange::NotConnected => ConnectionStatus::NotConnected,
            VpnServiceStateChange::Connecting => ConnectionStatus::Connecting,
            VpnServiceStateChange::Connected => ConnectionStatus::Connected,
            VpnServiceStateChange::Disconnecting => ConnectionStatus::Disconnecting,
            VpnServiceStateChange::ConnectionFailed(reason) => {
                error = Some(nym_vpn_proto::Error::from(reason));
                nym_vpn_proto::ConnectionStatus::ConnectionFailed
            }
        } as i32;

        Self { status, error }
    }
}
