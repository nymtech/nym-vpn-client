// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::net::UnixStream;

use crate::authentication::error::AuthenticationError;

pub(crate) async fn is_authenticated(
    stream: UnixStream,
    _shutdown_token: CancellationToken,
) -> Result<UnixStream, AuthenticationError> {
    Ok(stream)
}
