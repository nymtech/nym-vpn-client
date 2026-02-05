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

use crate::auth_result::{authorize, deny};
#[cfg(windows)]
use crate::named_pipe::Connector;
#[cfg(unix)]
use crate::uds::Uds;

#[cfg(unix)]
type StreamItem = tokio::net::UnixStream;
#[cfg(windows)]
type StreamItem = Connector<tokio::net::windows::named_pipe::NamedPipeServer>;

pub struct AuthenticationLayer<T> {
    listener: T,
    nym_certificate_serial_number: String,
    shutdown_token: CancellationToken,
}

impl<T> AuthenticationLayer<T> {
    fn new(
        listener: T,
        nym_certificate_serial_number: String,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            listener,
            nym_certificate_serial_number,
            shutdown_token,
        }
    }
}

fn skip_authentication_checks() -> bool {
    // Let debug builds skip authorization process
    // TODO: Disable feature gating once front-end prevents spamming
    cfg!(debug_assertions) || cfg!(not(feature = "authentication"))
}

async fn authorized_stream(
    stream: &mut StreamItem,
    nym_certificate_serial_number: String,
    shutdown_token: CancellationToken,
) -> bool {
    if skip_authentication_checks() {
        authorize(stream).await;
        return true;
    }
    match is_authenticated(stream, nym_certificate_serial_number, shutdown_token).await {
        Ok(()) => {
            authorize(stream).await;
            true
        }
        Err(err) => {
            deny(stream).await;
            tracing::debug!("Connection did not get authenticated: {err:?}");
            false
        }
    }
}

impl<T: Unpin + Stream<Item = Result<StreamItem>>> AuthenticationLayer<T> {
    fn stream(mut self) -> impl Stream<Item = Result<StreamItem>> {
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
                if authorized_stream(&mut stream, self.nym_certificate_serial_number.clone(), self.shutdown_token.clone()).await {
                    yield stream;
                }

            }
        }
    }
}

#[cfg(unix)]
pub fn incoming(
    uds: Uds,
    nym_certificate_serial_number: String,
    shutdown_token: CancellationToken,
) -> impl Stream<Item = Result<StreamItem>> {
    let auth_layer = AuthenticationLayer::new(uds, nym_certificate_serial_number, shutdown_token);
    auth_layer.stream()
}

#[cfg(windows)]
pub fn incoming(
    named_pipe: crate::named_pipe::NamedPipeListener,
    nym_certificate_serial_number: String,
    shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = Result<StreamItem>>> {
    let listener = Box::pin(named_pipe.incoming()?);
    let auth_layer =
        AuthenticationLayer::new(listener, nym_certificate_serial_number, shutdown_token);
    Ok(auth_layer.stream())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    // Debug builds (like tests or dev runs) are automatically authorized
    async fn debug_build_authorized() {
        assert!(skip_authentication_checks());
    }
}
