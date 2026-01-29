// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AccountCommandError, AvailableTickets, DeeplinkClient, DeeplinkKind, GetDeeplinkParams,
    VpnAccountSummary, VpnApiError, VpnApiErrorResponse,
};

use crate::{
    conversions::{
        ConversionError,
        prost::{offset_datetime_into_proto_timestamp, prost_timestamp_into_offset_datetime},
    },
    proto,
};

impl TryFrom<proto::AccountCommandError> for AccountCommandError {
    type Error = ConversionError;

    fn try_from(value: proto::AccountCommandError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "StoreAccountError.error_detail",
        ))?;
        Ok(match error_detail {
            proto::account_command_error::ErrorDetail::StorageError(err) => Self::Storage(err),
            proto::account_command_error::ErrorDetail::Internal(err) => Self::Internal(err),
            proto::account_command_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::VpnApi(vpn_api.try_into()?)
            }
            proto::account_command_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedVpnApiResponse(err)
            }
            proto::account_command_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            proto::account_command_error::ErrorDetail::NoDeviceStored(_) => Self::NoDeviceStored,
            proto::account_command_error::ErrorDetail::ExistingAccount(_) => Self::ExistingAccount,
            proto::account_command_error::ErrorDetail::Offline(_) => Self::Offline,
            proto::account_command_error::ErrorDetail::InsufficientFunds(_) => {
                Self::InsufficientFunds
            }
            proto::account_command_error::ErrorDetail::InvalidMnemonic(message) => {
                Self::InvalidMnemonic(message)
            }
            proto::account_command_error::ErrorDetail::InvalidSecret(message) => {
                Self::InvalidSecret(message)
            }
            proto::account_command_error::ErrorDetail::NyxdConnectionFailure(err) => {
                Self::NyxdConnectionFailure(err)
            }
            proto::account_command_error::ErrorDetail::NyxdQueryFailure(err) => {
                Self::NyxdQueryFailure(err)
            }
            proto::account_command_error::ErrorDetail::AccountDoesntExistOnChain(_) => {
                Self::AccountDoesntExistOnChain
            }
            proto::account_command_error::ErrorDetail::AccountDecentralised(_) => {
                Self::AccountDecentralised
            }
            proto::account_command_error::ErrorDetail::AccountNotDecentralised(_) => {
                Self::AccountNotDecentralised
            }
            proto::account_command_error::ErrorDetail::ZkNymAcquisitionFailure(err) => {
                Self::ZkNymAcquisitionFailure(err)
            }
            proto::account_command_error::ErrorDetail::DeeplinkError(message) => {
                Self::DeeplinkError(message)
            }
        })
    }
}

impl TryFrom<proto::VpnApiError> for VpnApiError {
    type Error = ConversionError;

    fn try_from(value: proto::VpnApiError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("VpnApiError.error_detail"))?;
        Ok(match error_detail {
            proto::vpn_api_error::ErrorDetail::Timeout(msg) => Self::Timeout(msg),
            proto::vpn_api_error::ErrorDetail::StatusCode(e) => Self::StatusCode {
                code: e.code.try_into().map_err(|e| {
                    ConversionError::Generic(format!("failed to convert status code: {e}"))
                })?,
                msg: e.message,
            },
            proto::vpn_api_error::ErrorDetail::Response(vpn_api_error_response) => {
                Self::Response(vpn_api_error_response.into())
            }
        })
    }
}

impl From<proto::VpnApiErrorResponse> for VpnApiErrorResponse {
    fn from(value: proto::VpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<AccountCommandError> for proto::AccountCommandError {
    fn from(value: AccountCommandError) -> Self {
        match value {
            AccountCommandError::Internal(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::Internal(err)),
            },
            AccountCommandError::Storage(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::StorageError(err)),
            },
            AccountCommandError::VpnApi(vpn_api_error) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::VpnApi(
                    vpn_api_error.into(),
                )),
            },
            AccountCommandError::UnexpectedVpnApiResponse(err) => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::UnexpectedResponse(err),
                ),
            },
            AccountCommandError::NoAccountStored => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::NoAccountStored(
                    true,
                )),
            },
            AccountCommandError::NoDeviceStored => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::NoDeviceStored(
                    true,
                )),
            },
            AccountCommandError::ExistingAccount => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::ExistingAccount(
                    true,
                )),
            },
            AccountCommandError::Offline => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::Offline(true)),
            },
            AccountCommandError::InsufficientFunds => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::InsufficientFunds(true),
                ),
            },
            AccountCommandError::InvalidMnemonic(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::InvalidMnemonic(
                    err,
                )),
            },
            AccountCommandError::InvalidSecret(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::InvalidSecret(
                    err,
                )),
            },
            AccountCommandError::NyxdConnectionFailure(err) => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::NyxdConnectionFailure(err),
                ),
            },
            AccountCommandError::NyxdQueryFailure(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::NyxdQueryFailure(
                    err,
                )),
            },
            AccountCommandError::AccountDoesntExistOnChain => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::AccountDoesntExistOnChain(true),
                ),
            },
            AccountCommandError::AccountNotDecentralised => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::AccountNotDecentralised(true),
                ),
            },
            AccountCommandError::AccountDecentralised => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::AccountDecentralised(true),
                ),
            },
            AccountCommandError::ZkNymAcquisitionFailure(err) => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::ZkNymAcquisitionFailure(err),
                ),
            },
            AccountCommandError::DeeplinkError(message) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::DeeplinkError(
                    message,
                )),
            },
        }
    }
}

impl From<AvailableTickets> for proto::AvailableTickets {
    fn from(ticketbooks: AvailableTickets) -> Self {
        Self {
            mixnet_entry_tickets: ticketbooks.mixnet_entry_tickets,
            mixnet_entry_data: ticketbooks.mixnet_entry_data,
            mixnet_entry_data_si: ticketbooks.mixnet_entry_data_si,
            mixnet_exit_tickets: ticketbooks.mixnet_exit_tickets,
            mixnet_exit_data: ticketbooks.mixnet_exit_data,
            mixnet_exit_data_si: ticketbooks.mixnet_exit_data_si,
            vpn_entry_tickets: ticketbooks.vpn_entry_tickets,
            vpn_entry_data: ticketbooks.vpn_entry_data,
            vpn_entry_data_si: ticketbooks.vpn_entry_data_si,
            vpn_exit_tickets: ticketbooks.vpn_exit_tickets,
            vpn_exit_data: ticketbooks.vpn_exit_data,
            vpn_exit_data_si: ticketbooks.vpn_exit_data_si,
        }
    }
}

impl From<proto::AvailableTickets> for AvailableTickets {
    fn from(ticketbooks: proto::AvailableTickets) -> Self {
        Self {
            mixnet_entry_tickets: ticketbooks.mixnet_entry_tickets,
            mixnet_entry_data: ticketbooks.mixnet_entry_data,
            mixnet_entry_data_si: ticketbooks.mixnet_entry_data_si,
            mixnet_exit_tickets: ticketbooks.mixnet_exit_tickets,
            mixnet_exit_data: ticketbooks.mixnet_exit_data,
            mixnet_exit_data_si: ticketbooks.mixnet_exit_data_si,
            vpn_entry_tickets: ticketbooks.vpn_entry_tickets,
            vpn_entry_data: ticketbooks.vpn_entry_data,
            vpn_entry_data_si: ticketbooks.vpn_entry_data_si,
            vpn_exit_tickets: ticketbooks.vpn_exit_tickets,
            vpn_exit_data: ticketbooks.vpn_exit_data,
            vpn_exit_data_si: ticketbooks.vpn_exit_data_si,
        }
    }
}

impl From<VpnApiError> for proto::VpnApiError {
    fn from(value: VpnApiError) -> Self {
        let error_detail = match value {
            VpnApiError::Timeout(msg) => proto::vpn_api_error::ErrorDetail::Timeout(msg),
            VpnApiError::StatusCode { code, msg: message } => {
                proto::vpn_api_error::ErrorDetail::StatusCode(proto::vpn_api_error::StatusError {
                    code: u32::from(code),
                    message,
                })
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

impl TryFrom<proto::VpnAccountSummary> for VpnAccountSummary {
    type Error = ConversionError;

    fn try_from(value: proto::VpnAccountSummary) -> Result<Self, Self::Error> {
        let subscription_valid_until = value
            .subscription_valid_until
            .map(|ts| {
                prost_timestamp_into_offset_datetime(ts).map_err(|e| {
                    ConversionError::ConvertTime("VpnAccountSummary.subscription_valid_until", e)
                })
            })
            .transpose()?;
        let traffic_reset_time = value
            .traffic_reset_time
            .map(|ts| {
                prost_timestamp_into_offset_datetime(ts).map_err(|e| {
                    ConversionError::ConvertTime("VpnAccountSummary.traffic_reset_time", e)
                })
            })
            .transpose()?;
        Ok(Self {
            subscription_valid_until,
            traffic_used_gb: value.traffic_used_gb,
            traffic_limit_gb: value.traffic_limit_gb,
            traffic_reset_time,
        })
    }
}

impl From<VpnAccountSummary> for proto::VpnAccountSummary {
    fn from(value: VpnAccountSummary) -> Self {
        let subscription_valid_until = value
            .subscription_valid_until
            .map(offset_datetime_into_proto_timestamp);
        let traffic_reset_time = value
            .traffic_reset_time
            .map(offset_datetime_into_proto_timestamp);
        Self {
            subscription_valid_until,
            traffic_used_gb: value.traffic_used_gb,
            traffic_limit_gb: value.traffic_limit_gb,
            traffic_reset_time,
        }
    }
}

impl From<GetDeeplinkParams> for proto::GetDeeplinkParams {
    fn from(value: GetDeeplinkParams) -> Self {
        let client: proto::DeeplinkClient = value.client.into();
        let kind: proto::DeeplinkKind = value.kind.into();
        Self {
            client: client as i32,
            locale: value.locale,
            kind: kind as i32,
            name: value.name,
        }
    }
}

impl TryFrom<proto::GetDeeplinkParams> for GetDeeplinkParams {
    type Error = ConversionError;

    fn try_from(value: proto::GetDeeplinkParams) -> Result<Self, Self::Error> {
        let client = proto::DeeplinkClient::try_from(value.client)
            .map_err(|_| ConversionError::NoValueSet("GetDeeplinkParams.client"))?;

        let kind = proto::DeeplinkKind::try_from(value.kind)
            .map_err(|_| ConversionError::NoValueSet("GetDeeplinkParams.kind"))?;

        Ok(Self {
            client: client.into(),
            locale: value.locale,
            kind: kind.into(),
            name: value.name,
        })
    }
}

impl From<DeeplinkClient> for proto::DeeplinkClient {
    fn from(value: DeeplinkClient) -> Self {
        match value {
            DeeplinkClient::Mobile => proto::DeeplinkClient::Mobile,
            DeeplinkClient::Desktop => proto::DeeplinkClient::Desktop,
            DeeplinkClient::Web => proto::DeeplinkClient::Web,
        }
    }
}

impl From<proto::DeeplinkClient> for DeeplinkClient {
    fn from(value: proto::DeeplinkClient) -> Self {
        match value {
            proto::DeeplinkClient::Mobile => DeeplinkClient::Mobile,
            proto::DeeplinkClient::Desktop => DeeplinkClient::Desktop,
            proto::DeeplinkClient::Web => DeeplinkClient::Web,
        }
    }
}

impl From<DeeplinkKind> for proto::DeeplinkKind {
    fn from(value: DeeplinkKind) -> Self {
        match value {
            DeeplinkKind::Privy => proto::DeeplinkKind::Privy,
            DeeplinkKind::PrivyAssociate => proto::DeeplinkKind::PrivyAssociate,
        }
    }
}

impl From<proto::DeeplinkKind> for DeeplinkKind {
    fn from(value: proto::DeeplinkKind) -> Self {
        match value {
            proto::DeeplinkKind::Privy => DeeplinkKind::Privy,
            proto::DeeplinkKind::PrivyAssociate => DeeplinkKind::PrivyAssociate,
        }
    }
}
