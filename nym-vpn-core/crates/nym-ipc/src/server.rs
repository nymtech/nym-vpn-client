// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{io::Result, path::PathBuf};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;
use tonic::transport::server::Connected;

pub fn create_incoming(
    socket_path: PathBuf,
    _shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = Result<impl AsyncRead + AsyncWrite + Connected + 'static>>> {
    #[cfg(unix)]
    {
        crate::uds::incoming(socket_path, _shutdown_token)
    }

    #[cfg(windows)]
    {
        crate::named_pipe::incoming(socket_path.into_os_string())
    }
}
