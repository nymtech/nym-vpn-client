// Copyright 2025-2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    io::{Error, ErrorKind, Result},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use tokio::net::{UnixListener, UnixStream};
use tokio_stream::{Stream, wrappers::UnixListenerStream};

use crate::{
    AuthenticationMaterial,
    authentication::{self, Transport},
};

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
    type Item = Result<Transport>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Transport>>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

pub async fn incoming(
    socket_path: PathBuf,
    auth_material: AuthenticationMaterial,
) -> Result<impl Stream<Item = Result<Transport>>> {
    ensure_no_other_listener(&socket_path).await?;

    // Remove previous socket file in case if the daemon crashed in the prior run and could not clean up the socket file.
    remove_previous_socket_file(&socket_path).await;
    tracing::info!("Starting socket listener on: {}", socket_path.display());

    let listener = UnixListener::bind(&socket_path)?;
    fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o666))?;
    let uds = Uds {
        socket_path,
        inner: UnixListenerStream::new(listener),
    };

    Ok(authentication::incoming(uds, auth_material))
}

async fn ensure_no_other_listener(socket_path: &Path) -> Result<()> {
    match UnixStream::connect(socket_path).await {
        Ok(_) => Err(Error::new(
            ErrorKind::AddrInUse,
            format!(
                "Another instance is already listening on {}",
                socket_path.display()
            ),
        )),
        Err(_) => Ok(()),
    }
}

async fn remove_previous_socket_file(socket_path: &Path) {
    match tokio::fs::remove_file(socket_path).await {
        Ok(_) => tracing::info!(
            "Removed previous command interface socket: {}",
            socket_path.display()
        ),
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            tracing::error!(
                "Failed to remove previous command interface socket: {:?}",
                err
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_socket_with_active_listener() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        let _listener = UnixListener::bind(&socket_path).unwrap();

        let err = ensure_no_other_listener(&socket_path).await.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AddrInUse);
    }

    #[tokio::test]
    async fn accepts_stale_socket_file() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        drop(UnixListener::bind(&socket_path).unwrap());
        assert!(socket_path.exists());

        ensure_no_other_listener(&socket_path).await.unwrap();
    }

    #[tokio::test]
    async fn accepts_missing_socket_file() {
        let dir = tempfile::tempdir().unwrap();

        ensure_no_other_listener(&dir.path().join("test.sock"))
            .await
            .unwrap();
    }
}
