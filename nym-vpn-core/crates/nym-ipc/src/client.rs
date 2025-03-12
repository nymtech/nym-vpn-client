// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[cfg(unix)]
pub async fn connect(
    socket_path: PathBuf,
) -> std::io::Result<hyper_util::rt::TokioIo<tokio::net::UnixStream>> {
    crate::uds::connect(socket_path).await
}

#[cfg(windows)]
pub async fn connect(
    socket_path: PathBuf,
) -> std::io::Result<hyper_util::rt::TokioIo<tokio::net::windows::named_pipe::NamedPipeClient>> {
    crate::named_pipe::connect(socket_path.into_os_string()).await
}
