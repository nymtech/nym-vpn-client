// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

//! WireGuard tunnel creation and management on Android and iOS

#[cfg(target_os = "ios")]
pub mod ios;
#[cfg(any(target_os = "ios", target_os = "android"))]
pub mod runner;
pub mod tunnel_settings;
pub mod two_hop_config;
pub mod two_hop_tunnel;
pub mod wg_config;

use crate::platform::error::FFIError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to locate tun fd")]
    CannotLocateTunFd,

    #[cfg(target_os = "ios")]
    #[error("failed to obtain tun interface name")]
    ObtainTunName,

    #[error("tunnel failure")]
    Tunnel(#[from] nym_wg_go::Error),

    #[cfg(target_os = "ios")]
    #[error("DNS resolution failure")]
    DnsResolution(#[from] ios::dns64::Error),

    #[error("failed to set network settings")]
    SetNetworkSettings(#[source] FFIError),

    #[cfg(target_os = "ios")]
    #[error("failed to set default path observer")]
    SetDefaultPathObserver(#[source] FFIError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
