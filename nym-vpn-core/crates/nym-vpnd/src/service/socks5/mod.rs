// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Lazy SOCKS5 proxy service that routes traffic through the Nym mixnet
//!
//! This module implements a lazy-initialed mixnet, triggered by the first SOCKS5 connection
//! - Listens on a public SOCKS5 port, or HTTP RPC port
//! - Initializes the Nym mixnet on first connection
//! - Proxies SOCKS5 traffic to internal Nym SDK SOCKS5 server
//! - Proxies HTTP RPC traffic to SOCKS5
//! - Shuts down mixnet after idle timeout

mod http_rpc_proxy;
mod lazy_service;
mod socks5_client;
mod socks5_wrapper;

pub use lazy_service::{LazySocks5Error, LazySocks5Service};
pub use nym_vpn_lib_types::{HttpRpcSettings, Socks5Settings, Socks5State, Socks5Status};
