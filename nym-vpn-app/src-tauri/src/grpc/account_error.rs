use crate::error::{BackendError, ErrorKey};
use nym_vpn_proto::proto::{
    AccountCommandError, VpnApiError, VpnApiErrorResponse,
    account_command_error::ErrorDetail as AccountError,
    vpn_api_error::ErrorDetail as VpnApiErrorDetail,
};
use tracing::error;

impl From<VpnApiError> for BackendError {
    fn from(error: VpnApiError) -> Self {
        let Some(detail) = error.error_detail else {
            error!("missing error detail in VpnApiError");
            return BackendError::internal("nym-vpn-api returned error", None);
        };
        match detail {
            VpnApiErrorDetail::Timeout(_) => BackendError::internal("nym-vpn-api timeout", None),
            VpnApiErrorDetail::StatusCode(code) => BackendError::internal_with_detail(
                "nym-vpn-api error",
                format!("nym-vpn-api returned: {code}"),
            ),
            VpnApiErrorDetail::Response(response) => BackendError::from(response),
        }
    }
}

impl From<VpnApiErrorResponse> for BackendError {
    fn from(error: VpnApiErrorResponse) -> Self {
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

impl From<AccountCommandError> for BackendError {
    fn from(error: AccountCommandError) -> Self {
        let Some(detail) = error.error_detail else {
            error!("missing error detail in AccountCommandError");
            return BackendError::internal_with_detail(
                "failed to run account command",
                "failed to run account command".to_string(),
            );
        };
        match detail {
            AccountError::Internal(data) => {
                BackendError::internal_with_detail("internal error", data)
            }
            AccountError::StorageError(data) => BackendError::internal_with_detail(
                "storage error",
                format!("storage error: {data}"),
            ),
            AccountError::VpnApi(error) => error.into(),
            AccountError::UnexpectedResponse(data) => BackendError::internal_with_detail(
                "unexpected response",
                format!("unexpected response: {data}"),
            ),
            AccountError::NoAccountStored(data) => BackendError::with_detail(
                "no account stored",
                ErrorKey::NoAccountStored,
                format!("no account stored : {data}"),
            ),
            AccountError::NoDeviceStored(data) => BackendError::with_detail(
                "no device stored",
                ErrorKey::NoDeviceStored,
                format!("no device stored : {data}"),
            ),
            AccountError::ExistingAccount(data) => BackendError::with_detail(
                "account already exists",
                ErrorKey::ExistingAccount,
                format!("account already exists : {data}"),
            ),
            AccountError::Offline(data) => BackendError::with_detail(
                "account controller is offline",
                ErrorKey::AccountControllerOffline,
                format!("account controller is offline : {data}"),
            ),
            AccountError::InvalidMnemonic(data) => BackendError::with_detail(
                "invalid mnemonic",
                ErrorKey::AccountInvalidMnemonic,
                format!("invalid mnemonic: {data}"),
            ),
        }
    }
}
