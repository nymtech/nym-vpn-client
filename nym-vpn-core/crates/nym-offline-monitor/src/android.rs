// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use super::Connectivity;

pub struct MonitorHandle;

impl MonitorHandle {
    #[allow(clippy::unused_async)]
    pub async fn connectivity(&self) -> Connectivity {
        Connectivity::PresumeOnline
    }
}

#[derive(Debug)]
pub struct Error;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generic error")
    }
}

#[allow(clippy::unused_async)]
pub async fn spawn_monitor(
    _sender: watch::Sender<Connectivity>,
    _shutdown_token: CancellationToken,
) -> Result<MonitorHandle, Error> {
    // todo: implement
    Ok(MonitorHandle)
}
