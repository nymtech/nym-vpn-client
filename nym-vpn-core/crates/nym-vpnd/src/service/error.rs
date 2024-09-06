use std::path::PathBuf;

use nym_vpn_lib::{
    credentials::ImportCredentialError as VpnLibImportCredentialError,
    gateway_directory::Error as DirError, wg_gateway_client::Error as WgGatewayClientError,
    CredentialStorageError, GatewayDirectoryError, NodeIdentity, NymIdError,
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
                        if let Some(CredentialStorageError::ConstraintUnique) =
                            source.downcast_ref::<CredentialStorageError>()
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

    #[error("we ran out of bandwidth when setting up the tunnel")]
    OutOfBandwidthWhenSettingUpTunnel,

    #[error("failed to bring up tunnel, wireguard auth failed for {gateway_id}")]
    FailedToBringInterfaceUp {
        gateway_id: Box<NodeIdentity>,
        public_key: String,
        reason: String,
    },

    #[error("failed to init firewall: {reason}")]
    FailedToInitFirewall { reason: String },

    #[error("failed to reset firewall policy: {reason}")]
    FailedToResetFirewallPolicy { reason: String },

    #[error("DNS error: {reason}")]
    FailedToInitDns { reason: String },

    #[error("failed to set DNS: {reason}")]
    FailedToSetDns { reason: String },

    #[error("failed to find the default interface: {reason}")]
    FailedToFindTheDefaultInterface { reason: String },
}

impl From<&nym_vpn_lib::Error> for ConnectionFailedError {
    fn from(err: &nym_vpn_lib::Error) -> Self {
        match err {
            nym_vpn_lib::Error::StartMixnetClientTimeout(timeout_sec) => {
                ConnectionFailedError::StartMixnetTimeout(*timeout_sec)
            }
            nym_vpn_lib::Error::FailedToSetupMixnetClient(e) => match e {
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
                nym_vpn_lib::MixnetError::InvalidCredential {
                    reason,
                    path,
                    gateway_id,
                } => ConnectionFailedError::InvalidCredential {
                    reason: reason.to_string(),
                    location: path.to_string_lossy().to_string(),
                    gateway_id: gateway_id.clone(),
                },
                nym_vpn_lib::MixnetError::FailedToSerializeMessage { .. }
                | nym_vpn_lib::MixnetError::ConnectionMonitorError(_) => {
                    ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
                }
            },
            nym_vpn_lib::Error::SetupMixTunnelError(e) => match e {
                nym_vpn_lib::SetupMixTunnelError::FailedToLookupGatewayIp {
                    gateway_id,
                    source,
                } => ConnectionFailedError::FailedToLookupGatewayIp {
                    gateway_id: gateway_id.clone(),
                    reason: source.to_string(),
                },
                nym_vpn_lib::SetupMixTunnelError::FailedToConnectToIpPacketRouter(inner) => {
                    ConnectionFailedError::FailedToConnectToIpPacketRouter {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupMixTunnelError::FailedToSetDns(inner) => {
                    ConnectionFailedError::FailedToSetDns {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupMixTunnelError::TunError(_)
                | nym_vpn_lib::SetupMixTunnelError::ConnectionMonitorError(_)
                | nym_vpn_lib::SetupMixTunnelError::FailedToAddIpv6Route(_)
                | nym_vpn_lib::SetupMixTunnelError::RoutingError(_) => {
                    ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
                }
            },
            nym_vpn_lib::Error::SetupWgTunnelError(e) => match e {
                nym_vpn_lib::SetupWgTunnelError::NotEnoughBandwidthToSetupTunnel => {
                    ConnectionFailedError::OutOfBandwidthWhenSettingUpTunnel
                }
                nym_vpn_lib::SetupWgTunnelError::FailedToBringInterfaceUp {
                    gateway_id,
                    public_key,
                    source,
                } => ConnectionFailedError::FailedToBringInterfaceUp {
                    gateway_id: gateway_id.clone(),
                    public_key: public_key.clone(),
                    reason: source.to_string(),
                },
                nym_vpn_lib::SetupWgTunnelError::FailedToLookupGatewayIp { gateway_id, source } => {
                    ConnectionFailedError::FailedToLookupGatewayIp {
                        gateway_id: gateway_id.clone(),
                        reason: source.to_string(),
                    }
                }
                nym_vpn_lib::SetupWgTunnelError::WgGatewayClientError(ee) => match ee {
                    WgGatewayClientError::OutOfBandwidth => ConnectionFailedError::OutOfBandwidth,
                    WgGatewayClientError::InvalidGatewayAuthResponse
                    | WgGatewayClientError::AuthenticatorClientError(_)
                    | WgGatewayClientError::WireguardTypesError(_)
                    | WgGatewayClientError::FailedToParseEntryGatewaySocketAddr(_) => {
                        ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
                    }
                },
                nym_vpn_lib::SetupWgTunnelError::AuthenticationNotPossible(_)
                | nym_vpn_lib::SetupWgTunnelError::RoutingError(_)
                | nym_vpn_lib::SetupWgTunnelError::FailedToParseEntryGatewayIpv4(_)
                | nym_vpn_lib::SetupWgTunnelError::AuthenticatorAddressNotFound
                | nym_vpn_lib::SetupWgTunnelError::WireguardConfigError(_) => {
                    ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
                }
            },
            nym_vpn_lib::Error::GatewayDirectoryError(e) => e.into(),
            nym_vpn_lib::Error::FailedToInitFirewall(inner) => {
                ConnectionFailedError::FailedToInitFirewall {
                    reason: inner.to_string(),
                }
            }
            nym_vpn_lib::Error::FailedToInitDns(inner) => ConnectionFailedError::FailedToInitDns {
                reason: inner.to_string(),
            },
            nym_vpn_lib::Error::FailedToResetFirewallPolicy { reason } => {
                ConnectionFailedError::FailedToResetFirewallPolicy {
                    reason: reason.to_string(),
                }
            }
            nym_vpn_lib::Error::DefaultInterfaceError(inner) => {
                ConnectionFailedError::FailedToFindTheDefaultInterface {
                    reason: inner.to_string(),
                }
            }
            nym_vpn_lib::Error::RoutingError(_)
            | nym_vpn_lib::Error::FailedToSendWireguardShutdown
            | nym_vpn_lib::Error::NymVpnExitWithError(_)
            | nym_vpn_lib::Error::NymVpnExitUnexpectedChannelClose => {
                ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
            }
            #[cfg(unix)]
            nym_vpn_lib::Error::TunProvider(_) => {
                ConnectionFailedError::Unhandled(format!("unhandled error: {err:#?}"))
            }
        }
    }
}

impl From<&nym_vpn_lib::GatewayDirectoryError> for ConnectionFailedError {
    fn from(e: &nym_vpn_lib::GatewayDirectoryError) -> Self {
        match e {
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
            GatewayDirectoryError::SameEntryAndExitGatewayFromCountry { requested_location } => {
                ConnectionFailedError::SameEntryAndExitGatewayFromCountry {
                    requested_location: requested_location.clone(),
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("invalid mnemonic")]
    InvalidMnemonic {
        #[from]
        source: bip39::Error,
    },

    #[error("failed to store account: {source}")]
    FailedToStoreAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to load account: {source}")]
    FailedToLoadAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("no nym-vpn-api url setup")]
    MissingApiUrl,

    #[error("invalid nym-vpn-api url")]
    InvalidApiUrl,

    #[error(transparent)]
    VpnApiClientError(#[from] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to load keys: {source}")]
    FailedToLoadKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
