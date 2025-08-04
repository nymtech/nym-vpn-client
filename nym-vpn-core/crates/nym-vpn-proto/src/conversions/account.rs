// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_lib_types::{
    AvailableTickets, ForgetAccountError, RegisterDeviceError, RequestZkNymErrorReason,
    RequestZkNymSuccess, StoreAccountError, SyncAccountError, SyncDeviceError, VpnApiError,
    VpnApiErrorResponse,
};

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::StoreAccountError> for StoreAccountError {
    type Error = ConversionError;

    fn try_from(value: proto::StoreAccountError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "StoreAccountError.error_detail",
        ))?;
        Ok(match error_detail {
            proto::store_account_error::ErrorDetail::InvalidMnemonic(message) => {
                Self::InvalidMnemonic(message)
            }
            proto::store_account_error::ErrorDetail::StorageError(err) => Self::Storage(err),
            proto::store_account_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::GetAccountEndpointFailure(vpn_api.try_into()?)
            }
            proto::store_account_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            proto::store_account_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<proto::SyncAccountError> for SyncAccountError {
    type Error = ConversionError;

    fn try_from(value: proto::SyncAccountError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("SyncAccountError.error_detail"))?;
        Ok(match error_detail {
            proto::sync_account_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            proto::sync_account_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::SyncAccountEndpointFailure(vpn_api.try_into()?)
            }
            proto::sync_account_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            proto::sync_account_error::ErrorDetail::Offline(_) => Self::Offline,
            proto::sync_account_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<proto::SyncDeviceError> for SyncDeviceError {
    type Error = ConversionError;

    fn try_from(value: proto::SyncDeviceError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("SyncDeviceError.error_detail"))?;
        Ok(match error_detail {
            proto::sync_device_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            proto::sync_device_error::ErrorDetail::NoDeviceStored(_) => Self::NoDeviceStored,
            proto::sync_device_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::SyncDeviceEndpointFailure(vpn_api.try_into()?)
            }
            proto::sync_device_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            proto::sync_device_error::ErrorDetail::Offline(_) => Self::Offline,
            proto::sync_device_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<proto::RegisterDeviceError> for RegisterDeviceError {
    type Error = ConversionError;

    fn try_from(value: proto::RegisterDeviceError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "RegisterDeviceError.error_detail",
        ))?;
        Ok(match error_detail {
            proto::register_device_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            proto::register_device_error::ErrorDetail::NoDeviceStored(_) => Self::NoDeviceStored,
            proto::register_device_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::RegisterDeviceEndpointFailure(vpn_api.try_into()?)
            }
            proto::register_device_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            proto::register_device_error::ErrorDetail::Offline(_) => Self::Offline,
            proto::register_device_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<proto::ForgetAccountError> for ForgetAccountError {
    type Error = ConversionError;

    fn try_from(value: proto::ForgetAccountError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "ForgetAccountError.error_detail",
        ))?;
        Ok(match error_detail {
            proto::forget_account_error::ErrorDetail::RegistrationInProgress(_) => {
                Self::RegistrationInProgress
            }
            proto::forget_account_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::UpdateDeviceErrorResponse(vpn_api.try_into()?)
            }
            proto::forget_account_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            proto::forget_account_error::ErrorDetail::RemoveAccount(err) => {
                Self::RemoveAccount(err)
            }
            proto::forget_account_error::ErrorDetail::RemoveDeviceKeys(err) => {
                Self::RemoveDeviceKeys(err)
            }
            proto::forget_account_error::ErrorDetail::ResetCredentialStore(err) => {
                Self::ResetCredentialStorage(err)
            }
            proto::forget_account_error::ErrorDetail::RemoveAccountFiles(err) => {
                Self::RemoveAccountFiles(err)
            }
            proto::forget_account_error::ErrorDetail::InitDeviceKeys(err) => {
                Self::InitDeviceKeys(err)
            }
            proto::forget_account_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

// We don't pass the source error across grpc, so on the recipient it will be empty. That's OK.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("(empty)")]
struct EmptyError;

impl TryFrom<proto::VpnApiError> for VpnApiError {
    type Error = ConversionError;

    fn try_from(value: proto::VpnApiError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("VpnApiError.error_detail"))?;
        Ok(match error_detail {
            proto::vpn_api_error::ErrorDetail::Timeout(_) => Self::Timeout(Arc::new(EmptyError)),
            proto::vpn_api_error::ErrorDetail::StatusCode(code) => Self::StatusCode {
                code: code.try_into().map_err(ConversionError::generic)?,
                source: Arc::new(EmptyError),
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

impl From<StoreAccountError> for proto::StoreAccountError {
    fn from(value: StoreAccountError) -> Self {
        match value {
            StoreAccountError::InvalidMnemonic(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::InvalidMnemonic(
                    err,
                )),
            },
            StoreAccountError::Storage(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::StorageError(err)),
            },
            StoreAccountError::GetAccountEndpointFailure(vpn_api) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::VpnApi(
                    vpn_api.into(),
                )),
            },
            StoreAccountError::UnexpectedResponse(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::UnexpectedResponse(
                    err,
                )),
            },
            StoreAccountError::Internal(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::Internal(err)),
            },
        }
    }
}

impl From<RequestZkNymSuccess> for proto::RequestZkNymSuccess {
    fn from(value: RequestZkNymSuccess) -> Self {
        Self {
            id: value.id,
            ticketbook_type: value.ticketbook_type,
        }
    }
}

impl From<proto::RequestZkNymSuccess> for RequestZkNymSuccess {
    fn from(value: proto::RequestZkNymSuccess) -> Self {
        Self {
            id: value.id,
            ticketbook_type: value.ticketbook_type,
        }
    }
}

impl From<RequestZkNymErrorReason> for proto::RequestZkNymError {
    fn from(error: RequestZkNymErrorReason) -> Self {
        let outcome = match error {
            RequestZkNymErrorReason::VpnApi(vpn_api_endpoint_failure) => {
                proto::request_zk_nym_error::Outcome::VpnApi(vpn_api_endpoint_failure.into())
            }
            RequestZkNymErrorReason::UnexpectedVpnApiResponse(err) => {
                proto::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(err)
            }
            RequestZkNymErrorReason::Storage(err) => {
                proto::request_zk_nym_error::Outcome::Storage(err)
            }
            RequestZkNymErrorReason::Internal(err) => {
                proto::request_zk_nym_error::Outcome::Internal(err)
            }
        };
        Self {
            outcome: Some(outcome),
        }
    }
}

impl TryFrom<proto::RequestZkNymError> for RequestZkNymErrorReason {
    type Error = ConversionError;

    fn try_from(value: proto::RequestZkNymError) -> Result<Self, Self::Error> {
        let error_outcome = value
            .outcome
            .ok_or(ConversionError::NoValueSet("RequestZkNymError.outcome"))?;

        Ok(match error_outcome {
            proto::request_zk_nym_error::Outcome::VpnApi(vpn_api) => {
                Self::VpnApi(vpn_api.try_into()?)
            }
            proto::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(message) => {
                Self::UnexpectedVpnApiResponse(message)
            }
            proto::request_zk_nym_error::Outcome::Storage(message) => Self::Storage(message),
            proto::request_zk_nym_error::Outcome::Internal(message) => Self::Internal(message),
        })
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
