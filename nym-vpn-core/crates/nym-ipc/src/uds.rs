// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    io::Result,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use hyper_util::rt::TokioIo;
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::{wrappers::UnixListenerStream, Stream};

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

pub async fn connect(socket_path: impl AsRef<Path>) -> Result<TokioIo<UnixStream>> {
    Ok(TokioIo::new(UnixStream::connect(socket_path).await?))
}

pub fn incoming(socket_path: PathBuf) -> Result<Uds> {
    let uds = UnixListener::bind(&socket_path)?;

    fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o766))?;

    Ok(Uds {
        socket_path,
        inner: UnixListenerStream::new(uds),
    })
}
