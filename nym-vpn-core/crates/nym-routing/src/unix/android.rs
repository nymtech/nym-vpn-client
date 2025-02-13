// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::imp::RouteManagerCommand;
use tokio::sync::mpsc;

/// Stub error type for routing errors on Android.
#[derive(Debug, thiserror::Error)]
#[error("Failed to send shutdown result")]
pub struct Error;

/// Stub route manager for Android
pub struct RouteManagerImpl {}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new() -> Result<Self, Error> {
        Ok(RouteManagerImpl {})
    }

    pub(crate) async fn run(
        self,
        mut manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<(), Error> {
        while let Some(command) = manage_rx.recv().await {
            match command {
                RouteManagerCommand::Shutdown(tx) => {
                    tx.send(()).map_err(|()| Error)?;
                    break;
                }
                RouteManagerCommand::AddRoutes(_routes, tx) => {
                    let _ = tx.send(Ok(()));
                }
                RouteManagerCommand::ClearRoutes => (),
            }
        }
        Ok(())
    }
}
