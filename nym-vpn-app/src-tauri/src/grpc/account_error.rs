use crate::error::{BackendError, ErrorKey};
use nym_vpn_proto::proto::{
    AccountCommandError, VpnApiError, VpnApiErrorResponse,
    account_command_error::ErrorDetail as AccountError,
    account_controller_state::Error as StateError, account_controller_state::ErrorStateReason,
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
                format!("nym-vpn-api returned: {code:?}"),
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
            return BackendError::internal("failed to run account command", None);
        };
        match detail {
            AccountError::Internal(e) => BackendError::internal_with_detail("AC internal error", e),
            AccountError::StorageError(e) => BackendError::internal_with_detail(
                "AC storage error",
                format!("AC storage error: {e}"),
            ),
            AccountError::VpnApi(error) => error.into(),
            AccountError::UnexpectedResponse(e) => BackendError::internal_with_detail(
                "AC unexpected response",
                format!("AC unexpected response: {e}"),
            ),
            AccountError::NoAccountStored(v) => BackendError::with_detail(
                "AC no account stored",
                ErrorKey::NoAccountStored,
                format!("AC no account stored : {v}"),
            ),
            AccountError::NoDeviceStored(v) => BackendError::with_detail(
                "AC no device stored",
                ErrorKey::NoDeviceStored,
                format!("AC no device stored : {v}"),
            ),
            AccountError::ExistingAccount(v) => BackendError::with_detail(
                "AC account already exists",
                ErrorKey::ExistingAccount,
                format!("AC account already exists : {v}"),
            ),
            AccountError::Offline(v) => {
                BackendError::internal_with_detail("AC is offline", format!("AC is offline : {v}"))
            }
            AccountError::InvalidMnemonic(e) => BackendError::with_detail(
                "invalid mnemonic",
                ErrorKey::AccountInvalidMnemonic,
                format!("invalid mnemonic: {e}"),
            ),
            AccountError::NyxdConnectionFailure(e) => {
                BackendError::internal_with_detail("failed to connect to nyxd", e)
            }
            AccountError::NyxdQueryFailure(e) => {
                BackendError::internal_with_detail("failed to resolve query to a nyxd instance", e)
            }
            AccountError::AccountDoesntExistOnChain(v) => BackendError::internal_with_detail(
                "account doesn't exist on chain",
                format!("account doesn't exist on chain: {v}"),
            ),
            AccountError::InsufficientFunds(v) => BackendError::internal_with_detail(
                "account does not have sufficient funds",
                format!("account does not have sufficient funds: {v}"),
            ),
            AccountError::AccountDecentralised(v) => BackendError::internal_with_detail(
                "account is set in decentralised mode",
                format!("account is set in decentralised mode: {v}"),
            ),
            AccountError::AccountNotDecentralised(v) => BackendError::internal_with_detail(
                "account is not set in decentralised mode",
                format!("account is not set in decentralised mode: {v}"),
            ),
            AccountError::ZkNymAcquisitionFailure(e) => {
                BackendError::internal_with_detail("failed to obtain zk-nym", e)
            }
        }
    }
}

impl From<StateError> for BackendError {
    fn from(error: StateError) -> Self {
        match error.reason() {
            ErrorStateReason::Internal => BackendError::internal("AC internal", None),
            ErrorStateReason::Storage => BackendError::internal("AC storage", None),
            ErrorStateReason::ApiFailure => BackendError::internal("AC api failure", None),
            ErrorStateReason::BandwidthExceeded => {
                BackendError::new("AC bandwidth exceeded", ErrorKey::BandwidthExceeded)
            }
            ErrorStateReason::AccountStatusNotActive => {
                BackendError::new("AC status not active", ErrorKey::AccountStatusNotActive)
            }
            ErrorStateReason::InactiveSubscription => {
                BackendError::new("AC inactive subscription", ErrorKey::NoSubscription)
            }
            ErrorStateReason::MaxDeviceReached => {
                BackendError::new("AC max device reached", ErrorKey::MaxDeviceReached)
            }
            ErrorStateReason::DeviceTimeDesynced => {
                BackendError::new("AC device time desynced", ErrorKey::DeviceTimeDesync)
            }
        }
    }
}
