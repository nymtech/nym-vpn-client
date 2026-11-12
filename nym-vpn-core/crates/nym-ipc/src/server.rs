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

/// Credentials of the client that sent the request, as recorded by the transport.
///
/// XPC provides the user and the audit session. The Unix socket used in debug builds only
/// provides the user.
#[cfg(target_os = "macos")]
pub fn peer_credentials(extensions: &tonic::Extensions) -> Option<crate::PeerCredentials> {
    #[cfg(any(not(debug_assertions), feature = "xpc"))]
    {
        extensions
            .get::<Option<crate::PeerCredentials>>()
            .copied()
            .flatten()
    }

    #[cfg(all(debug_assertions, not(feature = "xpc")))]
    {
        extensions
            .get::<tonic::transport::server::UdsConnectInfo>()
            .and_then(|info| info.peer_cred)
            .map(|cred| crate::PeerCredentials {
                user_id: cred.uid(),
                audit_session_id: None,
            })
    }
}
