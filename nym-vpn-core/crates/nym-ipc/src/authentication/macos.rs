// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio_stream::Stream;

use crate::authentication::{AuthenticationLayer, error::AuthenticationError};
#[cfg(any(not(debug_assertions), feature = "xpc"))]
use crate::xpc::{common::XpcConnection, daemon::XpcService};

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub type StreamItem = XpcConnection;
#[cfg(all(debug_assertions, not(feature = "xpc")))]
pub type StreamItem = tokio::net::UnixStream;

// Not implemented yet, implicit is to consider it authenticated
pub(crate) async fn is_authenticated(_stream: &mut StreamItem) -> Result<(), AuthenticationError> {
    Ok(())
}

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub(crate) fn incoming(xpc_service: XpcService) -> impl Stream<Item = std::io::Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(xpc_service);
    auth_layer.stream()
}

#[cfg(all(debug_assertions, not(feature = "xpc")))]
pub(crate) fn incoming(uds: crate::uds::Uds) -> impl Stream<Item = std::io::Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(uds);
    auth_layer.stream()
}
