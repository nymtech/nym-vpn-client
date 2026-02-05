// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::net::UnixStream;
use tokio_util::sync::CancellationToken;

use crate::authentication::error::AuthenticationError;

// Not implemented yet, implicit is to consider it authenticated
pub(crate) async fn is_authenticated(
    _stream: &mut UnixStream,
    _nym_certificate_serial_number: String,
    _shutdown_token: CancellationToken,
) -> Result<(), AuthenticationError> {
    Ok(())
}
