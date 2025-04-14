use crate::error::{BackendError, ErrorKey};
use nym_vpn_proto::forget_account_error::ErrorDetail as ForgetError;
use nym_vpn_proto::store_account_error::ErrorDetail as StoreError;
use nym_vpn_proto::{ForgetAccountError, StoreAccountError, VpnApiErrorResponse};
use tracing::error;

impl From<VpnApiErrorResponse> for BackendError {
    fn from(error: VpnApiErrorResponse) -> Self {
        let mut detail = format!("VPN API response error: {}", error.message);
        if let Some(code) = error.message_id {
            detail.push_str(&format!(" (id: {})", code));
        }
        if let Some(id) = error.code_reference_id {
            detail.push_str(&format!(" (code: {})", id));
        }
        BackendError::internal_with_detail("VPN API response error", detail)
    }
}

impl From<StoreAccountError> for BackendError {
    fn from(error: StoreAccountError) -> Self {
        let Some(detail) = error.error_detail else {
            error!("missing error detail in StoreAccountError");
            return BackendError::internal_with_detail(
                "failed to store account",
                "failed to store account".to_string(),
            );
        };
        match detail {
            StoreError::InvalidMnemonic(data) => BackendError::with_detail(
                "invalid mnemonic",
                ErrorKey::AccountInvalidMnemonic,
                format!("invalid mnemonic: {}", data),
            ),
            StoreError::StorageError(data) => BackendError::internal_with_detail(
                "storage error",
                format!("storage error: {}", data),
            ),
            StoreError::ErrorResponse(error) => error.into(),
            StoreError::UnexpectedResponse(data) => BackendError::internal_with_detail(
                "unexpected response",
                format!("unexpected response: {}", data),
            ),
            StoreError::Internal(data) => {
                BackendError::internal_with_detail("internal error", data)
            }
        }
    }
}

impl From<ForgetAccountError> for BackendError {
    fn from(error: ForgetAccountError) -> Self {
        let Some(detail) = error.error_detail else {
            error!("missing error detail in ForgetAccountError");
            return BackendError::internal_with_detail(
                "failed to forget account",
                "failed to forget account".to_string(),
            );
        };
        match detail {
            ForgetError::RegistrationInProgress(v) => match v {
                true => BackendError::internal_with_detail(
                    "registration in progress",
                    "registration in progress".to_string(),
                ),
                // is it even possible?
                false => BackendError::internal_with_detail(
                    "registration not in progress",
                    "registration not in progress".to_string(),
                ),
            },
            ForgetError::ErrorResponse(error) => error.into(),
            ForgetError::UnexpectedResponse(data) => BackendError::internal_with_detail(
                "unexpected response",
                format!("unexpected response: {}", data),
            ),
            ForgetError::RemoveAccount(data) => BackendError::internal_with_detail(
                "remove account",
                format!("remove account: {}", data),
            ),
            ForgetError::RemoveDeviceKeys(data) => BackendError::internal_with_detail(
                "remove device keys",
                format!("remove device keys: {}", data),
            ),
            ForgetError::ResetCredentialStore(data) => BackendError::internal_with_detail(
                "reset credential store",
                format!("reset credential store: {}", data),
            ),
            ForgetError::RemoveAccountFiles(data) => BackendError::internal_with_detail(
                "remove account files",
                format!("remove account files: {}", data),
            ),
            ForgetError::InitDeviceKeys(data) => BackendError::internal_with_detail(
                "init device keys",
                format!("init device keys: {}", data),
            ),
            ForgetError::Internal(data) => {
                BackendError::internal_with_detail("internal error", data)
            }
        }
    }
}
