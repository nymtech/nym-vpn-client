// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::*;
use futures::channel::oneshot;
use log::*;
use talpid_routing::RouteManager;
#[cfg(target_os = "linux")]
use talpid_types::ErrorExt;

pub(crate) async fn wait_for_interrupt(task_manager: nym_task::TaskManager) {
    if let Err(e) = task_manager.catch_interrupt().await {
        error!("Could not wait for interrupts anymore - {e}. Shutting down the tunnel.");
    }
}

pub(crate) async fn handle_interrupt(
    mut route_manager: RouteManager,
    wireguard_waiting: Option<(oneshot::Receiver<()>, tokio::task::JoinHandle<Result<()>>)>,
    tunnel_close_tx: oneshot::Sender<()>,
) -> Result<()> {
    let sig_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        debug!("Received interrupt signal");
        route_manager.clear_routes()?;
        #[cfg(target_os = "linux")]
        if let Err(error) =
            tokio::runtime::Handle::current().block_on(route_manager.clear_routing_rules())
        {
            error!(
                "{}",
                error.display_chain_with_msg("Failed to clear routing rules")
            );
        }
        tunnel_close_tx
            .send(())
            .map_err(|_| Error::OneshotSendError)?;
        Ok(())
    });

    if let Some((finished_shutdown_rx, tunnel_handle)) = wireguard_waiting {
        tunnel_handle.await??;
        sig_handle.await??;
        finished_shutdown_rx.await?;
    } else {
        sig_handle.await??;
    }
    Ok(())
}
