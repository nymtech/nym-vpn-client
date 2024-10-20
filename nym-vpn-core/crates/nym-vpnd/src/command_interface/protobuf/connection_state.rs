// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::ReadyToConnect;
use nym_vpn_proto::{
    is_ready_to_connect_response::IsReadyToConnectResponseType, ConnectionStateChange,
    ConnectionStatus, Error as ProtoError,
};

use crate::service::VpnServiceStateChange;

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

pub(crate) fn into_is_ready_to_connect_response_type(
    ready: ReadyToConnect,
) -> IsReadyToConnectResponseType {
    match ready {
        ReadyToConnect::Ready => IsReadyToConnectResponseType::Ready,
        ReadyToConnect::NoMnemonicStored => IsReadyToConnectResponseType::NoAccountStored,
        ReadyToConnect::AccountNotActive => IsReadyToConnectResponseType::AccountNotActive,
        ReadyToConnect::NoActiveSubscription => IsReadyToConnectResponseType::NoActiveSubscription,
        ReadyToConnect::DeviceNotRegistered => IsReadyToConnectResponseType::DeviceNotRegistered,
        ReadyToConnect::DeviceNotActive => IsReadyToConnectResponseType::DeviceNotActive,
    }
}
