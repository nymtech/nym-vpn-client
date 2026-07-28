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
mod skew_manager;

pub use client::VpnApiClient;
pub use fronted_http_client::{
    api_url_to_url, api_url_to_url_and_domain, api_urls_to_urls, fronted_http_client,
    fronted_http_client_builder,
};
pub use network_compatibility::NetworkCompatibility;
pub use resolve_host::{str_to_socket_addr, url_to_socket_addr};
pub use skew_manager::{DeviceTimeProvider, RemoteTimeProvider, SkewManager};
pub use types::ResolverOverrides;
