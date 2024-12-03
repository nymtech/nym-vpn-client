use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use nym_vpn_proto::account_error::AccountErrorType;
use nym_vpn_proto::connect_request_error::ConnectRequestErrorType;
use nym_vpn_proto::connection_status_update::StatusType;
use nym_vpn_proto::set_network_request_error::SetNetworkRequestErrorType;
use nym_vpn_proto::{
    error::ErrorType as DError, AccountError, ConnectRequestError, GatewayType,
    SetNetworkRequestError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::grpc::client::VpndError;

const MAX_REG_DEVICES_ID_PATTERN: &str = "register-device.max-devices-exceeded";

#[derive(Error, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CmdErrorSource {
    #[error("daemon error")]
    DaemonError,
    #[error("internal error")]
    InternalError,
    #[error("caller error")]
    CallerError,
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
/// Generic error type made to be passed to the frontend and
/// displayed in the UI as localized error message
pub struct BackendError {
    /// Human readable error message for debugging/logs purposes
    pub message: String,
    /// Error key to be used in the UI to display localized error message
    pub key: ErrorKey,
    /// Extra data to be passed along to help specialize the problem
    pub data: Option<HashMap<String, String>>,
}

impl BackendError {
    pub fn new(message: &str, key: ErrorKey) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: None,
        }
    }

    pub fn _with_data(message: &str, key: ErrorKey, data: HashMap<&str, String>) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(data.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }

    pub fn with_details(message: &str, key: ErrorKey, details: String) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(HashMap::from([("details".to_string(), details)])),
        }
    }

    pub fn with_optional_data(
        message: &str,
        key: ErrorKey,
        data: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            message: message.to_string(),
            key,
            data,
        }
    }

    pub fn internal(message: &str, data: Option<HashMap<String, String>>) -> Self {
        Self {
            message: message.to_string(),
            key: ErrorKey::InternalError,
            data: data.map(|d| d.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "message '{}' key [{:?}] data [{:?}]",
            self.message,
            self.key,
            self.data.as_ref()
        )
    }
}

impl From<VpndError> for BackendError {
    fn from(error: VpndError) -> Self {
        match error {
            VpndError::GrpcError(s) => {
                BackendError::new(&format!("grpc error: {}", s), ErrorKey::GrpcError)
            }
            VpndError::FailedToConnectIpc(_) | VpndError::FailedToConnectHttp(_) => {
                BackendError::new(
                    "not connected to the daemon",
                    ErrorKey::NotConnectedToDaemon,
                )
            }
            VpndError::Response(e) => e,
        }
    }
}

impl From<nym_vpn_proto::Error> for BackendError {
    fn from(error: nym_vpn_proto::Error) -> Self {
        Self {
            message: error.message.clone(),
            key: error.kind().into(),
            data: error.details.into(),
        }
    }
}

/// Enum of the possible specialized errors emitted by the daemon
/// or the app backend side, to be passed to the UI layer
#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum ErrorKey {
    /// Generic unhandled error
    UnknownError,
    /// Any error that is not explicitly handled, and not related
    /// to the application layer
    /// Extra data should be passed along to help specialize the problem
    InternalError,
    /// gRPC bare layer error, when an RPC call fails (aka `tonic::Status`)
    /// That is, the error does not come from the application layer
    GrpcError,
    /// Happens when the app is not connected to a running daemon
    /// and attempts to make a gRPC call
    NotConnectedToDaemon,
    // Forwarded from proto `error::ErrorType`, connection state update
    CSDaemonInternal,
    CSUnhandledExit,
    CStateNoValidCredential,
    CStateTimeout,
    CStateMixnetTimeout,
    CStateMixnetStoragePaths,
    CStateMixnetDefaultStorage,
    CStateMixnetBuildClient,
    CStateMixnetConnect,
    CStateMixnetEntryGateway,
    CStateIprFailedToConnect,
    CStateGwDir,
    CStateGwDirLookupGateways,
    CStateGwDirLookupGatewayId,
    CStateGwDirLookupRouterAddr,
    CStateGwDirLookupIp,
    CStateGwDirEntry,
    CStateGwDirEntryId,
    CStateGwDirEntryLocation,
    CStateGwDirExit,
    CStateGwDirExitLocation,
    CStateGwDirSameEntryAndExitGw,
    CStateOutOfBandwidth,
    CStateOutOfBandwidthSettingUpTunnel,
    CStateBringInterfaceUp,
    CStateFirewallInit,
    CStateFirewallResetPolicy,
    CStateDnsInit,
    CStateDnsSet,
    CStateFindDefaultInterface,
    CSAuthenticatorFailedToConnect,
    CSAuthenticatorConnectTimeout,
    CSAuthenticatorInvalidResponse,
    CSAuthenticatorRegistrationDataVerification,
    CSAuthenticatorEntryGatewaySocketAddr,
    CSAuthenticatorEntryGatewayIpv4,
    CSAuthenticatorWrongVersion,
    CSAuthenticatorMalformedReply,
    CSAuthenticatorAddressNotFound,
    CSAuthenticatorAuthenticationNotPossible,
    CSAddIpv6Route,
    CSTun,
    CSRouting,
    CSWireguardConfig,
    CSMixnetConnectionMonitor,
    // Forwarded from proto `account_error::AccountErrorType`
    AccountInvalidMnemonic,
    AccountStorage,
    AccountIsConnected,
    // Other account related errors, forwarded from `connect_request_error::ConnectRequestErrorType`
    ConnectGeneral,
    ConnectNoAccountStored,
    ConnectNoDeviceStored,
    ConnectUpdateAccount,
    ConnectUpdateDevice,
    ConnectRegisterDevice,
    // Forwarded from proto `connection_status_update::StatusType`
    EntryGatewayNotRouting,
    ExitRouterPingIpv4,
    ExitRouterPingIpv6,
    ExitRouterNotRoutingIpv4,
    ExitRouterNotRoutingIpv6,
    UserNoBandwidth,
    WgTunnelError,
    // Failure when querying countries from gRPC
    GetMixnetEntryCountriesQuery,
    GetMixnetExitCountriesQuery,
    GetWgCountriesQuery,
    // Forwarded from proto `set_network_request_error::SetNetworkRequestErrorType`
    InvalidNetworkName,
    /// Custom error for the "maximum number of registered devices reached" error as it's not
    /// yet specialized in the backend
    MaxRegisteredDevices,
}

impl From<DError> for ErrorKey {
    fn from(value: DError) -> Self {
        match value {
            DError::NoValidCredentials => ErrorKey::CStateNoValidCredential,
            DError::Timeout => ErrorKey::CStateTimeout,
            DError::MixnetTimeout => ErrorKey::CStateMixnetTimeout,
            DError::MixnetStoragePaths => ErrorKey::CStateMixnetStoragePaths,
            DError::MixnetDefaultStorage => ErrorKey::CStateMixnetDefaultStorage,
            DError::MixnetBuildClient => ErrorKey::CStateMixnetBuildClient,
            DError::MixnetConnect => ErrorKey::CStateMixnetConnect,
            DError::MixnetEntryGateway => ErrorKey::CStateMixnetEntryGateway,
            DError::IprFailedToConnect => ErrorKey::CStateIprFailedToConnect,
            DError::GatewayDirectory => ErrorKey::CStateGwDir,
            DError::GatewayDirectoryLookupGateways => ErrorKey::CStateGwDirLookupGateways,
            DError::GatewayDirectoryLookupGatewayIdentity => ErrorKey::CStateGwDirLookupGatewayId,
            DError::GatewayDirectoryLookupRouterAddress => ErrorKey::CStateGwDirLookupRouterAddr,
            DError::GatewayDirectoryLookupIp => ErrorKey::CStateGwDirLookupIp,
            DError::GatewayDirectoryEntry => ErrorKey::CStateGwDirEntry,
            DError::GatewayDirectoryEntryId => ErrorKey::CStateGwDirEntryId,
            DError::GatewayDirectoryEntryLocation => ErrorKey::CStateGwDirEntryLocation,
            DError::GatewayDirectoryExit => ErrorKey::CStateGwDirExit,
            DError::GatewayDirectoryExitLocation => ErrorKey::CStateGwDirExitLocation,
            DError::GatewayDirectorySameEntryAndExitGw => ErrorKey::CStateGwDirSameEntryAndExitGw,
            DError::OutOfBandwidth => ErrorKey::CStateOutOfBandwidth,
            DError::OutOfBandwidthWhenSettingUpTunnel => {
                ErrorKey::CStateOutOfBandwidthSettingUpTunnel
            }
            DError::BringInterfaceUp => ErrorKey::CStateBringInterfaceUp,
            DError::FirewallInit => ErrorKey::CStateFirewallInit,
            DError::FirewallResetPolicy => ErrorKey::CStateFirewallResetPolicy,
            DError::DnsInit => ErrorKey::CStateDnsInit,
            DError::DnsSet => ErrorKey::CStateDnsSet,
            DError::FindDefaultInterface => ErrorKey::CStateFindDefaultInterface,
            DError::Internal => ErrorKey::CSDaemonInternal,
            DError::AuthenticatorFailedToConnect => ErrorKey::CSAuthenticatorFailedToConnect,
            DError::AuthenticatorConnectTimeout => ErrorKey::CSAuthenticatorConnectTimeout,
            DError::AuthenticatorInvalidResponse => ErrorKey::CSAuthenticatorInvalidResponse,
            DError::AuthenticatorRegistrationDataVerification => {
                ErrorKey::CSAuthenticatorRegistrationDataVerification
            }
            DError::AuthenticatorEntryGatewaySocketAddr => {
                ErrorKey::CSAuthenticatorEntryGatewaySocketAddr
            }
            DError::AuthenticatorEntryGatewayIpv4 => ErrorKey::CSAuthenticatorEntryGatewayIpv4,
            DError::AuthenticatorWrongVersion => ErrorKey::CSAuthenticatorWrongVersion,
            DError::AuthenticatorMalformedReply => ErrorKey::CSAuthenticatorMalformedReply,
            DError::AuthenticatorAddressNotFound => ErrorKey::CSAuthenticatorAddressNotFound,
            DError::AuthenticatorAuthenticationNotPossible => {
                ErrorKey::CSAuthenticatorAuthenticationNotPossible
            }
            DError::AddIpv6Route => ErrorKey::CSAddIpv6Route,
            DError::Tun => ErrorKey::CSTun,
            DError::Routing => ErrorKey::CSRouting,
            DError::WireguardConfig => ErrorKey::CSWireguardConfig,
            DError::MixnetConnectionMonitor => ErrorKey::CSMixnetConnectionMonitor,
            DError::UnhandledExit => ErrorKey::CSUnhandledExit,
            _ => ErrorKey::UnknownError, // `Unspecified` & `Unhandled`
        }
    }
}

impl From<AccountError> for BackendError {
    fn from(error: AccountError) -> Self {
        let data = error.details.clone().into();
        match error.kind() {
            AccountErrorType::StoreAccountErrorUnspecified => {
                BackendError::internal("grpc UNSPECIFIED", data)
            }
            AccountErrorType::InvalidMnemonic => BackendError::with_optional_data(
                "The provided mnemonic was not able to be parsed",
                ErrorKey::AccountInvalidMnemonic,
                data,
            ),
            AccountErrorType::Storage => BackendError::with_optional_data(
                "General error from the storage backend",
                ErrorKey::AccountStorage,
                data,
            ),
            AccountErrorType::IsConnected => BackendError::with_optional_data(
                "Unable to proceed while connected",
                ErrorKey::AccountIsConnected,
                data,
            ),
        }
    }
}

impl From<ConnectRequestErrorType> for ErrorKey {
    fn from(error: ConnectRequestErrorType) -> Self {
        match error {
            ConnectRequestErrorType::Internal | ConnectRequestErrorType::Unspecified => {
                ErrorKey::InternalError
            }
            ConnectRequestErrorType::General => ErrorKey::ConnectGeneral,
            ConnectRequestErrorType::NoAccountStored => ErrorKey::ConnectNoAccountStored,
            ConnectRequestErrorType::NoDeviceStored => ErrorKey::ConnectNoDeviceStored,
            ConnectRequestErrorType::UpdateAccount => ErrorKey::ConnectUpdateAccount,
            ConnectRequestErrorType::UpdateDevice => ErrorKey::ConnectUpdateDevice,
            ConnectRequestErrorType::RegisterDevice => ErrorKey::ConnectRegisterDevice,
        }
    }
}

impl From<ConnectRequestError> for BackendError {
    fn from(error: ConnectRequestError) -> Self {
        let message = error.message.clone();

        // TODO trick to handle "Maximum number of registered devices reached" error which is
        //  not yet specialized in the backend
        if let Some(true) = error
            .message_id
            .as_ref()
            .map(|id| id.contains(MAX_REG_DEVICES_ID_PATTERN))
        {
            return BackendError::new(&message, ErrorKey::MaxRegisteredDevices);
        }

        BackendError::new(&message, ErrorKey::from(error.kind()))
    }
}

impl From<StatusType> for ErrorKey {
    fn from(value: StatusType) -> Self {
        match value {
            StatusType::EntryGatewayNotRoutingMixnetMessages => ErrorKey::EntryGatewayNotRouting,
            StatusType::ExitRouterNotRespondingToIpv4Ping => ErrorKey::ExitRouterPingIpv4,
            StatusType::ExitRouterNotRespondingToIpv6Ping => ErrorKey::ExitRouterPingIpv6,
            StatusType::ExitRouterNotRoutingIpv4Traffic => ErrorKey::ExitRouterNotRoutingIpv4,
            StatusType::ExitRouterNotRoutingIpv6Traffic => ErrorKey::ExitRouterNotRoutingIpv6,
            StatusType::NoBandwidth => ErrorKey::UserNoBandwidth,
            StatusType::WgTunnelError => ErrorKey::WgTunnelError,
            _ => ErrorKey::UnknownError, // & `Unspecified`
        }
    }
}

impl From<GatewayType> for ErrorKey {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MixnetEntry => ErrorKey::GetMixnetEntryCountriesQuery,
            GatewayType::MixnetExit => ErrorKey::GetMixnetExitCountriesQuery,
            GatewayType::Wg => ErrorKey::GetWgCountriesQuery,
            _ => ErrorKey::UnknownError, // & `Unspecified`
        }
    }
}

impl From<SetNetworkRequestErrorType> for ErrorKey {
    fn from(error: SetNetworkRequestErrorType) -> Self {
        match error {
            SetNetworkRequestErrorType::Internal => ErrorKey::InternalError,
            SetNetworkRequestErrorType::InvalidNetworkName => ErrorKey::InvalidNetworkName,
            SetNetworkRequestErrorType::Unspecified => ErrorKey::UnknownError,
        }
    }
}

impl From<SetNetworkRequestError> for BackendError {
    fn from(error: SetNetworkRequestError) -> Self {
        let message = error.message.clone();
        BackendError::new(&message, ErrorKey::from(error.kind()))
    }
}
