use std::path::PathBuf;

use nym_vpn_lib::{
    credential_storage_pre_ecash::error::StorageError,
    credentials::ImportCredentialError as VpnLibImportCredentialError, id_pre_ecash::NymIdError,
    GatewayDirectoryError,
};
use time::OffsetDateTime;
use tracing::error;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ImportCredentialError {
    #[error("vpn is connected")]
    VpnRunning,

    #[error("credential already imported")]
    CredentialAlreadyImported,

    #[error("storage error: {path}: {error}")]
    StorageError { path: PathBuf, error: String },

    #[error("failed to deserialize credential: {reason}")]
    DeserializationFailure { reason: String, location: PathBuf },

    #[error("credential expired: {expiration}")]
    CredentialExpired {
        expiration: OffsetDateTime,
        location: PathBuf,
    },
}

impl From<VpnLibImportCredentialError> for ImportCredentialError {
    fn from(err: VpnLibImportCredentialError) -> Self {
        match err {
            VpnLibImportCredentialError::CredentialStoreError { path, source } => {
                ImportCredentialError::StorageError {
                    path,
                    error: source.to_string(),
                }
            }
            VpnLibImportCredentialError::FailedToImportRawCredential { location, source } => {
                match source {
                    NymIdError::CredentialDeserializationFailure { source } => {
                        ImportCredentialError::DeserializationFailure {
                            reason: source.to_string(),
                            location,
                        }
                    }
                    NymIdError::ExpiredCredentialImport { expiration } => {
                        ImportCredentialError::CredentialExpired {
                            expiration,
                            location,
                        }
                    }
                    NymIdError::StorageError { source } => {
                        if let Some(StorageError::ConstraintUnique) =
                            source.downcast_ref::<StorageError>()
                        {
                            return ImportCredentialError::CredentialAlreadyImported;
                        }
                        ImportCredentialError::StorageError {
                            path: location,
                            error: source.to_string(),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ConnectionFailedError {
    // This error type used only while we we are in the process of properly mapping all errors. The
    // set of errors that can be returned by the vpn-lib is unmanagably large and is in the process
    // of being cleaned up, but will take some time
    #[error("failed to connect (unhandled): {0}")]
    Unhandled(String),

    #[error("failed to get next usable credential: {reason}")]
    InvalidCredential {
        reason: String,
        location: String,
        gateway_id: String,
    },

    #[error("failed to setup mixnet storage paths: {reason}")]
    FailedToSetupMixnetStoragePaths { reason: String },

    #[error("failed to create mixnet client with default storage: {reason}")]
    FailedToCreateMixnetClientWithDefaultStorage { reason: String },

    #[error("failed to build mixnet client: {reason}")]
    FailedToBuildMixnetClient { reason: String },

    #[error("failed to connect to mixnet: {reason}")]
    FailedToConnectToMixnet { reason: String },

    #[error("failed to connect to entry gateway {gateway_id}: {reason}")]
    FailedToConnectToMixnetEntryGateway { gateway_id: String, reason: String },

    #[error("timeout starting mixnet client after {0} seconds")]
    StartMixnetTimeout(u64),

    #[error("failed to setup gateway directory client: {reason}")]
    FailedToSetupGatewayDirectoryClient {
        config: Box<nym_vpn_lib::gateway_directory::Config>,
        reason: String,
    },

    #[error("failed to connect to ip packet router: {reason}")]
    FailedToConnectToIpPacketRouter { reason: String },

    #[error("failed to lookup gateways: {reason}")]
    FailedToLookupGateways { reason: String },

    #[error("failed to lookup gateway identity: {reason}")]
    FailedToLookupGatewayIdentity { reason: String },

    #[error("failed to lookup router address: {reason}")]
    FailedToLookupRouterAddress { reason: String },

    #[error("failed to select entry gateway: {reason}")]
    FailedToSelectEntryGateway { reason: String },

    #[error("failed to select exit gateway: {reason}")]
    FailedToSelectExitGateway { reason: String },

    #[error("selected gateway id not found: {requested_id}")]
    FailedToSelectEntryGatewayIdNotFound { requested_id: String },

    #[error("failed to select entry gateway location: {requested_location}")]
    FailedToSelectEntryGatewayLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("failed to select exit gateway location: {requested_location}")]
    FailedToSelectExitGatewayLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("failed to lookup gateway ip: {gateway_id}")]
    FailedToLookupGatewayIp { gateway_id: String, reason: String },

    #[error("unable to use same entry and exit gateway for location: {requested_location}")]
    SameEntryAndExitGatewayFromCountry { requested_location: String },

    #[error("we ran out of bandwidth")]
    OutOfBandwidth,
}

use nym_vpn_lib::gateway_directory::Error as DirError;

impl From<&nym_vpn_lib::Error> for ConnectionFailedError {
    fn from(err: &nym_vpn_lib::Error) -> Self {
        match err {
            nym_vpn_lib::Error::InvalidCredential {
                reason,
                path,
                gateway_id,
            } => ConnectionFailedError::InvalidCredential {
                reason: reason.to_string(),
                location: path.to_string_lossy().to_string(),
                gateway_id: gateway_id.clone(),
            },
            nym_vpn_lib::Error::StartMixnetTimeout(timeout_sec) => {
                ConnectionFailedError::StartMixnetTimeout(*timeout_sec)
            }
            nym_vpn_lib::Error::Mixnet(e) => match e {
                nym_vpn_lib::MixnetError::FailedToSetupMixnetStoragePaths(source) => {
                    ConnectionFailedError::FailedToSetupMixnetStoragePaths {
                        reason: source.to_string(),
                    }
                }
                nym_vpn_lib::MixnetError::FailedToCreateMixnetClientWithDefaultStorage(source) => {
                    ConnectionFailedError::FailedToCreateMixnetClientWithDefaultStorage {
                        reason: source.to_string(),
                    }
                }
                nym_vpn_lib::MixnetError::FailedToBuildMixnetClient(source) => {
                    ConnectionFailedError::FailedToBuildMixnetClient {
                        reason: source.to_string(),
                    }
                }
                nym_vpn_lib::MixnetError::FailedToConnectToMixnet(source) => {
                    ConnectionFailedError::FailedToConnectToMixnet {
                        reason: source.to_string(),
                    }
                }
                nym_vpn_lib::MixnetError::EntryGateway { gateway_id, source } => {
                    ConnectionFailedError::FailedToConnectToMixnetEntryGateway {
                        gateway_id: gateway_id.clone(),
                        reason: source.to_string(),
                    }
                }
            },
            nym_vpn_lib::Error::GatewayDirectoryError(e) => match e {
                GatewayDirectoryError::FailedtoSetupGatewayDirectoryClient { config, source } => {
                    ConnectionFailedError::FailedToSetupGatewayDirectoryClient {
                        config: Box::new(*config.clone()),
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::FailedToLookupGateways { source } => {
                    ConnectionFailedError::FailedToLookupGateways {
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::FailedToLookupGatewayIdentity { source } => {
                    ConnectionFailedError::FailedToLookupGatewayIdentity {
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::FailedToSelectEntryGateway {
                    source:
                        DirError::NoMatchingEntryGatewayForLocation {
                            requested_location,
                            available_countries,
                        },
                } => ConnectionFailedError::FailedToSelectEntryGatewayLocation {
                    requested_location: requested_location.clone(),
                    available_countries: available_countries.clone(),
                },
                GatewayDirectoryError::FailedToSelectEntryGateway {
                    source: DirError::NoMatchingGateway { requested_identity },
                } => ConnectionFailedError::FailedToSelectEntryGatewayIdNotFound {
                    requested_id: requested_identity.clone(),
                },
                GatewayDirectoryError::FailedToSelectEntryGateway { source } => {
                    ConnectionFailedError::FailedToSelectEntryGateway {
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::FailedToSelectExitGateway {
                    source:
                        DirError::NoMatchingExitGatewayForLocation {
                            requested_location,
                            available_countries,
                        },
                } => ConnectionFailedError::FailedToSelectExitGatewayLocation {
                    requested_location: requested_location.clone(),
                    available_countries: available_countries.clone(),
                },
                GatewayDirectoryError::FailedToSelectExitGateway { source } => {
                    ConnectionFailedError::FailedToSelectExitGateway {
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::FailedToLookupRouterAddress { source } => {
                    ConnectionFailedError::FailedToLookupRouterAddress {
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::FailedToLookupGatewayIp { gateway_id, source } => {
                    ConnectionFailedError::FailedToLookupGatewayIp {
                        gateway_id: gateway_id.clone(),
                        reason: source.to_string(),
                    }
                }
                GatewayDirectoryError::SameEntryAndExitGatewayFromCountry {
                    requested_location,
                } => ConnectionFailedError::SameEntryAndExitGatewayFromCountry {
                    requested_location: requested_location.clone(),
                },
            },
            nym_vpn_lib::Error::FailedToConnectToIpPacketRouter(inner) => {
                ConnectionFailedError::FailedToConnectToIpPacketRouter {
                    reason: inner.to_string(),
                }
            }
            nym_vpn_lib::Error::OutOfBandwidth => ConnectionFailedError::OutOfBandwidth,
            nym_vpn_lib::Error::AddrParseError(_)
            | nym_vpn_lib::Error::RoutingError(_)
            | nym_vpn_lib::Error::FailedToAddIpv6Route(_)
            | nym_vpn_lib::Error::DNSError(_)
            | nym_vpn_lib::Error::FirewallError(_)
            | nym_vpn_lib::Error::JoinError(_)
            | nym_vpn_lib::Error::CanceledError(_)
            | nym_vpn_lib::Error::FailedToSendWireguardShutdown
            | nym_vpn_lib::Error::TunError(_)
            | nym_vpn_lib::Error::WireguardConfigError(_)
            | nym_vpn_lib::Error::WireguardTypesError(_)
            | nym_vpn_lib::Error::DefaultInterfaceError
            | nym_vpn_lib::Error::StopError
            | nym_vpn_lib::Error::FailedToSerializeMessage { .. }
            | nym_vpn_lib::Error::CountryCodeNotFound
            | nym_vpn_lib::Error::FailedToDecodeBase58Credential { .. }
            | nym_vpn_lib::Error::ConnectionMonitorError(_)
            | nym_vpn_lib::Error::ImportCredentialError(_)
            | nym_vpn_lib::Error::InvalidGatewayAuthResponse
            | nym_vpn_lib::Error::AuthenticatorClientError(_)
            | nym_vpn_lib::Error::AuthenticationNotPossible(_)
            | nym_vpn_lib::Error::AuthenticatorAddressNotFound
            | nym_vpn_lib::Error::NotEnoughBandwidth
            | nym_vpn_lib::Error::BadWireguardEvent => {
                ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
            }
            #[cfg(windows)]
            nym_vpn_lib::Error::AdminPrivilegesRequired { .. } => {
                ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
            }
            #[cfg(unix)]
            nym_vpn_lib::Error::TunProvider(_)
            | nym_vpn_lib::Error::RootPrivilegesRequired { .. } => {
                ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreAccountError {
    #[error("invalid mnemonic")]
    InvalidMnemonic {
        #[from]
        source: bip39::Error,
    },

    #[error("failed to store account: {source}")]
    FailedToStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
