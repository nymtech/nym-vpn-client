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

use tokio::net::UnixListener;
use tokio_stream::{Stream, wrappers::UnixListenerStream};
use tokio_util::sync::CancellationToken;

use crate::authentication::{self, StreamItem};

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
    type Item = Result<StreamItem>;

    #[allow(clippy::useless_conversion)]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<StreamItem>>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map(|n| n.map(|r| r.map(Into::into)))
    }
}

#[allow(unused)]
pub fn incoming(
    socket_path: PathBuf,
    _shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = Result<StreamItem>>> {
    let listener = UnixListener::bind(&socket_path)?;
    fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o766))?;
    let uds = Uds {
        socket_path,
        inner: UnixListenerStream::new(listener),
    };

    Ok(authentication::incoming_uds(
        uds,
        #[cfg(target_os = "linux")]
        _shutdown_token,
    ))
}
