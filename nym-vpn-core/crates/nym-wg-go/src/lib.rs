// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "amnezia")]
pub mod amnezia;
pub mod netstack;
pub mod uapi;
pub mod wireguard_go;

use std::{fmt, net::SocketAddr};

use ipnetwork::IpNetwork;
use nym_crypto::asymmetric::x25519;
use nym_wireguard_types::PresharedKey;
use uapi::UapiConfigBuilder;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to start the tunnel (code: {0})")]
    StartTunnel(i32),

    #[error("failed to start UDP proxy (code: {0})")]
    StartUdpProxy(i32),

    #[error("failed to start TCP proxy (code: {0})")]
    StartTcpProxy(i32),

    #[error("failed to set UAPI config (code: {0})")]
    SetUapiConfig(i64),

    #[error("failed to get UAPI config")]
    GetUapiConfig,

    #[error("tunnel is stopped")]
    TunnelStopped,

    #[cfg(target_os = "android")]
    #[error("failed to obtain tunnel socket fd")]
    ObtainSocketFd,

    #[error("failed to convert {0} to C string")]
    ConvertToCString(&'static str),

    #[error("failed to convert {0} to string")]
    ConvertToString(&'static str),

    #[error("failed to parse listen address")]
    ParseListenAddr,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type LoggingCallback = unsafe extern "system" fn(
    level: u32, // WgLogLevel
    msg: *const std::ffi::c_char,
    context: *mut std::ffi::c_void,
);

/// WireGuard peer configuration.
pub struct PeerConfig {
    pub public_key: x25519::PublicKey,
    pub preshared_key: Option<PresharedKey>,
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<IpNetwork>,
}

impl PeerConfig {
    fn append_to(self, config_builder: &mut UapiConfigBuilder) {
        config_builder.add("public_key", self.public_key.as_bytes().as_ref());
        if let Some(preshared_key) = self.preshared_key.as_ref() {
            config_builder.add("preshared_key", preshared_key.as_bytes().as_ref());
        }

        config_builder.add("endpoint", self.endpoint.to_string().as_str());

        if !self.allowed_ips.is_empty() {
            config_builder.add("replace_allowed_ips", "true");
        }

        for allowed_ip in self.allowed_ips.iter() {
            config_builder.add("allowed_ip", allowed_ip.to_string().as_str());
        }
    }
}

impl fmt::Debug for PeerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PeerConfig")
            .field("public_key", &self.public_key)
            .field(
                "preshared_key",
                &self.preshared_key.as_ref().map(|_| "(hidden)"),
            )
            .field("endpoint", &self.endpoint)
            .field("allowed_ips", &self.allowed_ips)
            .finish()
    }
}

/// Holds new endpoint for the peer matching by public key.
#[derive(Debug, Clone, PartialEq)]
pub struct PeerEndpointUpdate {
    pub public_key: x25519::PublicKey,
    pub endpoint: SocketAddr,
}

impl PeerEndpointUpdate {
    fn append_to(&self, config_builder: &mut UapiConfigBuilder) {
        config_builder.add("public_key", self.public_key.as_bytes().as_ref());
        config_builder.add("endpoint", self.endpoint.to_string().as_str());
    }

    pub fn is_loopback(&self) -> bool {
        self.endpoint.ip().is_loopback()
    }
}
