// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    ActionAfterDisconnect, ClientErrorReason, ConnectionData, ForgetAccountError, Gateway,
    MixnetConnectionData, RegisterDeviceError, SyncAccountError, SyncDeviceError,
    TunnelConnectionData, TunnelState, VpnApiError, VpnApiErrorResponse, WireguardConnectionData,
    WireguardNode,
};

use crate::proto;

impl From<ActionAfterDisconnect> for proto::tunnel_state::ActionAfterDisconnect {
    fn from(value: ActionAfterDisconnect) -> Self {
        match value {
            ActionAfterDisconnect::Error => Self::Error,
            ActionAfterDisconnect::Nothing => Self::Nothing,
            ActionAfterDisconnect::Offline => Self::Offline,
            ActionAfterDisconnect::Reconnect => Self::Reconnect,
        }
    }
}

impl From<ClientErrorReason> for proto::tunnel_state::Error {
    fn from(value: ClientErrorReason) -> Self {
        match value {
            ClientErrorReason::Firewall => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::Firewall.into(),
                detail: None,
            },
            ClientErrorReason::Routing => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::Routing.into(),
                detail: None,
            },
            ClientErrorReason::SameEntryAndExitGateway => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::SameEntryAndExitGateway.into(),
                detail: None,
            },
            ClientErrorReason::InvalidEntryGatewayCountry => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::InvalidEntryGatewayCountry.into(),
                detail: None,
            },
            ClientErrorReason::InvalidExitGatewayCountry => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::InvalidExitGatewayCountry.into(),
                detail: None,
            },
            ClientErrorReason::MaxDevicesReached => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::MaxDevicesReached.into(),
                detail: None,
            },
            ClientErrorReason::BandwidthExceeded => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::BandwidthExceeded.into(),
                detail: None,
            },
            ClientErrorReason::SubscriptionExpired => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::SubscriptionExpired.into(),
                detail: None,
            },
            ClientErrorReason::Dns(detail) => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::Dns.into(),
                detail,
            },
            ClientErrorReason::Api(detail) => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::Api.into(),
                detail,
            },
            ClientErrorReason::DeviceTimeOutOfSync => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::DeviceTimeOutOfSync.into(),
                detail: None,
            },
            ClientErrorReason::CreateMixnetStorage => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::CreateMixnetStorage.into(),
                detail: None,
            },
            ClientErrorReason::Internal(detail) => proto::tunnel_state::Error {
                reason: proto::tunnel_state::ErrorStateReason::Internal.into(),
                detail,
            },
        }
    }
}

impl From<SyncAccountError> for proto::SyncAccountError {
    fn from(value: SyncAccountError) -> Self {
        match value {
            SyncAccountError::NoAccountStored => proto::SyncAccountError {
                error_detail: Some(proto::sync_account_error::ErrorDetail::NoAccountStored(
                    true,
                )),
            },
            SyncAccountError::SyncAccountEndpointFailure(vpn_api) => proto::SyncAccountError {
                error_detail: Some(proto::sync_account_error::ErrorDetail::VpnApi(
                    vpn_api.into(),
                )),
            },
            SyncAccountError::UnexpectedResponse(err) => proto::SyncAccountError {
                error_detail: Some(proto::sync_account_error::ErrorDetail::UnexpectedResponse(
                    err,
                )),
            },
            SyncAccountError::Offline => proto::SyncAccountError {
                error_detail: Some(proto::sync_account_error::ErrorDetail::Offline(true)),
            },
            SyncAccountError::Internal(err) => proto::SyncAccountError {
                error_detail: Some(proto::sync_account_error::ErrorDetail::Internal(err)),
            },
        }
    }
}

impl From<SyncDeviceError> for proto::SyncDeviceError {
    fn from(value: SyncDeviceError) -> Self {
        match value {
            SyncDeviceError::NoAccountStored => proto::SyncDeviceError {
                error_detail: Some(proto::sync_device_error::ErrorDetail::NoAccountStored(true)),
            },
            SyncDeviceError::NoDeviceStored => proto::SyncDeviceError {
                error_detail: Some(proto::sync_device_error::ErrorDetail::NoDeviceStored(true)),
            },
            SyncDeviceError::SyncDeviceEndpointFailure(vpn_api) => proto::SyncDeviceError {
                error_detail: Some(proto::sync_device_error::ErrorDetail::VpnApi(
                    vpn_api.into(),
                )),
            },
            SyncDeviceError::UnexpectedResponse(err) => proto::SyncDeviceError {
                error_detail: Some(proto::sync_device_error::ErrorDetail::UnexpectedResponse(
                    err,
                )),
            },
            SyncDeviceError::Offline => proto::SyncDeviceError {
                error_detail: Some(proto::sync_device_error::ErrorDetail::Offline(true)),
            },
            SyncDeviceError::Internal(err) => proto::SyncDeviceError {
                error_detail: Some(proto::sync_device_error::ErrorDetail::Internal(err)),
            },
        }
    }
}

impl From<RegisterDeviceError> for proto::RegisterDeviceError {
    fn from(value: RegisterDeviceError) -> Self {
        match value {
            RegisterDeviceError::NoAccountStored => proto::RegisterDeviceError {
                error_detail: Some(proto::register_device_error::ErrorDetail::NoAccountStored(
                    true,
                )),
            },
            RegisterDeviceError::NoDeviceStored => proto::RegisterDeviceError {
                error_detail: Some(proto::register_device_error::ErrorDetail::NoDeviceStored(
                    true,
                )),
            },
            RegisterDeviceError::RegisterDeviceEndpointFailure(vpn_api) => {
                proto::RegisterDeviceError {
                    error_detail: Some(proto::register_device_error::ErrorDetail::VpnApi(
                        vpn_api.into(),
                    )),
                }
            }
            RegisterDeviceError::UnexpectedResponse(err) => proto::RegisterDeviceError {
                error_detail: Some(
                    proto::register_device_error::ErrorDetail::UnexpectedResponse(err),
                ),
            },
            RegisterDeviceError::Offline => proto::RegisterDeviceError {
                error_detail: Some(proto::register_device_error::ErrorDetail::Offline(true)),
            },
            RegisterDeviceError::Internal(err) => proto::RegisterDeviceError {
                error_detail: Some(proto::register_device_error::ErrorDetail::Internal(err)),
            },
        }
    }
}

impl From<ForgetAccountError> for proto::ForgetAccountError {
    fn from(value: ForgetAccountError) -> Self {
        match value {
            ForgetAccountError::RegistrationInProgress => Self {
                error_detail: Some(
                    proto::forget_account_error::ErrorDetail::RegistrationInProgress(true),
                ),
            },
            ForgetAccountError::UpdateDeviceErrorResponse(vpn_api) => Self {
                error_detail: Some(proto::forget_account_error::ErrorDetail::VpnApi(
                    vpn_api.into(),
                )),
            },
            ForgetAccountError::UnexpectedResponse(err) => Self {
                error_detail: Some(
                    proto::forget_account_error::ErrorDetail::UnexpectedResponse(err),
                ),
            },
            ForgetAccountError::RemoveAccount(err) => Self {
                error_detail: Some(proto::forget_account_error::ErrorDetail::RemoveAccount(err)),
            },
            ForgetAccountError::RemoveDeviceKeys(err) => Self {
                error_detail: Some(proto::forget_account_error::ErrorDetail::RemoveDeviceKeys(
                    err,
                )),
            },
            ForgetAccountError::ResetCredentialStorage(err) => Self {
                error_detail: Some(
                    proto::forget_account_error::ErrorDetail::ResetCredentialStore(err),
                ),
            },
            ForgetAccountError::RemoveAccountFiles(err) => Self {
                error_detail: Some(
                    proto::forget_account_error::ErrorDetail::RemoveAccountFiles(err),
                ),
            },
            ForgetAccountError::InitDeviceKeys(err) => Self {
                error_detail: Some(proto::forget_account_error::ErrorDetail::InitDeviceKeys(
                    err,
                )),
            },
            ForgetAccountError::Internal(err) => Self {
                error_detail: Some(proto::forget_account_error::ErrorDetail::Internal(err)),
            },
        }
    }
}

impl From<VpnApiError> for proto::VpnApiError {
    fn from(value: VpnApiError) -> Self {
        let error_detail = match value {
            VpnApiError::Timeout(..) => proto::vpn_api_error::ErrorDetail::Timeout(true),
            VpnApiError::StatusCode { code, .. } => {
                proto::vpn_api_error::ErrorDetail::StatusCode(code.into())
            }
            VpnApiError::Response(vpn_api_error_response) => {
                proto::vpn_api_error::ErrorDetail::Response(vpn_api_error_response.into())
            }
        };
        Self {
            error_detail: Some(error_detail),
        }
    }
}

impl From<VpnApiErrorResponse> for proto::VpnApiErrorResponse {
    fn from(value: VpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<TunnelState> for proto::TunnelState {
    fn from(value: TunnelState) -> proto::TunnelState {
        let proto_state: proto::tunnel_state::State = match value {
            TunnelState::Disconnected => {
                proto::tunnel_state::State::Disconnected(proto::tunnel_state::Disconnected {})
            }
            TunnelState::Connecting {
                retry_attempt,
                connection_data,
            } => proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                retry_attempt,
                connection_data: connection_data.map(proto::ConnectionData::from),
            }),
            TunnelState::Connected { connection_data } => {
                proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                    connection_data: Some(proto::ConnectionData::from(connection_data)),
                })
            }
            TunnelState::Disconnecting { after_disconnect } => {
                proto::tunnel_state::State::Disconnecting(proto::tunnel_state::Disconnecting {
                    after_disconnect: proto::tunnel_state::ActionAfterDisconnect::from(
                        after_disconnect,
                    ) as i32,
                })
            }
            TunnelState::Offline { reconnect } => {
                proto::tunnel_state::State::Offline(proto::tunnel_state::Offline { reconnect })
            }
            TunnelState::Error(reason) => {
                proto::tunnel_state::State::Error(proto::tunnel_state::Error::from(reason))
            }
        };

        proto::TunnelState {
            state: Some(proto_state),
        }
    }
}

impl From<WireguardNode> for proto::WireguardNode {
    fn from(value: WireguardNode) -> proto::WireguardNode {
        proto::WireguardNode {
            public_key: value.public_key,
            endpoint: value.endpoint.to_string(),
            private_ipv4: value.private_ipv4.to_string(),
            private_ipv6: value.private_ipv6.to_string(),
        }
    }
}

impl From<ConnectionData> for proto::ConnectionData {
    fn from(value: ConnectionData) -> proto::ConnectionData {
        proto::ConnectionData {
            entry_gateway: Some(proto::Gateway::from(value.entry_gateway)),
            exit_gateway: Some(proto::Gateway::from(value.exit_gateway)),
            connected_at: value
                .connected_at
                .map(crate::conversions::prost::offset_datetime_into_proto_timestamp),
            tunnel: Some(proto::TunnelConnectionData::from(value.tunnel)),
        }
    }
}

impl From<MixnetConnectionData> for proto::MixnetConnectionData {
    fn from(value: MixnetConnectionData) -> proto::MixnetConnectionData {
        proto::MixnetConnectionData {
            nym_address: Some(proto::Address {
                nym_address: value.nym_address.to_string(),
                gateway_id: value.nym_address.gateway_id,
            }),
            exit_ipr: Some(proto::Address {
                nym_address: value.exit_ipr.to_string(),
                gateway_id: value.exit_ipr.gateway_id,
            }),
            entry_ip: value.entry_ip.to_string(),
            exit_ip: value.exit_ip.to_string(),
            ipv4: value.ipv4.to_string(),
            ipv6: value.ipv6.to_string(),
        }
    }
}

impl From<WireguardConnectionData> for proto::WireguardConnectionData {
    fn from(value: WireguardConnectionData) -> proto::WireguardConnectionData {
        proto::WireguardConnectionData {
            entry: Some(proto::WireguardNode::from(value.entry)),
            exit: Some(proto::WireguardNode::from(value.exit)),
        }
    }
}

impl From<TunnelConnectionData> for proto::TunnelConnectionData {
    fn from(value: TunnelConnectionData) -> proto::TunnelConnectionData {
        let state = match value {
            TunnelConnectionData::Mixnet(data) => proto::tunnel_connection_data::State::Mixnet(
                proto::tunnel_connection_data::Mixnet {
                    data: Some(proto::MixnetConnectionData::from(data)),
                },
            ),
            TunnelConnectionData::Wireguard(data) => {
                proto::tunnel_connection_data::State::Wireguard(
                    proto::tunnel_connection_data::Wireguard {
                        data: Some(proto::WireguardConnectionData::from(data)),
                    },
                )
            }
        };

        proto::TunnelConnectionData { state: Some(state) }
    }
}

impl From<Gateway> for proto::Gateway {
    fn from(value: Gateway) -> Self {
        Self { id: value.id }
    }
}
