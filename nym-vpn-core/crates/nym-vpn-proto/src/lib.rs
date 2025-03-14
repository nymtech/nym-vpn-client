// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

tonic::include_proto!("nym.vpn");

#[cfg(feature = "conversions")]
pub mod conversions;

// Re-export needed prost types
pub use prost_types::Timestamp;
