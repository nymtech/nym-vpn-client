// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{error::*, NymVpnCtrlMessage};
use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use log::*;
use talpid_routing::RouteManager;
#[cfg(target_os = "linux")]
use talpid_types::ErrorExt;

pub(crate) async fn wait_for_interrupt(mut task_manager: nym_task::TaskManager) {
    if let Err(e) = task_manager.catch_interrupt().await {
        error!("Could not wait for interrupts anymore - {e}. Shutting down the tunnel.");
    }
}

pub(crate) async fn wait_for_interrupt_and_signal(
    mut task_manager: Option<nym_task::TaskManager>,
    mut vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let task_manager_wait = async {
        if let Some(task_manager) = &mut task_manager {
            task_manager.wait_for_error().await
        } else {
            std::future::pending().await
        }
    };
    let res = tokio::select! {
        biased;
        message = vpn_ctrl_rx.next() => {
            log::debug!("Received message: {:?}", message);
            match message {
                Some(NymVpnCtrlMessage::Stop) => {
                    log::info!("Received stop message");
                }
                None => {
                    log::info!("Channel closed, stopping");
                }
            }
            Ok(())
        }
        Some(msg) = task_manager_wait => {
            log::info!("Task error: {:?}", msg);
            Err(msg)
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received SIGINT");
            Ok(())
        },
    };
    if let Some(mut task_manager) = task_manager {
        info!("Sending shutdown signal");
        task_manager.signal_shutdown().ok();

        info!("Waiting for tasks to finish... (Press ctrl-c to force)");
        task_manager.wait_for_shutdown().await;

        info!("Stopping mixnet client");
    }
    res
}

#[cfg_attr(target_os = "windows", allow(unused_mut))]
pub(crate) async fn handle_interrupt(
    mut route_manager: RouteManager,
    wireguard_waiting: Option<(oneshot::Receiver<()>, tokio::task::JoinHandle<Result<()>>)>,
    tunnel_close_tx: oneshot::Sender<()>,
) -> Result<()> {
    let is_wireguard_waiting = wireguard_waiting.is_some();

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
        if is_wireguard_waiting {
            tunnel_close_tx
                .send(())
                .map_err(|_| Error::FailedToSendWireguardTunnelClose)?;
        }
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

#[cfg(unix)]
pub fn unix_has_root(binary_name: &str) -> Result<()> {
    if nix::unistd::geteuid().is_root() {
        debug!("Root privileges acquired");
        Ok(())
    } else {
        Err(Error::RootPrivilegesRequired {
            binary_name: binary_name.to_string(),
        })
    }
}

#[cfg(windows)]
pub fn win_has_admin(binary_name: &str) -> Result<()> {
    if is_elevated::is_elevated() {
        debug!("Admin privileges acquired");
        Ok(())
    } else {
        Err(Error::AdminPrivilegesRequired {
            binary_name: binary_name.to_string(),
        })
    }
}
