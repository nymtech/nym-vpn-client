// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, io::Result, os::unix::fs::PermissionsExt, path::PathBuf};

use async_stream::try_stream;
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::{Stream, StreamExt, wrappers::UnixListenerStream};
use tokio_util::sync::CancellationToken;

use crate::authentication::is_authenticated;

pub struct Uds {
    socket_path: PathBuf,
    inner: UnixListenerStream,
    shutdown_token: CancellationToken,
}

impl Uds {
    fn stream(mut self) -> impl Stream<Item = Result<UnixStream>> {
        try_stream! {
            loop {
                let next_stream = tokio::select! {
                    _ = self.shutdown_token.cancelled() => {
                        break;
                    }
                    stream = self.inner.next() => {
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

impl Drop for Uds {
    fn drop(&mut self) {
        if let Ok(()) = fs::remove_file(&self.socket_path) {
            tracing::trace!("Removed socket file at: {}", self.socket_path.display());
        }
    }
}

pub fn incoming(
    socket_path: PathBuf,
    shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = Result<UnixStream>>> {
    let uds: UnixListener = UnixListener::bind(&socket_path)?;

    fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o766))?;
    let uds = Uds {
        socket_path,
        inner: UnixListenerStream::new(uds),
        shutdown_token,
    };

    Ok(uds.stream())
}
