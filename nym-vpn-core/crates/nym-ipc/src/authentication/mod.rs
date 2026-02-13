// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::{StreamItem, incoming, is_authenticated};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::{StreamItem, incoming, is_authenticated};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::{StreamItem, incoming, is_authenticated};

pub(crate) mod error;

use async_stream::try_stream;
use tokio_stream::{Stream, StreamExt};

use std::io::Result;

use crate::auth_result::{authorize, deny};

pub struct AuthenticationLayer<T> {
    listener: T,
    #[cfg(target_os = "windows")]
    nym_certificate_serial_number: String,
    #[cfg(target_os = "linux")]
    shutdown_token: tokio_util::sync::CancellationToken,
}

impl<T> AuthenticationLayer<T> {
    fn new(
        listener: T,
        #[cfg(target_os = "windows")] nym_certificate_serial_number: String,
        #[cfg(target_os = "linux")] shutdown_token: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            listener,
            #[cfg(target_os = "windows")]
            nym_certificate_serial_number,
            #[cfg(target_os = "linux")]
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
    #[cfg(target_os = "windows")] nym_certificate_serial_number: String,
    #[cfg(target_os = "linux")] shutdown_token: tokio_util::sync::CancellationToken,
) -> bool {
    if skip_authentication_checks() {
        authorize(stream).await;
        return true;
    }
    match is_authenticated(
        stream,
        #[cfg(target_os = "windows")]
        nym_certificate_serial_number,
        #[cfg(target_os = "linux")]
        shutdown_token,
    )
    .await
    {
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
                #[cfg(not(target_os = "linux"))]
                let shutdown_signal = std::future::pending::<()>();
                #[cfg(target_os = "linux")]
                let shutdown_signal = self.shutdown_token.cancelled();

                let next_stream = tokio::select! {
                    _ = shutdown_signal => {
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
                if authorized_stream(&mut stream, #[cfg(target_os = "windows")] self.nym_certificate_serial_number.clone(), #[cfg(target_os = "linux")] self.shutdown_token.clone()).await {
                    yield stream;
                }

            }
        }
    }
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
