// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::ExitPoint;

/// SOCKS5 enable request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct EnableSocks5Request {
    pub socks5_settings: Socks5Settings,
    pub http_rpc_settings: HttpRpcSettings,
    pub exit_point: ExitPoint,
}

/// SOCKS5 proxy settings
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Socks5Settings {
    /// SOCKS5 listen address, e.g., 127.0.0.1:1080
    #[cfg_attr(feature = "typescript-bindings", ts(as = "Option<String>"))]
    pub listen_address: Option<SocketAddr>,
}

/// HTTP RPC proxy settings
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct HttpRpcSettings {
    /// HTTP RPC listen address, e.g., 127.0.0.1:8545
    #[cfg_attr(feature = "typescript-bindings", ts(as = "Option<String>"))]
    pub listen_address: Option<SocketAddr>,
}

/// SOCKS5 service state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum Socks5State {
    /// Service is disabled
    Disabled,
    /// Enabled but no active connections (mixnet client not running)
    Idle,
    /// Active connections, mixnet client is running
    Connected,
    /// Error state
    Error,
}

impl std::fmt::Display for Socks5State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Socks5State::Disabled => "Disabled",
            Socks5State::Idle => "Idle",
            Socks5State::Connected => "Connected",
            Socks5State::Error => "Error",
        };
        write!(f, "{s}")
    }
}

/// SOCKS5 service status
#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Socks5Status {
    /// Current state of the SOCKS5 service
    pub state: Socks5State,
    /// SOCKS5 settings
    pub socks5_settings: Socks5Settings,
    /// HTTP RPC settings
    pub http_rpc_settings: HttpRpcSettings,
    /// Error message (if in error state)
    pub error_message: Option<String>,
    /// Number of active connections
    pub active_connections: u32,
}
