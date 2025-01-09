// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::AccountCommandError;
use nym_vpn_lib::{
    gateway_directory::Error as DirError, tunnel_state_machine, GatewayDirectoryError,
    NodeIdentity, Recipient,
};
use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};
use tracing::error;

use super::config::ConfigSetupError;

// Failure to initiate the connect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceConnectError {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("failed to connect: {0}")]
    Account(#[from] AccountNotReady),

    #[error("connection attempt cancelled")]
    Cancel,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum AccountNotReady {
    #[error("update account failed: {message}")]
    UpdateAccount {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("update device failed: {message}")]
    UpdateDevice {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("register device failed: {message}")]
    RegisterDevice {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device identity stored")]
    NoDeviceStored,

    // There are usually multiple independent zknym requests at a time
    #[error("failed to request zk-nym(s)")]
    RequestZkNym {
        failed: Vec<nym_vpn_account_controller::RequestZkNymError>,
    },

    #[error("general error: {0}")]
    General(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<AccountCommandError> for AccountNotReady {
    fn from(err: AccountCommandError) -> Self {
        match err {
            AccountCommandError::SyncAccountEndpointFailure(e) => AccountNotReady::UpdateAccount {
                message: e.message,
                message_id: e.message_id,
                code_reference_id: e.code_reference_id,
            },
            AccountCommandError::SyncDeviceEndpointFailure(e) => AccountNotReady::UpdateDevice {
                message: e.message,
                message_id: e.message_id,
                code_reference_id: e.code_reference_id,
            },
            AccountCommandError::RegisterDeviceEndpointFailure(e) => {
                AccountNotReady::RegisterDevice {
                    message: e.message,
                    message_id: e.message_id,
                    code_reference_id: e.code_reference_id,
                }
            }
            AccountCommandError::RequestZkNym {
                successes: _,
                failed,
            } => AccountNotReady::RequestZkNym { failed },
            AccountCommandError::NoAccountStored => AccountNotReady::NoAccountStored,
            AccountCommandError::NoDeviceStored => AccountNotReady::NoDeviceStored,
            AccountCommandError::RemoveAccount(e) => AccountNotReady::General(e),
            AccountCommandError::RemoveDeviceIdentity(e) => AccountNotReady::General(e),
            AccountCommandError::ResetCredentialStorage(e) => AccountNotReady::General(e),
            AccountCommandError::RemoveAccountFiles(e) => AccountNotReady::General(e),
            AccountCommandError::InitDeviceKeys(e) => AccountNotReady::General(e),
            AccountCommandError::General(err) => AccountNotReady::General(err),
            AccountCommandError::Internal(err) => AccountNotReady::Internal(err),
            AccountCommandError::UnregisterDeviceApiClientFailure(err) => {
                AccountNotReady::Internal(err)
            }
            AccountCommandError::RegistrationInProgress => {
                AccountNotReady::Internal(err.to_string())
            }
        }
    }
}

// Failure to initiate the disconnect
#[derive(Clone, Debug, thiserror::Error)]
pub enum VpnServiceDisconnectError {
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ConnectionFailedError {
    #[error("failed to connect (unhandled): {0}")]
    Unhandled(String),

    #[error("failed to connect (unhandled exit): {0}")]
    UnhandledExit(String),

    // Errors that happen, that shouldn't ever really happen
    #[error("internal error occurred: {0}")]
    InternalError(String),

    #[error("failed to get next usable credential")]
    InvalidCredential,

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
    AuthenticatorRegistrationDataVerificationFailed { reason: String },

    #[error("failed to parse the entry gateway socket addr in response: {reason}")]
    WgEntryGatewaySocketAddrFailedToParse { reason: String },

    #[error("failed to parse the entry gateway ipv4 in response: {reason}")]
    WgEntryGatewayIpv4FailedToParse { reason: String },

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
        // TODO: this will be replaced with error reason passed via grpc
        ConnectionFailedError::InternalError(err.to_string())
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

impl From<&nym_vpn_lib::wg_gateway_client::ErrorMessage> for ConnectionFailedError {
    fn from(err: &nym_vpn_lib::wg_gateway_client::ErrorMessage) -> Self {
        match err {
            nym_vpn_lib::wg_gateway_client::ErrorMessage::OutOfBandwidth {
                gateway_id,
                authenticator_address,
            } => ConnectionFailedError::OutOfBandwidth {
                gateway_id: gateway_id.clone(),
                authenticator_address: authenticator_address.clone(),
            },
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

    #[error("failed to check if account is stored: {source}")]
    FailedToCheckIfAccountIsStored {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to remove account: {source}")]
    FailedToRemoveAccount {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to forget account: {source}")]
    FailedToForgetAccount {
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

    #[error("failed to get account summary")]
    FailedToGetAccountSummary,

    #[error("failed to send command")]
    SendCommand {
        source: Box<SendError<nym_vpn_account_controller::AccountCommand>>,
    },

    #[error("account controller not ready to handle command")]
    RecvCommand { source: Box<RecvError> },

    #[error("no account stored")]
    NoAccountStored,

    #[error("failed to init device keys")]
    FailedToInitDeviceKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to reset device keys")]
    FailedToResetDeviceKeys {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error(transparent)]
    AccountControllerError {
        source: nym_vpn_account_controller::Error,
    },

    #[error(transparent)]
    AccountCommandError {
        source: nym_vpn_account_controller::AccountCommandError,
    },

    #[error("account not configured")]
    AccountManagementNotConfigured,

    #[error("failed to parse account links")]
    FailedToParseAccountLinks,

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("unable to proceed while connected")]
    IsConnected,
}

#[derive(Debug, thiserror::Error)]
pub enum SetNetworkError {
    #[error("failed to read config")]
    ReadConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to write config")]
    WriteConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to set network: {0}")]
    NetworkNotFound(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("account error: {0}")]
    Account(#[source] AccountError),

    #[error("config setup error: {0}")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("state machine error: {0}")]
    StateMachine(#[source] tunnel_state_machine::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
