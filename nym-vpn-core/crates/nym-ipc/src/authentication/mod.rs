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

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::is_authenticated;

use tokio_util::sync::CancellationToken;

pub(crate) mod error;

use std::io::Result;

use async_stream::try_stream;
use tokio_stream::{Stream, StreamExt};

use crate::auth_result::authorize;
#[cfg(windows)]
use crate::named_pipe::Connector;
#[cfg(unix)]
use crate::uds::Uds;

#[cfg(unix)]
type StreamItem = Result<tokio::net::UnixStream>;
#[cfg(windows)]
type StreamItem = Result<Connector<tokio::net::windows::named_pipe::NamedPipeServer>>;

pub struct AuthenticationLayer<T> {
    listener: T,
    shutdown_token: CancellationToken,
}

impl<T> AuthenticationLayer<T> {
    fn new(listener: T, shutdown_token: CancellationToken) -> Self {
        Self {
            listener,
            shutdown_token,
        }
    }
}

impl<T: Unpin + Stream<Item = StreamItem>> AuthenticationLayer<T> {
    fn stream(mut self) -> impl Stream<Item = StreamItem> {
        try_stream! {
            loop {
                let next_stream = tokio::select! {
                    _ = self.shutdown_token.cancelled() => {
                        break;
                    }
                    stream = self.listener.next() => {
                        stream
                    }
                };
                let Some(stream) = next_stream else {
                    break;
                };
                let mut stream = stream?;
                // Let debug builds skip authorization process
                // TODO: Disable feature gating once front-end prevents spamming
                if cfg!(debug_assertions) || cfg!(not(feature = "authentication")) {
                    authorize(&mut stream).await;
                    yield stream;
                    continue;
                }
                match is_authenticated(stream, self.shutdown_token.clone()).await{
                    Ok(stream) => yield stream,
                    Err(err) => tracing::debug!("Connection did not get authenticated: {err:?}"),
                }

            }
        }
    }
}

#[cfg(unix)]
pub fn incoming(uds: Uds, shutdown_token: CancellationToken) -> impl Stream<Item = StreamItem> {
    let auth_layer = AuthenticationLayer::new(uds, shutdown_token);
    auth_layer.stream()
}

#[cfg(windows)]
pub fn incoming(
    named_pipe: crate::named_pipe::NamedPipeListener,
    shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = StreamItem>> {
    let listener = Box::pin(named_pipe.incoming()?);
    let auth_layer = AuthenticationLayer::new(listener, shutdown_token);
    Ok(auth_layer.stream())
}
