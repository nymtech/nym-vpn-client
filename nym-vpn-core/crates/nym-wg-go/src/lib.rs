// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#![cfg(any(target_os = "ios", target_os = "android"))]

mod logging;
pub mod netstack;
pub mod uapi;
pub mod wireguard_go;

use std::{fmt, net::SocketAddr};

use base64::engine::Engine;
use ipnetwork::IpNetwork;
use uapi::UapiConfigBuilder;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to convert IP addr into C string")]
    IpAddrToCstr,

    #[error("failed to convert socket addr into C string")]
    SocketAddrToCstr,

    #[error("config contains nul byte")]
    ConfigContainsNulByte,

    #[error("failed to start netstack tunnel (code: {})", _0)]
    StartTunnel(i32),

    #[error("failed to open connection through the tunnel (code: {})", _0)]
    OpenConnection(i32),

    #[error("failed to set UAPI config (code: {})", _0)]
    SetUapiConfig(i64),

    #[error("failed to obtain tunnel socket fd")]
    ObtainSocketFd,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type LoggingCallback = unsafe extern "system" fn(
    level: u32, // WgLogLevel
    msg: *const std::ffi::c_char,
    context: *mut std::ffi::c_void,
);

/// WireGuard peer configuration.
pub struct PeerConfig {
    pub public_key: PublicKey,
    pub preshared_key: Option<PresharedKey>,
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<IpNetwork>,
}

impl PeerConfig {
    fn append_to(&self, config_builder: &mut UapiConfigBuilder) {
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
#[derive(Debug)]
pub struct PeerEndpointUpdate {
    pub public_key: PublicKey,
    pub endpoint: SocketAddr,
}

impl PeerEndpointUpdate {
    fn append_to(&self, config_builder: &mut UapiConfigBuilder) {
        config_builder.add("public_key", self.public_key.as_bytes().as_ref());
        config_builder.add("endpoint", self.endpoint.to_string().as_str());
    }
}

#[derive(Clone)]
pub struct PrivateKey(x25519_dalek::StaticSecret);

impl PrivateKey {
    pub fn from_base64(s: &str) -> Option<Self> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(s).ok()?;
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Some(PrivateKey::from(key))
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self.0)
    }
}

impl From<[u8; 32]> for PrivateKey {
    fn from(bytes: [u8; 32]) -> Self {
        Self(x25519_dalek::StaticSecret::from(bytes))
    }
}

#[derive(Clone)]
pub struct PublicKey(x25519_dalek::PublicKey);

impl PublicKey {
    pub fn from_base64(s: &str) -> Option<Self> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(s).ok()?;
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Some(PublicKey::from(key))
        } else {
            None
        }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.as_bytes())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_base64())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_base64())
    }
}

impl From<[u8; 32]> for PublicKey {
    fn from(public_key: [u8; 32]) -> PublicKey {
        PublicKey(x25519_dalek::PublicKey::from(public_key))
    }
}

impl<'a> From<&'a x25519_dalek::StaticSecret> for PublicKey {
    fn from(private_key: &'a x25519_dalek::StaticSecret) -> PublicKey {
        PublicKey(x25519_dalek::PublicKey::from(private_key))
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct PresharedKey([u8; 32]);

impl PresharedKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for PresharedKey {
    fn from(key: [u8; 32]) -> PresharedKey {
        PresharedKey(key)
    }
}
