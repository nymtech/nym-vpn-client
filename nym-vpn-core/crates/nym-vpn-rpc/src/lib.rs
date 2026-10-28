// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod client;
pub mod codec;
mod macros;
pub mod service;

pub type Empty = ();

pub use client::{Error, Result, RpcClient, get_rpc_socket_path};
pub use service::{NymVpnService, NymVpnServiceClient, NymVpnServiceServer};
