// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod any_tunnel_handle;
pub mod gateway_provider;
pub mod mixnet;
mod tombstone;
pub mod transports;
pub mod wireguard;

pub use gateway_provider::SelectedGateways;

#[cfg(windows)]
use super::route_handler;
use crate::MixnetError;
pub use any_tunnel_handle::AnyTunnelHandle;
use gateway_provider::GatewayProviderError;
pub use tombstone::Tombstone;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to select gateways")]
    SelectGateways(#[source] Box<GatewayProviderError>),

    #[error("mixnet tunnel has failed")]
    MixnetClient(#[from] MixnetError),

    #[error("{} has no ip addresses announced", gateway_id)]
    NoIpAddressAnnounced { gateway_id: String },

    #[error("bandwidth monitor error")]
    BandwidthMonitor(#[from] crate::bandwidth_monitor::Error),

    #[error("registration client error")]
    RegistrationClient(#[source] Box<nym_registration_client::RegistrationClientError>),

    #[cfg(target_os = "ios")]
    #[error("failed to resolve using dns64")]
    ResolveDns64(#[from] wireguard::dns64::Error),

    #[error("WireGuard error")]
    Wireguard(#[from] nym_wg_go::Error),

    #[error("failed to dup tunnel file descriptor")]
    DupFd(#[source] std::io::Error),

    #[cfg(target_os = "android")]
    #[error("failed to create DNS filter proxy")]
    CreateDnsFilterProxy(#[source] std::io::Error),

    #[cfg(windows)]
    #[error("failed to add default route listener")]
    AddDefaultRouteListener(#[source] route_handler::Error),

    #[error("transport error")]
    Transport(#[from] transports::TransportError),

    #[error("connection cancelled")]
    Cancelled,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
