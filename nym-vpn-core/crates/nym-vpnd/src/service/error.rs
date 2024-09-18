use std::path::PathBuf;

use nym_vpn_lib::{
    credentials::ImportCredentialError as VpnLibImportCredentialError,
    gateway_directory::Error as DirError, wg_gateway_client::Error as WgGatewayClientError,
    AuthenticatorClientError, CredentialStorageError, GatewayDirectoryError, NodeIdentity,
    NymIdError, Recipient,
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

    // Errors that happen, that shouldn't ever really happen
    #[error("internal error occurred: {0}")]
    InternalError(String),

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

    #[error("failed to connect to authenticator at {gateway_id}: {reason}")]
    FailedToConnectToAuthenticator {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
        reason: String,
    },

    #[error("timeout waiting for connect response from authenticator at {gateway_id}: {reason}")]
    TimeoutWaitingForConnectResponseFromAuthenticator {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
        reason: String,
    },

    #[error("invalid gateway auth response from `{gateway_id}`: {reason}")]
    InvalidGatewayAuthResponse {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
        reason: String,
    },

    #[error("verification failed for wg gateway response: {reason}")]
    WgGatewayResponseVerificationFailed { reason: String },

    #[error("failed to parse the entry gateway socket addr in response: {reason}")]
    WgGatewayResponseEntryGatewaySocketAddrFailedToParse { reason: String },

    #[error("failed to parse the entry gateway ipv4 in response: {reason}")]
    WgGatewayResponseEntryGatewayIpv4FailedToParse { reason: String },

    #[error("gateway authenticator responded with unexpected version: {received}")]
    AuthenticatorRespondedWithWrongVersion {
        expected: u8,
        received: u8,
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
    },

    #[error("mailformed authenticator reply from `{gateway_id}`: {reason}")]
    MailformedAuthenticatorReply {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
        reason: String,
    },

    #[error("authenticator address not found for gateway: `{gateway_id}`")]
    AuthenticatorAddressNotFound { gateway_id: Box<NodeIdentity> },

    #[error("authentication not possible: {reason}")]
    AuthenticationNotPossible { reason: String },

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
    FailedToLookupGatewayIp {
        gateway_id: Box<NodeIdentity>,
        reason: String,
    },

    #[error("unable to use same entry and exit gateway for location: {requested_location}")]
    SameEntryAndExitGatewayFromCountry { requested_location: String },

    #[error("we ran out of bandwidth with gateway: `{gateway_id}`")]
    OutOfBandwidth {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
    },

    #[error("we ran out of bandwidth when setting up the tunnel: `{gateway_id}`")]
    OutOfBandwidthWhenSettingUpTunnel {
        gateway_id: Box<NodeIdentity>,
        authenticator_address: Box<Recipient>,
    },

    #[error("failed to bring up tunnel to gateway `{gateway_id}` with public key `{public_key}`: {reason}")]
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

    #[error("failed to add ipv6 route: {reason}")]
    FailedToAddIpv6Route { reason: String },

    #[error("tun device error: {reason}")]
    TunError { reason: String },

    #[error("routing error: {reason}")]
    RoutingError { reason: String },

    #[error("wireguard config error: {reason}")]
    WireguardConfigError { reason: String },

    #[error("mixnet connection monitor error: {0}")]
    MixnetConnectionMonitorError(String),
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
                nym_vpn_lib::MixnetError::FailedToSerializeMessage { source } => {
                    ConnectionFailedError::InternalError(source.to_string())
                }
                nym_vpn_lib::MixnetError::ConnectionMonitorError(_) => {
                    ConnectionFailedError::MixnetConnectionMonitorError(err.to_string())
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
                nym_vpn_lib::SetupMixTunnelError::TunError(inner) => {
                    ConnectionFailedError::TunError {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupMixTunnelError::FailedToAddIpv6Route(inner) => {
                    ConnectionFailedError::FailedToAddIpv6Route {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupMixTunnelError::RoutingError(inner) => {
                    ConnectionFailedError::RoutingError {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupMixTunnelError::ConnectionMonitorError(_) => {
                    ConnectionFailedError::MixnetConnectionMonitorError(err.to_string())
                }
            },
            nym_vpn_lib::Error::SetupWgTunnelError(e) => match e {
                nym_vpn_lib::SetupWgTunnelError::NotEnoughBandwidthToSetupTunnel {
                    gateway_id,
                    authenticator_address,
                } => ConnectionFailedError::OutOfBandwidthWhenSettingUpTunnel {
                    gateway_id: gateway_id.clone(),
                    authenticator_address: authenticator_address.clone(),
                },
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
                nym_vpn_lib::SetupWgTunnelError::WgGatewayClientError {
                    gateway_id,
                    authenticator_address,
                    source,
                } => match source {
                    WgGatewayClientError::OutOfBandwidth => ConnectionFailedError::OutOfBandwidth {
                        gateway_id: gateway_id.clone(),
                        authenticator_address: authenticator_address.clone(),
                    },
                    WgGatewayClientError::AuthenticatorClientError(auth_err) => match auth_err {
                        AuthenticatorClientError::TimeoutWaitingForConnectResponse => {
                            ConnectionFailedError::TimeoutWaitingForConnectResponseFromAuthenticator {
                                gateway_id: gateway_id.clone(),
                                authenticator_address: authenticator_address.clone(),
                                reason: auth_err.to_string(),
                            }
                        }
                        AuthenticatorClientError::FailedToSendMixnetMessage(_) => {
                            ConnectionFailedError::FailedToConnectToAuthenticator {
                                gateway_id: gateway_id.clone(),
                                authenticator_address: authenticator_address.clone(),
                                reason: auth_err.to_string(),
                            }
                        }
                        AuthenticatorClientError::NoMixnetMessagesReceived
                        | AuthenticatorClientError::UnableToGetMixnetHandle => {
                            ConnectionFailedError::InternalError(auth_err.to_string())
                        }
                        AuthenticatorClientError::ReceivedResponseWithOldVersion { expected, received  }
                        | AuthenticatorClientError::ReceivedResponseWithNewVersion { expected, received } => {
                            ConnectionFailedError::AuthenticatorRespondedWithWrongVersion {
                                expected: *expected,
                                received: *received,
                                gateway_id: gateway_id.clone(),
                                authenticator_address: authenticator_address.clone(),
                            }
                        }
                        AuthenticatorClientError::NoVersionInMessage => {
                            ConnectionFailedError::MailformedAuthenticatorReply {
                                gateway_id: gateway_id.clone(),
                                authenticator_address: authenticator_address.clone(),
                                reason: "no version in message".to_string(),
                            }
                        }
                    },
                    WgGatewayClientError::InvalidGatewayAuthResponse => {
                        ConnectionFailedError::InvalidGatewayAuthResponse {
                            gateway_id: gateway_id.clone(),
                            authenticator_address: authenticator_address.clone(),
                            reason: err.to_string(),
                        }
                    }
                    WgGatewayClientError::VerificationFailed(inner) => {
                        ConnectionFailedError::WgGatewayResponseVerificationFailed {
                            reason: inner.to_string(),
                        }
                    }
                    WgGatewayClientError::FailedToParseEntryGatewaySocketAddr(inner) => {
                        ConnectionFailedError::WgGatewayResponseEntryGatewaySocketAddrFailedToParse {
                            reason: inner.to_string(),
                        }
                    }
                },
                nym_vpn_lib::SetupWgTunnelError::RoutingError(inner) => {
                    ConnectionFailedError::RoutingError {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupWgTunnelError::WireguardConfigError(inner) => {
                    ConnectionFailedError::WireguardConfigError {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupWgTunnelError::AuthenticatorAddressNotFound { gateway_id } => {
                    ConnectionFailedError::AuthenticatorAddressNotFound {
                        gateway_id: gateway_id.clone(),
                    }
                }
                nym_vpn_lib::SetupWgTunnelError::AuthenticationNotPossible(inner) => {
                    ConnectionFailedError::AuthenticationNotPossible {
                        reason: inner.to_string(),
                    }
                }
                nym_vpn_lib::SetupWgTunnelError::FailedToParseEntryGatewayIpv4(inner) => {
                    ConnectionFailedError::WgGatewayResponseEntryGatewayIpv4FailedToParse {
                        reason: inner.to_string(),
                    }
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
            nym_vpn_lib::Error::RoutingError(inner) => {
                ConnectionFailedError::RoutingError {
                    reason: inner.to_string(),
                }
            }
            nym_vpn_lib::Error::FailedToSendWireguardShutdown
            | nym_vpn_lib::Error::NymVpnExitWithError(_)
            | nym_vpn_lib::Error::NymVpnExitUnexpectedChannelClose => {
                ConnectionFailedError::InternalError(err.to_string())
            }
            #[cfg(unix)]
            nym_vpn_lib::Error::TunProvider(inner) => {
                ConnectionFailedError::TunError { reason: inner.to_string() }
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
