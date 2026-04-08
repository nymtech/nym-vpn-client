// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[allow(clippy::large_enum_variant)]
pub mod proto {
    pub use prost_types::Timestamp;
    // remove when this bug is resolved: https://github.com/hyperium/tonic/issues/2388
    use std::primitive::{bool, i32};

    tonic::include_proto!("nym_vpn_service");
}

#[cfg(feature = "conversions")]
pub mod conversions;

#[cfg(feature = "rpc_client")]
pub mod rpc_client;
