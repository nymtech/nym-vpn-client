// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod error;
pub mod resolve_host;
pub mod request;
pub mod response;
pub mod types;
pub mod fronted_http_client;

pub(crate) mod jwt;

mod client;
mod network_compatibility;
mod routes;


pub use client::{ResolverOverrides, VpnApiClient};
pub use resolve_host::{str_to_socket_addr, url_to_socket_addr};
pub use network_compatibility::NetworkCompatibility;
pub use fronted_http_client::{build_fronted_http_client};
pub use response::ApiUrl;
