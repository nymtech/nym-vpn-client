// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::NodeIdentity;

use crate::{tunnel_setup::WaitInterfaceUpError, MixnetError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    RoutingError(#[from] talpid_routing::Error),

    #[error("failed to init dns: {0}")]
    FailedToInitDns(#[source] talpid_core::dns::Error),

    #[error("failed to init firewall: {0}")]
    FailedToInitFirewall(String),

    #[error("failed to reset firewall policy: {reason}")]
    FailedToResetFirewallPolicy { reason: String },

    #[error("failed to send shutdown message to wireguard tunnel")]
    FailedToSendWireguardShutdown,

    #[error(transparent)]
    GatewayDirectoryError(#[from] GatewayDirectoryError),

    #[error("could not obtain the default interface: {0}")]
    DefaultInterfaceError(String),

    #[error(transparent)]
    SetupWgTunnelError(#[from] SetupWgTunnelError),

    #[error(transparent)]
    SetupMixTunnelError(#[from] SetupMixTunnelError),

    #[error("timeout after waiting {0}s for mixnet client to start")]
    StartMixnetClientTimeout(u64),

    #[error("failed to setup mixnet client: {0}")]
    FailedToSetupMixnetClient(#[source] MixnetError),

    #[error("{0}")]
    NymVpnExitWithError(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("vpn exit listener channel unexpected close when listening")]
    NymVpnExitUnexpectedChannelClose,

    #[cfg(any(target_os = "ios", target_os = "android"))]
    #[error("vpn errored on stop")]
    StopError,

    #[cfg(any(target_os = "ios", target_os = "android"))]
    #[error("failed setting up local TUN network device: {0}")]
    TunError(#[from] tun2::Error),

    #[cfg(any(unix, target_os = "android"))]
    #[error("{0}")]
    TunProvider(#[from] talpid_tunnel::tun_provider::Error),

    #[cfg(target_os = "ios")]
    #[error("{0}")]
    UniffiError(#[from] crate::platform::error::VpnError),

    #[cfg(target_os = "ios")]
    #[error("failed to run wireguard tunnel")]
    RunTunnel(#[from] crate::mobile::Error),
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

    #[error("unable to use same entry and exit gateway for location: {requested_location}")]
    SameEntryAndExitGatewayFromCountry { requested_location: String },
}

#[derive(thiserror::Error, Debug)]
pub enum SetupMixTunnelError {
    #[error("{0}")]
    ConnectionMonitorError(#[from] nym_connection_monitor::Error),

    #[error("failed to connect to ip packet router: {0}")]
    FailedToConnectToIpPacketRouter(#[source] nym_ip_packet_client::Error),

    #[error("failed to lookup gateway ip: {gateway_id}: {source}")]
    FailedToLookupGatewayIp {
        gateway_id: String,
        source: nym_gateway_directory::Error,
    },

    #[error("failed setting up local TUN network device: {0}")]
    TunError(#[from] tun2::Error),

    #[error("failed to add ipv6 route: {0}")]
    FailedToAddIpv6Route(#[source] std::io::Error),

    #[error("{0}")]
    RoutingError(#[from] talpid_routing::Error),

    #[error("failed to set DNS: {0}")]
    FailedToSetDns(#[source] talpid_core::dns::Error),

    #[cfg(target_os = "android")]
    #[error("vpn errored on stop")]
    StopError,

    #[cfg(target_os = "ios")]
    #[error("{0}")]
    UniffiError(#[from] crate::platform::error::VpnError),

    #[cfg(target_os = "ios")]
    #[error("failed to locate tun fd")]
    CannotLocateTunFd,
}

#[derive(thiserror::Error, Debug)]
pub enum SetupWgTunnelError {
    #[error("wiregurad authentication is not possible due to one of the gateways not running the authenticator process: {0}")]
    AuthenticationNotPossible(String),

    #[error("failed to find authenticator address")]
    AuthenticatorAddressNotFound,

    #[error("not enough bandwidth to setup tunnel")]
    NotEnoughBandwidthToSetupTunnel,

    #[error("failed to lookup gateway ip: {gateway_id}: {source}")]
    FailedToLookupGatewayIp {
        gateway_id: String,
        source: nym_gateway_directory::Error,
    },

    #[error(transparent)]
    WgGatewayClientError(#[from] nym_wg_gateway_client::Error),

    #[error("{0}")]
    RoutingError(#[from] talpid_routing::Error),

    #[error("{0}")]
    WireguardConfigError(#[from] talpid_wireguard::config::Error),

    #[error("failed to parse entry gateway ipv4: {0}")]
    FailedToParseEntryGatewayIpv4(#[source] std::net::AddrParseError),

    #[error("failed to bring up wireguard interface for gateway `{gateway_id}` with public key `{public_key}`: {source}")]
    FailedToBringInterfaceUp {
        gateway_id: Box<NodeIdentity>,
        public_key: String,
        source: WaitInterfaceUpError,
    },
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
