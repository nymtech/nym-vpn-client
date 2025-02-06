// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

tonic::include_proto!("nym.vpn");

// client implementation only
tonic::include_proto!("grpc.health.v1");

// needed for reflection
pub const VPN_FD_SET: &[u8] = tonic::include_file_descriptor_set!("vpn_descriptor");

#[cfg(feature = "conversions")]
pub mod conversions;

// Re-export needed prost types
pub use prost_types::Timestamp;
