use nym_vpn_proto as p;
use nym_vpn_proto::VpnApiErrorResponse;
use p::register_device_error::ErrorDetail as RegDeviceErr;
use p::request_zk_nym_error::Outcome as ZkNymErr;
use p::sync_account_error::ErrorDetail as SyncAccountErr;
use p::sync_device_error::ErrorDetail as SyncDeviceErr;
use p::tunnel_state::error::ErrorStateReason;
use p::tunnel_state::BaseErrorStateReason;
use p::{
    RegisterDeviceError, RequestZkNymBundle, RequestZkNymError, SyncAccountError, SyncDeviceError,
};
use serde::Serialize;
use std::fmt;
use strum::Display;
use tracing::{error, warn};
use ts_rs::TS;

#[derive(Serialize, Clone, Debug, Display, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "key", content = "data")]
pub enum TunnelError {
    Internal,
    Firewall,
    Routing,
    Dns,
    TunDevice,
    TunnelProvider,
    SameEntryAndExitGw,
    InvalidEntryGwCountry,
    InvalidExitGwCountry,
    BadBandwidthIncrease,
    DuplicateTunFd,
    // SyncAccountError mapping
    SyncAccountNoAccountStored(bool),
    SyncAccountUnexpectedResponse(String),
    SyncAccountInternal(String),
    SyncAccountVpnApi(String),
    // SyncDeviceError mapping
    SyncDeviceNoAccountStored(bool),
    SyncDeviceNoDeviceStored(bool),
    SyncDeviceUnexpectedResponse(String),
    SyncDeviceInternal(String),
    SyncDeviceVpnApi(String),
    // RegisterDeviceError mapping
    RegisterDeviceNoAccountStored(bool),
    RegisterDeviceNoDeviceStored(bool),
    RegisterDeviceUnexpectedResponse(String),
    RegisterDeviceInternal(String),
    RegisterDeviceVpnApi(String),
    // RequestZkNymError mapping
    ReqZknymNoAccountStored(bool),
    ReqZknymNoDeviceStored(bool),
    ReqZknymUnexpectedResponse(String),
    ReqZknymStorage(String),
    ReqZknymInternal(String),
    ReqZknymVpnApi(String),
}

impl From<Option<ErrorStateReason>> for TunnelError {
    fn from(reason: Option<ErrorStateReason>) -> Self {
        let Some(error) = reason else {
            warn!("missing error reason in TunnelError");
            return TunnelError::Internal;
        };
        match error {
            ErrorStateReason::BaseReason(e) => BaseErrorStateReason::try_from(e)
                .inspect_err(|e| error!("failed to convert to BaseErrorStateReason: {}", e))
                .unwrap_or(BaseErrorStateReason::Internal)
                .into(),
            ErrorStateReason::SyncAccount(e) => e.into(),
            ErrorStateReason::SyncDevice(e) => e.into(),
            ErrorStateReason::RegisterDevice(e) => e.into(),
            ErrorStateReason::RequestZkNym(e) => e.into(),
            ErrorStateReason::RequestZkNymBundle(e) => e.into(),
        }
    }
}

impl From<BaseErrorStateReason> for TunnelError {
    fn from(reason: BaseErrorStateReason) -> Self {
        match reason {
            BaseErrorStateReason::Internal => TunnelError::Internal,
            BaseErrorStateReason::Firewall => TunnelError::Firewall,
            BaseErrorStateReason::Routing => TunnelError::Routing,
            BaseErrorStateReason::Dns => TunnelError::Dns,
            BaseErrorStateReason::TunDevice => TunnelError::TunDevice,
            BaseErrorStateReason::TunnelProvider => TunnelError::TunnelProvider,
            BaseErrorStateReason::SameEntryAndExitGateway => TunnelError::SameEntryAndExitGw,
            BaseErrorStateReason::InvalidEntryGatewayCountry => TunnelError::InvalidEntryGwCountry,
            BaseErrorStateReason::InvalidExitGatewayCountry => TunnelError::InvalidExitGwCountry,
            BaseErrorStateReason::BadBandwidthIncrease => TunnelError::BadBandwidthIncrease,
            BaseErrorStateReason::DuplicateTunFd => TunnelError::DuplicateTunFd,
        }
    }
}

impl From<SyncAccountError> for TunnelError {
    fn from(error: SyncAccountError) -> Self {
        let Some(e) = error.error_detail else {
            warn!("missing error detail in SyncAccountError");
            return TunnelError::Internal;
        };
        match e {
            SyncAccountErr::NoAccountStored(b) => TunnelError::SyncAccountNoAccountStored(b),
            SyncAccountErr::UnexpectedResponse(s) => TunnelError::SyncAccountUnexpectedResponse(s),
            SyncAccountErr::Internal(s) => TunnelError::SyncAccountInternal(s),
            SyncAccountErr::ErrorResponse(res) => {
                TunnelError::SyncAccountVpnApi(VpnApiError(res).to_string())
            }
        }
    }
}

impl From<SyncDeviceError> for TunnelError {
    fn from(error: SyncDeviceError) -> Self {
        let Some(e) = error.error_detail else {
            warn!("missing error detail in SyncDeviceError");
            return TunnelError::Internal;
        };
        match e {
            SyncDeviceErr::NoAccountStored(b) => TunnelError::SyncDeviceNoAccountStored(b),
            SyncDeviceErr::NoDeviceStored(b) => TunnelError::SyncDeviceNoDeviceStored(b),
            SyncDeviceErr::UnexpectedResponse(s) => TunnelError::SyncDeviceUnexpectedResponse(s),
            SyncDeviceErr::Internal(s) => TunnelError::SyncDeviceInternal(s),
            SyncDeviceErr::ErrorResponse(res) => {
                TunnelError::SyncDeviceVpnApi(VpnApiError(res).to_string())
            }
        }
    }
}

impl From<RegisterDeviceError> for TunnelError {
    fn from(error: RegisterDeviceError) -> Self {
        let Some(e) = error.error_detail else {
            warn!("missing error detail in RegisterDeviceError");
            return TunnelError::Internal;
        };
        match e {
            RegDeviceErr::NoAccountStored(b) => TunnelError::RegisterDeviceNoAccountStored(b),
            RegDeviceErr::NoDeviceStored(b) => TunnelError::RegisterDeviceNoDeviceStored(b),
            RegDeviceErr::UnexpectedResponse(s) => TunnelError::RegisterDeviceUnexpectedResponse(s),
            RegDeviceErr::Internal(s) => TunnelError::RegisterDeviceInternal(s),
            RegDeviceErr::ErrorResponse(res) => {
                TunnelError::RegisterDeviceVpnApi(VpnApiError(res).to_string())
            }
        }
    }
}

impl From<RequestZkNymError> for TunnelError {
    fn from(error: RequestZkNymError) -> Self {
        let Some(e) = error.outcome else {
            warn!("missing error detail in RequestZkNymError");
            return TunnelError::Internal;
        };
        match e {
            ZkNymErr::NoAccountStored(b) => TunnelError::ReqZknymNoAccountStored(b),
            ZkNymErr::NoDeviceStored(b) => TunnelError::ReqZknymNoDeviceStored(b),
            ZkNymErr::UnexpectedVpnApiResponse(s) => TunnelError::ReqZknymUnexpectedResponse(s),
            ZkNymErr::Storage(s) => TunnelError::ReqZknymStorage(s),
            ZkNymErr::Internal(s) => TunnelError::ReqZknymInternal(s),
            ZkNymErr::VpnApi(res) => TunnelError::ReqZknymVpnApi(VpnApiError(res).to_string()),
        }
    }
}

impl From<RequestZkNymBundle> for TunnelError {
    fn from(bundle: RequestZkNymBundle) -> Self {
        // TODO how the heck client is expected to deal with that kind of API??

        if bundle.failures.is_empty() {
            warn!("no failure found in RequestZkNymBundle");
            return TunnelError::Internal;
        }
        // let's suppose: get the first one and we good? ><
        bundle.failures[0].clone().into()
    }
}

struct VpnApiError(VpnApiErrorResponse);

impl fmt::Display for VpnApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.0.message_id {
            write!(f, "{}, ID [{}]", self.0.message, id)
        } else {
            write!(f, "{}", self.0.message)
        }
    }
}
