// Copyright 2025-2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    io::Result,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::net::{UnixListener, UnixStream};
use tokio_stream::{Stream, wrappers::UnixListenerStream};
use tokio_util::sync::CancellationToken;

use crate::authentication;

pub struct Uds {
    socket_path: PathBuf,
    inner: UnixListenerStream,
}

impl Drop for Uds {
    fn drop(&mut self) {
        if let Ok(()) = fs::remove_file(&self.socket_path) {
            tracing::trace!("Removed socket file at: {}", self.socket_path.display());
        }
    }
}

impl Stream for Uds {
    type Item = Result<UnixStream>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<UnixStream>>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

pub fn incoming(
    socket_path: PathBuf,
    shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = Result<UnixStream>>> {
    let listener = UnixListener::bind(&socket_path)?;
    fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o766))?;
    let uds = Uds {
        socket_path,
        inner: UnixListenerStream::new(listener),
    };

    Ok(authentication::incoming(uds, shutdown_token))
}
