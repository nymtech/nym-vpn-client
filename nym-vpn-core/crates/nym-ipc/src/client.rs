// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io::Result, path::PathBuf};

use hyper_util::rt::TokioIo;

#[cfg(unix)]
pub async fn connect(socket_path: PathBuf) -> Result<TokioIo<tokio::net::UnixStream>> {
    crate::uds::connect(socket_path).await
}

#[cfg(windows)]
pub async fn connect(
    socket_path: PathBuf,
) -> Result<TokioIo<tokio::net::windows::named_pipe::NamedPipeClient>> {
    crate::named_pipe::connect(socket_path.into_os_string()).await
}
