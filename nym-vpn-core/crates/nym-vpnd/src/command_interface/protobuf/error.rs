// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use maplit::hashmap;
use nym_vpn_proto::{
    account_error::AccountErrorType, error::ErrorType, import_error::ImportErrorType,
    Error as ProtoError, ImportError as ProtoImportError,
};

use crate::service::{AccountError, ConnectionFailedError, ImportCredentialError};

impl From<ImportCredentialError> for ProtoImportError {
    fn from(err: ImportCredentialError) -> Self {
        match err {
            ImportCredentialError::VpnRunning => ProtoImportError {
                kind: ImportErrorType::VpnRunning as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::CredentialAlreadyImported => ProtoImportError {
                kind: ImportErrorType::CredentialAlreadyImported as i32,
                message: err.to_string(),
                details: Default::default(),
            },
            ImportCredentialError::StorageError {
                ref path,
                ref error,
            } => ProtoImportError {
                kind: ImportErrorType::StorageError as i32,
                message: err.to_string(),
                details: hashmap! {
                    "path".to_string() => path.to_string_lossy().to_string(),
                    "error".to_string() => error.to_string()
                },
            },
            ImportCredentialError::DeserializationFailure {
                ref reason,
                ref location,
            } => ProtoImportError {
                kind: ImportErrorType::DeserializationFailure as i32,
                message: err.to_string(),
                details: hashmap! {
                    "location".to_string() => location.to_string_lossy().to_string(),
                    "reason".to_string() => reason.clone(),
                },
            },
            ImportCredentialError::CredentialExpired {
                expiration,
                ref location,
            } => ProtoImportError {
                kind: ImportErrorType::CredentialExpired as i32,
                message: err.to_string(),
                details: hashmap! {
                    "location".to_string() => location.to_string_lossy().to_string(),
                    "expiration".to_string() => expiration.to_string(),
                },
            },
        }
    }
}

impl From<ConnectionFailedError> for ProtoError {
    fn from(err: ConnectionFailedError) -> Self {
        match err {
            ConnectionFailedError::Unhandled(ref reason) => ProtoError {
                kind: ErrorType::Unhandled as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::InternalError(ref reason) => ProtoError {
                kind: ErrorType::Internal as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::InvalidCredential {
                reason,
                location,
                gateway_id,
            } => ProtoError {
                kind: ErrorType::NoValidCredentials as i32,
                message: reason,
                details: [
                    ("location".to_string(), location),
                    ("gateway_id".to_string(), gateway_id),
                ]
                .into_iter()
                .collect(),
            },
            ConnectionFailedError::FailedToSetupMixnetStoragePaths { ref reason } => ProtoError {
                kind: ErrorType::MixnetStoragePaths as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToCreateMixnetClientWithDefaultStorage { ref reason } => {
                ProtoError {
                    kind: ErrorType::MixnetDefaultStorage as i32,
                    message: err.to_string(),
                    details: hashmap! {
                        "reason".to_string() => reason.to_string(),
                    },
                }
            }
            ConnectionFailedError::FailedToBuildMixnetClient { ref reason } => ProtoError {
                kind: ErrorType::MixnetBuildClient as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToConnectToMixnet { ref reason } => ProtoError {
                kind: ErrorType::MixnetConnect as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToConnectToMixnetEntryGateway {
                ref gateway_id,
                ref reason,
            } => ProtoError {
                kind: ErrorType::MixnetEntryGateway as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.clone(),
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::StartMixnetTimeout(timeout) => ProtoError {
                kind: ErrorType::MixnetTimeout as i32,
                message: timeout.to_string(),
                details: Default::default(),
            },
            ConnectionFailedError::FailedToSetupGatewayDirectoryClient {
                ref config,
                ref reason,
            } => ProtoError {
                kind: ErrorType::GatewayDirectory as i32,
                message: err.to_string(),
                details: hashmap! {
                    "config".to_string() => config.to_string(),
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToConnectToIpPacketRouter { ref reason } => ProtoError {
                kind: ErrorType::IprFailedToConnect as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToConnectToAuthenticator {
                ref gateway_id,
                ref authenticator_address,
                ref reason,
            } => ProtoError {
                kind: ErrorType::AuthenticatorFailedToConnect as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.to_string(),
                    "authenticator_address".to_string() => authenticator_address.to_string(),
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::TimeoutWaitingForConnectResponseFromAuthenticator {
                ref gateway_id,
                ref authenticator_address,
                ref reason,
            } => ProtoError {
                kind: ErrorType::AuthenticatorConnectTimeout as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.to_string(),
                    "authenticator_address".to_string() => authenticator_address.to_string(),
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::InvalidGatewayAuthResponse {
                gateway_id,
                authenticator_address,
                reason,
            } => todo!(),
            ConnectionFailedError::WgGatewayResponseVerificationFailed { reason } => todo!(),
            ConnectionFailedError::WgGatewayResponseEntryGatewaySocketAddrFailedToParse {
                reason,
            } => todo!(),
            ConnectionFailedError::WgGatewayResponseEntryGatewayIpv4FailedToParse { reason } => {
                todo!()
            }
            ConnectionFailedError::AuthenticatorRespondedWithWrongVersion {
                expected,
                received,
                gateway_id,
                authenticator_address,
            } => todo!(),
            ConnectionFailedError::MailformedAuthenticatorReply {
                gateway_id,
                authenticator_address,
                reason,
            } => todo!(),
            ConnectionFailedError::AuthenticatorAddressNotFound { gateway_id } => todo!(),
            ConnectionFailedError::AuthenticationNotPossible { reason } => todo!(),
            ConnectionFailedError::FailedToLookupGateways { ref reason } => ProtoError {
                kind: ErrorType::GatewayDirectoryLookupGateways as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToLookupGatewayIdentity { ref reason } => ProtoError {
                kind: ErrorType::GatewayDirectoryLookupGatewayIdentity as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToLookupRouterAddress { ref reason } => ProtoError {
                kind: ErrorType::GatewayDirectoryLookupRouterAddress as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToLookupGatewayIp {
                ref gateway_id,
                ref reason,
            } => ProtoError {
                kind: ErrorType::GatewayDirectoryLookupIp as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.to_string(),
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToSelectEntryGateway { ref reason } => ProtoError {
                kind: ErrorType::GatewayDirectoryEntry as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToSelectExitGateway { ref reason } => ProtoError {
                kind: ErrorType::GatewayDirectoryExit as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.clone(),
                },
            },
            ConnectionFailedError::FailedToSelectEntryGatewayIdNotFound { ref requested_id } => {
                ProtoError {
                    kind: ErrorType::GatewayDirectoryEntryId as i32,
                    message: err.to_string(),
                    details: hashmap! {
                        "requested_id".to_string() => requested_id.clone(),
                    },
                }
            }
            ConnectionFailedError::FailedToSelectEntryGatewayLocation {
                ref requested_location,
                ref available_countries,
            } => ProtoError {
                kind: ErrorType::GatewayDirectoryEntryLocation as i32,
                message: err.to_string(),
                details: hashmap! {
                    "requested_location".to_string() => requested_location.clone(),
                    "available_countries".to_string() => available_countries.join(", "),
                },
            },
            ConnectionFailedError::FailedToSelectExitGatewayLocation {
                ref requested_location,
                ref available_countries,
            } => ProtoError {
                kind: ErrorType::GatewayDirectoryExitLocation as i32,
                message: err.to_string(),
                details: hashmap! {
                    "requested_location".to_string() => requested_location.clone(),
                    "available_countries".to_string() => available_countries.join(", "),
                },
            },
            ConnectionFailedError::SameEntryAndExitGatewayFromCountry {
                ref requested_location,
            } => ProtoError {
                kind: ErrorType::GatewayDirectorySameEntryAndExitGw as i32,
                message: err.to_string(),
                details: hashmap! {
                    "requested_location".to_string() => requested_location.clone(),
                },
            },
            ConnectionFailedError::OutOfBandwidth {
                ref gateway_id,
                ref authenticator_address,
            } => ProtoError {
                kind: ErrorType::OutOfBandwidth as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.to_string(),
                    "authenticator_address".to_string() => authenticator_address.to_string(),
                },
            },
            ConnectionFailedError::OutOfBandwidthWhenSettingUpTunnel {
                ref gateway_id,
                ref authenticator_address,
            } => ProtoError {
                kind: ErrorType::OutOfBandwidthWhenSettingUpTunnel as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.to_string(),
                    "authenticator_address".to_string() => authenticator_address.to_string(),
                },
            },
            ConnectionFailedError::FailedToBringInterfaceUp {
                ref gateway_id,
                ref public_key,
                ref reason,
            } => ProtoError {
                kind: ErrorType::BringInterfaceUp as i32,
                message: err.to_string(),
                details: hashmap! {
                    "gateway_id".to_string() => gateway_id.to_string(),
                    "public_key".to_string() => public_key.clone(),
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToInitFirewall { ref reason } => ProtoError {
                kind: ErrorType::FirewallInit as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToResetFirewallPolicy { ref reason } => ProtoError {
                kind: ErrorType::FirewallResetPolicy as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToInitDns { ref reason } => ProtoError {
                kind: ErrorType::DnsInit as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToSetDns { ref reason } => ProtoError {
                kind: ErrorType::DnsSet as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToFindTheDefaultInterface { ref reason } => ProtoError {
                kind: ErrorType::FindDefaultInterface as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => reason.to_string(),
                },
            },
            ConnectionFailedError::FailedToAddIpv6Route { reason } => todo!(),
            ConnectionFailedError::TunError { reason } => todo!(),
            ConnectionFailedError::RoutingError { reason } => todo!(),
            ConnectionFailedError::WireguardConfigError { reason } => todo!(),
            ConnectionFailedError::MixnetConnectionMonitorError(reason) => todo!(),
        }
    }
}

impl From<AccountError> for nym_vpn_proto::AccountError {
    fn from(err: AccountError) -> Self {
        match err {
            AccountError::InvalidMnemonic { source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::InvalidMnemonic as i32,
                message: err.to_string(),
                details: hashmap! {
                    "source".to_string() => source.to_string(),
                },
            },
            AccountError::FailedToStoreAccount { ref source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {
                    "source".to_string() => source.to_string(),
                },
            },
            AccountError::FailedToLoadAccount { ref source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {
                    "source".to_string() => source.to_string(),
                },
            },
            AccountError::MissingApiUrl => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::InvalidApiUrl => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::VpnApiClientError(_) => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::FailedToLoadKeys { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
        }
    }
}
