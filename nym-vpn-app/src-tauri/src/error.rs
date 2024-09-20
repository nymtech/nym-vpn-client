use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use nym_vpn_proto::connection_status_update::StatusType;
use nym_vpn_proto::import_error::ImportErrorType;
use nym_vpn_proto::{error::ErrorType as DError, GatewayType, ImportError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::grpc::client::VpndError;

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

    pub fn _new_with_data(message: &str, key: ErrorKey, data: HashMap<&str, String>) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(data.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }

    pub fn new_with_details(message: &str, key: ErrorKey, details: String) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(HashMap::from([("details".to_string(), details)])),
        }
    }

    pub fn new_with_optional_data(
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

    pub fn new_internal(message: &str, data: Option<HashMap<String, String>>) -> Self {
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
    /// gRPC bare layer error, when a RPC call fails (aka `Tonic::Status`)
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
    /// Import invalid credential format -> base58 decoding failed
    CredentialInvalid,
    // Forwarded from proto `import_error::ImportErrorType`
    CredentialVpnRunning,
    CredentialAlreadyImported,
    CredentialStorageError,
    CredentialDeserializationFailure,
    CredentialExpired,
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

impl From<ImportError> for BackendError {
    fn from(error: ImportError) -> Self {
        let data = error.details.clone().into();
        match error.kind() {
            ImportErrorType::Unspecified => BackendError::new_internal("grpc unspecified", data),
            ImportErrorType::VpnRunning => BackendError::new_with_optional_data(
                "vpn running",
                ErrorKey::CredentialVpnRunning,
                data,
            ),
            ImportErrorType::CredentialAlreadyImported => BackendError::new_with_optional_data(
                "credential already imported",
                ErrorKey::CredentialAlreadyImported,
                data,
            ),
            ImportErrorType::StorageError => {
                // TODO remove this
                // backward compatibility check with the old error message from daemon
                if data.as_ref().is_some_and(|d| {
                    d.get("error")
                        .is_some_and(|e| e.contains("unique constraint violation"))
                }) {
                    return BackendError::new_with_optional_data(
                        "credential already imported",
                        ErrorKey::CredentialAlreadyImported,
                        data,
                    );
                }
                BackendError::new_with_optional_data(
                    "credential strorage error",
                    ErrorKey::CredentialStorageError,
                    data,
                )
            }
            ImportErrorType::DeserializationFailure => BackendError::new_with_optional_data(
                "credential deserialization failure",
                ErrorKey::CredentialDeserializationFailure,
                data,
            ),
            ImportErrorType::CredentialExpired => BackendError::new_with_optional_data(
                "credential expired",
                ErrorKey::CredentialExpired,
                data,
            ),
        }
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
