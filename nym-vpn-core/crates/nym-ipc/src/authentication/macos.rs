// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::net::UnixStream;
use tokio_stream::Stream;

use crate::{
    authentication::{AuthenticationLayer, error::AuthenticationError},
    uds::Uds,
};

pub(crate) type StreamItem = tokio::net::UnixStream;

// Not implemented yet, implicit is to consider it authenticated
pub(crate) async fn is_authenticated(_stream: &mut UnixStream) -> Result<(), AuthenticationError> {
    Ok(())
}

pub(crate) fn incoming(uds: Uds) -> impl Stream<Item = std::io::Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(uds);
    auth_layer.stream()
}
