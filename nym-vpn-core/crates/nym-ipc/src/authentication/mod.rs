// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::is_authenticated;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::is_authenticated;
use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) mod error;

use std::io::Result;

use async_stream::try_stream;
use tokio::net::UnixStream;
use tokio_stream::{Stream, StreamExt};

use crate::uds::Uds;

pub struct AuthenticationLayer {
    uds: Uds,
    shutdown_token: CancellationToken,
}

impl AuthenticationLayer {
    fn new(uds: Uds, shutdown_token: CancellationToken) -> Self {
        Self {
            uds,
            shutdown_token,
        }
    }
}

impl AuthenticationLayer {
    fn stream(mut self) -> impl Stream<Item = Result<UnixStream>> {
        try_stream! {
            loop {
                let next_stream = tokio::select! {
                    _ = self.shutdown_token.cancelled() => {
                        break;
                    }
                    stream = self.uds.next() => {
                        stream
                    }
                };
                let Some(stream) = next_stream else {
                    break;
                };
                match is_authenticated(stream?, self.shutdown_token.clone()).await{
                    Ok(stream) => yield stream,
                    Err(err) => tracing::debug!("Connection did not get authenticated: {err:?}"),
                }

            }
        }
    }
}

pub fn incoming(
    uds: Uds,
    shutdown_token: CancellationToken,
) -> impl Stream<Item = Result<UnixStream>> {
    let auth_layer = AuthenticationLayer::new(uds, shutdown_token);
    auth_layer.stream()
}
