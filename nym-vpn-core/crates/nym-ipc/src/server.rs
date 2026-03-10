// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::Result;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::Stream;
use tonic::transport::server::Connected;

#[allow(unused)]
pub async fn create_incoming(
    #[cfg(target_os = "windows")] nym_certificate_serial_number: String,
    #[cfg(target_os = "macos")] signing_requirement: String,
    #[cfg(unix)] shutdown_token: tokio_util::sync::CancellationToken,
) -> Result<impl Stream<Item = Result<impl AsyncRead + AsyncWrite + Connected + 'static>>> {
    #[cfg(target_os = "macos")]
    {
        #[cfg(all(debug_assertions, not(feature = "xpc")))]
        {
            crate::uds::incoming(shutdown_token).await
        }
        #[cfg(any(not(debug_assertions), feature = "xpc"))]
        {
            crate::xpc::incoming(signing_requirement, shutdown_token)
        }
    }
    #[cfg(target_os = "linux")]
    {
        crate::uds::incoming(shutdown_token).await
    }

    #[cfg(target_os = "windows")]
    {
        crate::named_pipe::incoming(nym_certificate_serial_number)
    }
}
