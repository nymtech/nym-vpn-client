// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::Result;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::Stream;
use tonic::transport::server::Connected;

#[allow(unused)]
pub fn create_incoming(
    socket_path: std::path::PathBuf,
    #[cfg(target_os = "windows")] nym_certificate_serial_number: String,
    #[cfg(unix)] shutdown_token: tokio_util::sync::CancellationToken,
) -> Result<impl Stream<Item = Result<impl AsyncRead + AsyncWrite + Connected + 'static>>> {
    #[cfg(target_os = "macos")]
    {
        #[cfg(all(debug_assertions, not(feature = "xpc")))]
        {
            crate::uds::incoming(socket_path, shutdown_token)
        }
        #[cfg(any(not(debug_assertions), feature = "xpc"))]
        {
            crate::xpc::incoming(shutdown_token)
        }
    }
    #[cfg(target_os = "linux")]
    {
        crate::uds::incoming(socket_path, shutdown_token)
    }

    #[cfg(target_os = "windows")]
    {
        crate::named_pipe::incoming(socket_path.into_os_string(), nym_certificate_serial_number)
    }
}
