// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_lib_types::{
    ForgetAccountError, RegisterDeviceError, RequestZkNymErrorReason, RequestZkNymSuccess,
    StoreAccountError, SyncAccountError, SyncDeviceError, VpnApiError, VpnApiErrorResponse,
};

use crate::{
    ForgetAccountError as ProtoForgetAccountError, RegisterDeviceError as ProtoRegisterDeviceError,
    RequestZkNymError as ProtoRequestZkNymError, RequestZkNymSuccess as ProtoRequestZkNymSuccess,
    StoreAccountError as ProtoStoreAccountError, SyncAccountError as ProtoSyncAccountError,
    SyncDeviceError as ProtoSyncDeviceError, VpnApiError as ProtoVpnApiError,
    VpnApiErrorResponse as ProtoVpnApiErrorResponse, conversions::ConversionError,
};

impl TryFrom<ProtoStoreAccountError> for StoreAccountError {
    type Error = ConversionError;

    fn try_from(value: ProtoStoreAccountError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "StoreAccountError.error_detail",
        ))?;
        Ok(match error_detail {
            crate::store_account_error::ErrorDetail::InvalidMnemonic(message) => {
                Self::InvalidMnemonic(message)
            }
            crate::store_account_error::ErrorDetail::StorageError(err) => Self::Storage(err),
            crate::store_account_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::GetAccountEndpointFailure(vpn_api.try_into()?)
            }
            crate::store_account_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            crate::store_account_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<ProtoSyncAccountError> for SyncAccountError {
    type Error = ConversionError;

    fn try_from(value: ProtoSyncAccountError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("SyncAccountError.error_detail"))?;
        Ok(match error_detail {
            crate::sync_account_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            crate::sync_account_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::SyncAccountEndpointFailure(vpn_api.try_into()?)
            }
            crate::sync_account_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            crate::sync_account_error::ErrorDetail::Offline(_) => Self::Offline,
            crate::sync_account_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<ProtoSyncDeviceError> for SyncDeviceError {
    type Error = ConversionError;

    fn try_from(value: ProtoSyncDeviceError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("SyncDeviceError.error_detail"))?;
        Ok(match error_detail {
            crate::sync_device_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            crate::sync_device_error::ErrorDetail::NoDeviceStored(_) => Self::NoDeviceStored,
            crate::sync_device_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::SyncDeviceEndpointFailure(vpn_api.try_into()?)
            }
            crate::sync_device_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            crate::sync_device_error::ErrorDetail::Offline(_) => Self::Offline,
            crate::sync_device_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl TryFrom<ProtoRegisterDeviceError> for RegisterDeviceError {
    type Error = ConversionError;

    fn try_from(value: ProtoRegisterDeviceError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "RegisterDeviceError.error_detail",
        ))?;
        Ok(match error_detail {
            crate::register_device_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            crate::register_device_error::ErrorDetail::NoDeviceStored(_) => Self::NoDeviceStored,
            crate::register_device_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::RegisterDeviceEndpointFailure(vpn_api.try_into()?)
            }
            crate::register_device_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            crate::register_device_error::ErrorDetail::Offline(_) => Self::Offline,
            crate::register_device_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

impl From<ProtoRequestZkNymSuccess> for RequestZkNymSuccess {
    fn from(value: ProtoRequestZkNymSuccess) -> Self {
        Self { id: value.id }
    }
}

impl TryFrom<ProtoRequestZkNymError> for RequestZkNymErrorReason {
    type Error = ConversionError;

    fn try_from(value: ProtoRequestZkNymError) -> Result<Self, Self::Error> {
        let error_outcome = value
            .outcome
            .ok_or(ConversionError::NoValueSet("RequestZkNymError.outcome"))?;

        Ok(match error_outcome {
            crate::request_zk_nym_error::Outcome::NoAccountStored(_) => Self::NoAccountStored,
            crate::request_zk_nym_error::Outcome::NoDeviceStored(_) => Self::NoDeviceStored,
            crate::request_zk_nym_error::Outcome::VpnApi(vpn_api) => {
                Self::VpnApi(vpn_api.try_into()?)
            }
            crate::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(message) => {
                Self::UnexpectedVpnApiResponse(message)
            }
            crate::request_zk_nym_error::Outcome::Storage(message) => Self::Storage(message),
            crate::request_zk_nym_error::Outcome::Offline(_) => Self::Offline,
            crate::request_zk_nym_error::Outcome::Internal(message) => Self::Internal(message),
        })
    }
}

impl TryFrom<ProtoForgetAccountError> for ForgetAccountError {
    type Error = ConversionError;

    fn try_from(value: ProtoForgetAccountError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "ForgetAccountError.error_detail",
        ))?;
        Ok(match error_detail {
            crate::forget_account_error::ErrorDetail::RegistrationInProgress(_) => {
                Self::RegistrationInProgress
            }
            crate::forget_account_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::UpdateDeviceErrorResponse(vpn_api.try_into()?)
            }
            crate::forget_account_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedResponse(err)
            }
            crate::forget_account_error::ErrorDetail::RemoveAccount(err) => {
                Self::RemoveAccount(err)
            }
            crate::forget_account_error::ErrorDetail::RemoveDeviceKeys(err) => {
                Self::RemoveDeviceKeys(err)
            }
            crate::forget_account_error::ErrorDetail::ResetCredentialStore(err) => {
                Self::ResetCredentialStorage(err)
            }
            crate::forget_account_error::ErrorDetail::RemoveAccountFiles(err) => {
                Self::RemoveAccountFiles(err)
            }
            crate::forget_account_error::ErrorDetail::InitDeviceKeys(err) => {
                Self::InitDeviceKeys(err)
            }
            crate::forget_account_error::ErrorDetail::Internal(err) => Self::Internal(err),
        })
    }
}

// We don't pass the source error across grpc, so on the recipient it will be empty. That's OK.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("(empty)")]
struct EmptyError;

impl TryFrom<ProtoVpnApiError> for VpnApiError {
    type Error = ConversionError;

    fn try_from(value: ProtoVpnApiError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("VpnApiError.error_detail"))?;
        Ok(match error_detail {
            crate::vpn_api_error::ErrorDetail::Timeout(_) => Self::Timeout(Arc::new(EmptyError)),
            crate::vpn_api_error::ErrorDetail::StatusCode(code) => Self::StatusCode {
                code: code.try_into().map_err(ConversionError::generic)?,
                source: Arc::new(EmptyError),
            },
            crate::vpn_api_error::ErrorDetail::Response(vpn_api_error_response) => {
                Self::Response(vpn_api_error_response.into())
            }
        })
    }
}

impl From<ProtoVpnApiErrorResponse> for VpnApiErrorResponse {
    fn from(value: ProtoVpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}
