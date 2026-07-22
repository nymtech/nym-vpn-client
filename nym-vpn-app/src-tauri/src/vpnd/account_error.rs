use crate::error::{BackendError, ErrorKey};
use nym_vpn_lib_types as lib;

impl From<lib::AccountCommandError> for BackendError {
    fn from(error: lib::AccountCommandError) -> Self {
        match error {
            lib::AccountCommandError::Internal(e) => {
                BackendError::internal_with_detail("AC internal error", e)
            }
            lib::AccountCommandError::Storage(e) => BackendError::internal_with_detail(
                "AC storage error",
                format!("AC storage error: {e}"),
            ),
            lib::AccountCommandError::VpnApi(error) => error.into(),
            lib::AccountCommandError::UnexpectedVpnApiResponse(e) => {
                BackendError::internal_with_detail(
                    "AC unexpected response",
                    format!("AC unexpected response: {e}"),
                )
            }
            lib::AccountCommandError::NoAccountStored => {
                BackendError::new("AC no account stored", ErrorKey::NoAccountStored)
            }
            lib::AccountCommandError::NoDeviceStored => {
                BackendError::new("AC no device stored", ErrorKey::NoDeviceStored)
            }
            lib::AccountCommandError::ExistingAccount => {
                BackendError::new("AC account already exists", ErrorKey::ExistingAccount)
            }
            lib::AccountCommandError::Offline => BackendError::internal("AC is offline", None),
            lib::AccountCommandError::InvalidMnemonic(e) => BackendError::with_detail(
                "invalid passphrase",
                ErrorKey::AccountInvalidMnemonic,
                format!("invalid passphrase: {e}"),
            ),
            lib::AccountCommandError::InvalidSecret(e) => BackendError::with_detail(
                "invalid secret",
                ErrorKey::AccountInvalidSecret,
                format!("invalid secret: {e}"),
            ),
            lib::AccountCommandError::NyxdConnectionFailure(e) => {
                BackendError::internal_with_detail("failed to connect to nyxd", e)
            }
            lib::AccountCommandError::NyxdQueryFailure(e) => {
                BackendError::internal_with_detail("failed to resolve query to a nyxd instance", e)
            }
            lib::AccountCommandError::AccountDoesntExistOnChain => {
                BackendError::internal("account doesn't exist on chain", None)
            }
            lib::AccountCommandError::AccountDecentralised => {
                BackendError::internal("account is set in decentralised mode", None)
            }
            lib::AccountCommandError::AccountNotDecentralised => {
                BackendError::internal("account is not set in decentralised mode", None)
            }
            lib::AccountCommandError::ZkNymAcquisitionFailure(e) => {
                BackendError::internal_with_detail("failed to obtain zk-nym", e)
            }
            lib::AccountCommandError::DeeplinkError(message) => {
                BackendError::internal_with_detail("deeplink error", message)
            }
            lib::AccountCommandError::InsufficientFunds => {
                BackendError::new("insufficient funds", ErrorKey::InsufficientFunds)
            }
        }
    }
}

impl From<lib::AccountControllerErrorStateReason> for BackendError {
    fn from(error: lib::AccountControllerErrorStateReason) -> Self {
        match error {
            lib::AccountControllerErrorStateReason::Storage { context, details } => {
                BackendError::internal_with_detail(
                    "AC storage error",
                    format!("{} - {}", context, details),
                )
            }
            lib::AccountControllerErrorStateReason::ApiFailure { context, details } => {
                BackendError::internal_with_detail(
                    "AC API failure",
                    format!("{} - {}", context, details),
                )
            }
            lib::AccountControllerErrorStateReason::Internal { context, details } => {
                BackendError::internal_with_detail(
                    "AC internal error",
                    format!("{} - {}", context, details),
                )
            }
            lib::AccountControllerErrorStateReason::BandwidthExceeded { context } => {
                BackendError::with_detail(
                    "AC bandwidth exceeded",
                    ErrorKey::BandwidthExceeded,
                    context,
                )
            }
            lib::AccountControllerErrorStateReason::AccountStatusNotActive { status } => {
                BackendError::with_detail(
                    "AC account status not active",
                    ErrorKey::AccountStatusNotActive,
                    status,
                )
            }
            lib::AccountControllerErrorStateReason::InactiveSubscription => {
                BackendError::new("AC inactive subscription", ErrorKey::NoSubscription)
            }
            lib::AccountControllerErrorStateReason::MaxDeviceReached => {
                BackendError::new("AC max devices reached", ErrorKey::MaxDeviceReached)
            }
            lib::AccountControllerErrorStateReason::DeviceTimeDesynced => {
                BackendError::new("AC device time desynced", ErrorKey::DeviceTimeDesync)
            }
        }
    }
}

impl From<lib::VpnApiError> for BackendError {
    fn from(error: lib::VpnApiError) -> Self {
        match error {
            lib::VpnApiError::Timeout(t) => {
                BackendError::internal(&format!("VPN API timeout: {t}"), None)
            }
            lib::VpnApiError::StatusCode { code, msg } => BackendError::internal(
                &format!("VPN API error, status code: {code}, error: {msg}"),
                None,
            ),
            lib::VpnApiError::Response(response) => BackendError::from(response),
        }
    }
}

impl From<lib::VpnApiErrorResponse> for BackendError {
    fn from(error: lib::VpnApiErrorResponse) -> Self {
        let mut detail = format!("VPN API response error: {}", error.message);
        if let Some(code) = error.message_id {
            detail.push_str(&format!(" (id: {code})"));
        }
        if let Some(id) = error.code_reference_id {
            detail.push_str(&format!(" (code: {id})"));
        }
        BackendError::internal_with_detail("VPN API response error", detail)
    }
}
