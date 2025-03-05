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
    Internal(Option<String>),
    Firewall,
    Routing,
    Dns,
    SameEntryAndExitGw,
    InvalidEntryGwCountry,
    InvalidExitGwCountry,
    Api(String),
}

impl TunnelError {
    pub fn no_reason(from: Option<&str>) -> Self {
        if let Some(v) = from {
            return TunnelError::Internal(Some(format!("{}: BadTunnelErrorReason", v)));
        }
        TunnelError::Internal(Some("BadTunnelErrorReason".to_string()))
    }

    pub fn internal(reason: &str) -> Self {
        TunnelError::Internal(Some(reason.to_string()))
    }

    fn api(from: &str, error: VpnApiError) -> Self {
        TunnelError::Api(format!("{from}: {error}"))
    }
}

impl From<Option<ErrorStateReason>> for TunnelError {
    fn from(reason: Option<ErrorStateReason>) -> Self {
        let Some(error) = reason else {
            warn!("missing error reason in TunnelError");
            return TunnelError::no_reason(None);
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
            BaseErrorStateReason::Internal => TunnelError::Internal(None),
            BaseErrorStateReason::Firewall => TunnelError::Firewall,
            BaseErrorStateReason::Routing => TunnelError::Routing,
            BaseErrorStateReason::Dns => TunnelError::Dns,
            BaseErrorStateReason::TunDevice => {
                TunnelError::internal(BaseErrorStateReason::TunDevice.as_str_name())
            }
            BaseErrorStateReason::TunnelProvider => {
                TunnelError::internal(BaseErrorStateReason::TunnelProvider.as_str_name())
            }
            BaseErrorStateReason::SameEntryAndExitGateway => TunnelError::SameEntryAndExitGw,
            BaseErrorStateReason::InvalidEntryGatewayCountry => TunnelError::InvalidEntryGwCountry,
            BaseErrorStateReason::InvalidExitGatewayCountry => TunnelError::InvalidExitGwCountry,
            BaseErrorStateReason::BadBandwidthIncrease => {
                TunnelError::internal(BaseErrorStateReason::BadBandwidthIncrease.as_str_name())
            }
            BaseErrorStateReason::DuplicateTunFd => {
                TunnelError::internal(BaseErrorStateReason::DuplicateTunFd.as_str_name())
            }
            BaseErrorStateReason::ResolveGatewayAddrs => {
                TunnelError::internal(BaseErrorStateReason::ResolveGatewayAddrs.as_str_name())
            }
            BaseErrorStateReason::StartLocalDnsResolver => {
                TunnelError::internal(BaseErrorStateReason::StartLocalDnsResolver.as_str_name())
            }
        }
    }
}

impl From<SyncAccountError> for TunnelError {
    fn from(error: SyncAccountError) -> Self {
        let Some(e) = error.error_detail else {
            warn!("missing error detail in SyncAccountError");
            return TunnelError::no_reason(Some("SyncAccount"));
        };
        match e {
            SyncAccountErr::NoAccountStored(b) => {
                TunnelError::internal(&format!("SyncAccountNoAccountStored {b}"))
            }
            SyncAccountErr::UnexpectedResponse(s) => {
                TunnelError::internal(&format!("SyncAccountUnexpectedResponse {s}"))
            }
            SyncAccountErr::Internal(s) => {
                TunnelError::internal(&format!("SyncAccountInternal {s}"))
            }
            SyncAccountErr::ErrorResponse(res) => TunnelError::api("SyncAccount", VpnApiError(res)),
        }
    }
}

impl From<SyncDeviceError> for TunnelError {
    fn from(error: SyncDeviceError) -> Self {
        let Some(e) = error.error_detail else {
            warn!("missing error detail in SyncDeviceError");
            return TunnelError::no_reason(Some("SyncDevice"));
        };
        match e {
            SyncDeviceErr::NoAccountStored(b) => {
                TunnelError::internal(&format!("SyncDeviceNoAccountStored {b}"))
            }
            SyncDeviceErr::NoDeviceStored(b) => {
                TunnelError::internal(&format!("SyncDeviceNoDeviceStored {b}"))
            }
            SyncDeviceErr::UnexpectedResponse(s) => {
                TunnelError::internal(&format!("SyncDeviceUnexpectedResponse {s}"))
            }
            SyncDeviceErr::Internal(s) => TunnelError::internal(&format!("SyncDeviceInternal {s}")),
            SyncDeviceErr::ErrorResponse(res) => TunnelError::api("SyncDevice", VpnApiError(res)),
        }
    }
}

impl From<RegisterDeviceError> for TunnelError {
    fn from(error: RegisterDeviceError) -> Self {
        let Some(e) = error.error_detail else {
            warn!("missing error detail in RegisterDeviceError");
            return TunnelError::no_reason(Some("RegisterDevice"));
        };
        match e {
            RegDeviceErr::NoAccountStored(b) => {
                TunnelError::internal(&format!("RegisterDeviceNoAccountStored {b}"))
            }
            RegDeviceErr::NoDeviceStored(b) => {
                TunnelError::internal(&format!("RegisterDeviceNoDeviceStored {b}"))
            }
            RegDeviceErr::UnexpectedResponse(s) => {
                TunnelError::internal(&format!("RegisterDeviceUnexpectedResponse {s}"))
            }
            RegDeviceErr::Internal(s) => {
                TunnelError::internal(&format!("RegisterDeviceInternal {s}"))
            }
            RegDeviceErr::ErrorResponse(res) => {
                TunnelError::api("RegisterDevice", VpnApiError(res))
            }
        }
    }
}

impl From<RequestZkNymError> for TunnelError {
    fn from(error: RequestZkNymError) -> Self {
        let Some(e) = error.outcome else {
            warn!("missing error detail in RequestZkNymError");
            return TunnelError::no_reason(Some("zknym"));
        };
        match e {
            ZkNymErr::NoAccountStored(b) => {
                TunnelError::internal(&format!("zknymNoAccountStored {b}"))
            }
            ZkNymErr::NoDeviceStored(b) => {
                TunnelError::internal(&format!("zknymNoDeviceStored {b}"))
            }
            ZkNymErr::UnexpectedVpnApiResponse(s) => {
                TunnelError::internal(&format!("zknymUnexpectedVpnApiResponse {s}"))
            }
            ZkNymErr::Storage(s) => TunnelError::internal(&format!("zknymStorage {s}")),
            ZkNymErr::Internal(s) => TunnelError::internal(&format!("zknymInternal {s}")),
            ZkNymErr::VpnApi(res) => TunnelError::api("zknym", VpnApiError(res)),
        }
    }
}

impl From<RequestZkNymBundle> for TunnelError {
    fn from(bundle: RequestZkNymBundle) -> Self {
        // TODO how the heck client is expected to deal with that kind of API??

        if bundle.failures.is_empty() {
            warn!("no failure found in RequestZkNymBundle");
            return TunnelError::no_reason(Some("RequestZkNymBundle"));
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
