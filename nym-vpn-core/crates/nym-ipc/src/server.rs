// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::Result;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::Stream;
use tonic::transport::server::Connected;

use crate::authentication::AuthenticationMaterial;

#[allow(unused)]
pub async fn create_incoming(
    socket_path: std::path::PathBuf,
    auth_material: AuthenticationMaterial,
) -> Result<impl Stream<Item = Result<impl AsyncRead + AsyncWrite + Connected + 'static>>> {
    #[cfg(target_os = "macos")]
    {
        #[cfg(all(debug_assertions, not(feature = "xpc")))]
        {
            crate::uds::incoming(socket_path, auth_material).await
        }
        #[cfg(any(not(debug_assertions), feature = "xpc"))]
        {
            crate::xpc::incoming(auth_material)
        }
    }
    #[cfg(target_os = "linux")]
    {
        crate::uds::incoming(socket_path, auth_material).await
    }

    #[cfg(target_os = "windows")]
    {
        crate::named_pipe::incoming(socket_path.into_os_string(), auth_material)
    }
}
