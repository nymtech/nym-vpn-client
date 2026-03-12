// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio_stream::Stream;

#[cfg(any(not(debug_assertions), feature = "xpc"))]
use crate::xpc::{common::XpcConnection, daemon::XpcService};
use crate::{
    AuthenticationMaterial,
    authentication::{AuthenticationLayer, error::AuthenticationError},
};

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub type Transport = XpcConnection;
#[cfg(all(debug_assertions, not(feature = "xpc")))]
pub type Transport = tokio::net::UnixStream;

#[derive(Clone)]
pub struct SigningRequirements {
    pub daemon_req: String,
    pub client_req: String,
}

// Authentication happens in XPC layer, so if stream got through it means it's
// authenticated
pub(crate) async fn is_authenticated(
    _stream: &mut Transport,
    _auth_material: AuthenticationMaterial,
) -> Result<(), AuthenticationError> {
    Ok(())
}

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub(crate) fn incoming(xpc_service: XpcService) -> impl Stream<Item = std::io::Result<Transport>> {
    // XPC has built in authentication mechanism
    let auth_layer = AuthenticationLayer::new(xpc_service, None);
    auth_layer.stream()
}

#[cfg(all(debug_assertions, not(feature = "xpc")))]
pub(crate) fn incoming(
    uds: crate::uds::Uds,
    _auth_material: AuthenticationMaterial,
) -> impl Stream<Item = std::io::Result<Transport>> {
    // No authentication mechanism for MacOS UDS
    let auth_layer = AuthenticationLayer::new(uds, None);
    auth_layer.stream()
}
