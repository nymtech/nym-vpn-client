// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod proto {
    tonic::include_proto!("nym_vpn_service");
}

#[cfg(feature = "conversions")]
pub mod conversions;

#[cfg(feature = "rpc_client")]
pub mod rpc_client;
