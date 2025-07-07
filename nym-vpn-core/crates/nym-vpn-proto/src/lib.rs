// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod proto {
    // Re-export needed prost types
    pub use prost_types::Timestamp;

    tonic::include_proto!("nym.vpn");
}

#[cfg(feature = "conversions")]
pub mod conversions;

#[cfg(feature = "rpc_client")]
pub mod rpc_client;
