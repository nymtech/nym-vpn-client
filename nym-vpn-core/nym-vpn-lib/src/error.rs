// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    RoutingError(#[from] talpid_routing::Error),

    #[error("{0}")]
    DNSError(#[from] talpid_core::dns::Error),

    // We are not returning the underlying talpid_core::firewall:Error error as I ran into issues
    // with the Send marker trait not being implemented when building on Mac. Possibly we can fix
    // this in the future.
    #[error("{0}")]
    FirewallError(String),

    #[error("{0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("{0}")]
    CanceledError(#[from] futures::channel::oneshot::Canceled),

    #[error("failed to send shutdown message to wireguard tunnel")]
    FailedToSendWireguardShutdown,

    #[error("failed setting up local TUN network device: {0}")]
    TunError(#[from] tun2::Error),

    #[error("{0}")]
    WireguardConfigError(#[from] talpid_wireguard::config::Error),

    #[error(transparent)]
    WgGatewayClientError(#[from] nym_wg_gateway_client::WgGatewayClientError),

    #[error("could not obtain the default interface")]
    DefaultInterfaceError,

    #[error(transparent)]
    Mixnet(#[from] MixnetError),

    #[error("timeout after waiting {0}s for mixnet client to start")]
    StartMixnetTimeout(u64),

    #[error("vpn errored on stop")]
    StopError,

    #[cfg(any(unix, target_os = "android"))]
    #[error("{0}")]
    TunProvider(#[from] talpid_tunnel::tun_provider::Error),

    #[cfg(target_os = "ios")]
    #[error("{0}")]
    UniffiError(#[from] crate::platform::error::FFIError),

    #[error("failed to serialize message")]
    FailedToSerializeMessage {
        #[from]
        source: bincode::Error,
    },

    #[error("gateway does not contain a two character country ISO")]
    CountryCodeNotFound,

    #[error(transparent)]
    GatewayDirectoryError(#[from] GatewayDirectoryError),

    #[error("failed decode base58 credential: {source}")]
    FailedToDecodeBase58Credential {
        #[from]
        source: bs58::decode::Error,
    },

    #[error("{0}")]
    ConnectionMonitorError(#[from] nym_connection_monitor::Error),

    #[cfg(unix)]
    #[error("sudo/root privileges required, try rerunning with sudo: `sudo -E {binary_name} run`")]
    RootPrivilegesRequired { binary_name: String },

    #[cfg(windows)]
    #[error("administrator privileges required, try rerunning with administrator privileges: `runas /user:Administrator {binary_name} run`")]
    AdminPrivilegesRequired { binary_name: String },

    #[error("invalid credential: {reason}")]
    InvalidCredential {
        reason: crate::credentials::CheckImportedCredentialError,
        path: PathBuf,
        gateway_id: String,
    },

    #[error(transparent)]
    ImportCredentialError(#[from] crate::credentials::ImportCredentialError),

    #[error("failed to connect to ip packet router: {0}")]
    FailedToConnectToIpPacketRouter(#[source] nym_ip_packet_client::Error),

    #[error("received bad event for wireguard tunnel creation")]
    BadWireguardEvent,

    #[error("wiregurad authentication is not possible due to one of the gateways not running the authenticator process: {0}")]
    AuthenticationNotPossible(String),

    #[error("failed to find authenticator address")]
    AuthenticatorAddressNotFound,

    #[error("not enough bandwidth")]
    NotEnoughBandwidth,

    #[error("failed to add ipv6 route: {0}")]
    FailedToAddIpv6Route(#[source] std::io::Error),

    #[error("failed to parse entry gateway ipv4: {0}")]
    FailedToParseEntryGatewayIpv4(#[source] std::net::AddrParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum GatewayDirectoryError {
    #[error("failed to setup gateway directory client: {source}")]
    FailedtoSetupGatewayDirectoryClient {
        config: Box<nym_gateway_directory::Config>,
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateways: {source}")]
    FailedToLookupGateways {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateway identity: {source}")]
    FailedToLookupGatewayIdentity {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to select entry gateway: {source}")]
    FailedToSelectEntryGateway {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to select exit gateway: {source}")]
    FailedToSelectExitGateway {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup router address: {source}")]
    FailedToLookupRouterAddress {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateway ip: {gateway_id}: {source}")]
    FailedToLookupGatewayIp {
        gateway_id: String,
        source: nym_gateway_directory::Error,
    },

    #[error("unable to use same entry and exit gateway for location: {requested_location}")]
    SameEntryAndExitGatewayFromCountry { requested_location: String },
}

// Errors specific to the mixnet. This often comes from the nym-sdk crate, but not necessarily.
#[derive(thiserror::Error, Debug)]
pub enum MixnetError {
    #[error("failed to setup mixnet storage paths: {0}")]
    FailedToSetupMixnetStoragePaths(#[source] nym_sdk::Error),

    #[error("failed to create mixnet client with default storage: {0}")]
    FailedToCreateMixnetClientWithDefaultStorage(#[source] nym_sdk::Error),

    #[error("failed to build mixnet client: {0}")]
    FailedToBuildMixnetClient(#[source] nym_sdk::Error),

    #[error("failed to connect to mixnet: {0}")]
    FailedToConnectToMixnet(#[source] nym_sdk::Error),

    #[error("failed to connect to mixnet entry gateway {gateway_id}: {source}")]
    EntryGateway {
        gateway_id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
