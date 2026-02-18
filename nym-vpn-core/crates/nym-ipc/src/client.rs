// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io::Result, path::PathBuf};

use hyper_util::rt::TokioIo;
use tokio::io::AsyncRead;

/// Connect timeout used when the pipe reports that it's busy.
#[cfg(windows)]
const PIPE_AVAILABILITY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

async fn accepted<T: AsyncRead + Unpin>(mut conn: T) -> Result<TokioIo<T>> {
    let auth_res = crate::auth_result::AuthenticaticationResult::recv(&mut conn).await;
    if auth_res.accepted() {
        Ok(TokioIo::new(conn))
    } else {
        Err(std::io::ErrorKind::PermissionDenied.into())
    }
}

#[cfg(unix)]
pub async fn connect(socket_path: PathBuf) -> Result<TokioIo<tokio::net::UnixStream>> {
    let conn = tokio::net::UnixStream::connect(socket_path).await?;
    accepted(conn).await
}

#[cfg(windows)]
pub async fn connect(
    socket_path: PathBuf,
) -> Result<TokioIo<tokio::net::windows::named_pipe::NamedPipeClient>> {
    let attempt_start = tokio::time::Instant::now();
    let pipe_name = socket_path.into_os_string();
    let conn = loop {
        match tokio::net::windows::named_pipe::ClientOptions::new()
            .read(true)
            .write(true)
            .open(&pipe_name)
        {
            Err(e)
                if e.raw_os_error()
                    == Some(windows::Win32::Foundation::ERROR_PIPE_BUSY.0 as i32) =>
            {
                if attempt_start.elapsed() < PIPE_AVAILABILITY_TIMEOUT {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    continue;
                } else {
                    return Err(e);
                }
            }
            result => break result?,
        }
    };

    accepted(conn).await
}
