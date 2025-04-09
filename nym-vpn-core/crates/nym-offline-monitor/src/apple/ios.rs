// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use super::path_monitor;
pub use super::path_monitor::ConnectivityHandle;
use crate::Connectivity;

pub async fn spawn_monitor(
    sender: watch::Sender<Connectivity>,
    shutdown_token: CancellationToken,
) -> Result<ConnectivityHandle, nym_common::BoxedError> {
    Ok(path_monitor::spawn_monitor(sender, shutdown_token.clone()).await)
}
