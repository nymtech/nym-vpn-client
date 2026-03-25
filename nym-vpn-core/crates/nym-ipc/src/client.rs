// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::Result;

use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::auth_result::{AuthenticaticationQuery, AuthenticaticationResult};

/// Connect timeout used when the pipe reports that it's busy.
#[cfg(windows)]
const PIPE_AVAILABILITY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

async fn accepted<T: AsyncWrite + AsyncRead + Unpin>(mut conn: T) -> Result<TokioIo<T>> {
    // XPC requires the client to be the first to send queries before the
    // connection is initiated, so best to do it on all platforms
    AuthenticaticationQuery::query(&mut conn).await;
    let auth_res = AuthenticaticationResult::recv(&mut conn).await;
    match auth_res {
        AuthenticaticationResult::Accepted => Ok(TokioIo::new(conn)),
        AuthenticaticationResult::Denied => Err(std::io::ErrorKind::PermissionDenied.into()),
        AuthenticaticationResult::Closed => Err(std::io::ErrorKind::ConnectionRefused.into()),
    }
}

#[cfg(all(target_os = "macos", any(not(debug_assertions), feature = "xpc")))]
pub async fn connect(
    _socket_path: std::path::PathBuf,
) -> Result<TokioIo<crate::xpc::common::XpcConnection>> {
    let conn = crate::xpc::client::connect().await?;
    accepted(conn).await
}

#[cfg(any(
    target_os = "linux",
    all(target_os = "macos", debug_assertions, not(feature = "xpc"))
))]
pub async fn connect(socket_path: std::path::PathBuf) -> Result<TokioIo<tokio::net::UnixStream>> {
    let conn = tokio::net::UnixStream::connect(socket_path).await?;
    accepted(conn).await
}

#[cfg(target_os = "windows")]
pub async fn connect(
    socket_path: std::path::PathBuf,
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
