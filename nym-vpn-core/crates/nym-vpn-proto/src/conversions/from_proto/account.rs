// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_lib_types::{
    ForgetAccountError, RegisterDeviceError, RequestZkNymErrorReason, RequestZkNymSuccess,
    StoreAccountError, SyncAccountError, SyncDeviceError, VpnApiError, VpnApiErrorResponse,
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

impl From<proto::RequestZkNymSuccess> for RequestZkNymSuccess {
    fn from(value: proto::RequestZkNymSuccess) -> Self {
        Self { id: value.id }
    }
}

impl TryFrom<proto::RequestZkNymError> for RequestZkNymErrorReason {
    type Error = ConversionError;

    fn try_from(value: proto::RequestZkNymError) -> Result<Self, Self::Error> {
        let error_outcome = value
            .outcome
            .ok_or(ConversionError::NoValueSet("RequestZkNymError.outcome"))?;

        Ok(match error_outcome {
            proto::request_zk_nym_error::Outcome::NoAccountStored(_) => Self::NoAccountStored,
            proto::request_zk_nym_error::Outcome::NoDeviceStored(_) => Self::NoDeviceStored,
            proto::request_zk_nym_error::Outcome::VpnApi(vpn_api) => {
                Self::VpnApi(vpn_api.try_into()?)
            }
            proto::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(message) => {
                Self::UnexpectedVpnApiResponse(message)
            }
            proto::request_zk_nym_error::Outcome::Storage(message) => Self::Storage(message),
            proto::request_zk_nym_error::Outcome::Offline(_) => Self::Offline,
            proto::request_zk_nym_error::Outcome::Internal(message) => Self::Internal(message),
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
