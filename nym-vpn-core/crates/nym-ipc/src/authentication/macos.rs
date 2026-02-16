// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio_stream::Stream;

use crate::{
    authentication::{AuthenticationLayer, error::AuthenticationError},
    xpc::{common::XpcConnection, daemon::XpcService},
};

pub(crate) type StreamItem = XpcConnection;

// Not implemented yet, implicit is to consider it authenticated
pub(crate) async fn is_authenticated(_stream: &mut StreamItem) -> Result<(), AuthenticationError> {
    Ok(())
}

pub(crate) fn incoming(xpc_service: XpcService) -> impl Stream<Item = std::io::Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(xpc_service);
    auth_layer.stream()
}
