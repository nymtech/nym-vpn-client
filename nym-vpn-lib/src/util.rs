// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{error::*, tunnel_setup::WgTunnelSetup, NymVpnCtrlMessage};
use futures::{channel::mpsc, StreamExt};
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
    wireguard_waiting: Option<[WgTunnelSetup; 2]>,
) -> Result<()> {
    let (tunnel_close_tx, finished_shutdown_rx, tunnel_handle) = match wireguard_waiting {
        Some([entry_setup, exit_setup]) => (
            Some([entry_setup.tunnel_close_tx, exit_setup.tunnel_close_tx]),
            Some([entry_setup.receiver, exit_setup.receiver]),
            Some([entry_setup.handle, exit_setup.handle]),
        ),
        None => (None, None, None),
    };

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

        if let Some([entry_tx, exit_tx]) = tunnel_close_tx {
            let ret1 = entry_tx.send(());
            let ret2 = exit_tx.send(());
            if ret1.is_err() || ret2.is_err() {
                return Err(Error::FailedToSendWireguardTunnelClose);
            }
        }
        Ok(())
    });

    if let Some([h1, h2]) = tunnel_handle {
        let ret1 = h1.await;
        let ret2 = h2.await;
        ret1??;
        ret2??;
    }
    sig_handle.await??;
    if let Some([rx1, rx2]) = finished_shutdown_rx {
        let ret1 = rx1.await;
        let ret2 = rx2.await;
        ret1?;
        ret2?;
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
