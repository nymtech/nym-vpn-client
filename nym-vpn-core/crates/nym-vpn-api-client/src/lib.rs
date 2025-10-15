// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod error;
pub mod fronted_http_client;
pub mod request;
pub mod resolve_host;
pub mod response;
pub mod types;

pub(crate) mod jwt;

mod client;
mod network_compatibility;
mod routes;

pub use client::{ResolverOverrides, VpnApiClient};
pub use fronted_http_client::build_fronted_http_client;
pub use network_compatibility::NetworkCompatibility;
pub use resolve_host::{str_to_socket_addr, url_to_socket_addr};
