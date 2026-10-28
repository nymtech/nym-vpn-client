// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod client;
pub mod codec;

pub type Empty = ();

pub mod service {
    include!(concat!(env!("OUT_DIR"), "/nym_vpn.NymVpnService.rs"));
}

pub use client::{Error, Result, RpcClient, get_rpc_socket_path};
pub use service::{
    nym_vpn_service_client::NymVpnServiceClient,
    nym_vpn_service_server::{NymVpnService, NymVpnServiceServer},
};
