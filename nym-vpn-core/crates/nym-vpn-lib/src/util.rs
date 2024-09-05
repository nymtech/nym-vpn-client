// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{channel::mpsc, StreamExt};
use talpid_routing::RouteManager;
use tracing::{error, info};

use crate::{
    error::{Error, Result},
    tunnel_setup::WgTunnelSetup,
    vpn::NymVpnCtrlMessage,
};

pub(crate) async fn wait_for_interrupt(
    mut task_manager: Option<nym_task::TaskManager>,
    mut vpn_ctrl_rx: Option<mpsc::UnboundedReceiver<NymVpnCtrlMessage>>,
    route_manager: RouteManager,
    wireguard_waiting: Option<[WgTunnelSetup; 2]>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let task_manager_wait = async {
        if let Some(task_manager) = &mut task_manager {
            task_manager.wait_for_error().await
        } else {
            std::future::pending().await
        }
    };

    let vpn_ctrl_rx_wait = async {
        if let Some(vpn_ctrl_rx) = &mut vpn_ctrl_rx {
            vpn_ctrl_rx.next().await
        } else {
            std::future::pending().await
        }
    };

    let res = tokio::select! {
        biased;
        Some(message) = vpn_ctrl_rx_wait => match message {
            NymVpnCtrlMessage::Stop => {
                log::info!("Received stop message");
                Ok(())
            }
        },
        Some(msg) = task_manager_wait => {
            log::info!("Task error: {:?}", msg);
            Err(msg)
        }
        else => {
            log::error!("Unexpected channel close when waiting for interrupt");
            Ok(())
        }
    };

    if let Some(mut task_manager) = task_manager {
        info!("Sending shutdown signal");
        task_manager.signal_shutdown().ok();

        handle_interrupt(route_manager, wireguard_waiting).await;

        info!("Waiting for tasks to finish... (Press ctrl-c to force)");
        // TODO: this contains another signal handler that needs to be moved out.
        task_manager.wait_for_shutdown().await;

        info!("Stopping mixnet client");
    }
    res
}

#[cfg_attr(target_os = "windows", allow(unused_mut))]
pub(crate) async fn handle_interrupt(
    route_manager: RouteManager,
    wireguard_waiting: Option<[WgTunnelSetup; 2]>,
) {
    tokio::task::spawn_blocking(|| drop(route_manager))
        .await
        .ok();
    let Some(wireguard_waiting) = wireguard_waiting else {
        return;
    };
    let [entry, exit] = wireguard_waiting;

    entry.tunnel_close_tx.send(()).ok();
    if let Err(err) = entry.receiver.await {
        error!("Error on entry signal handle {}", err);
    }
    if let Err(err) = entry.handle.await {
        error!("Error on entry tunnel handle {}", err);
    }

    exit.tunnel_close_tx.send(()).ok();
    if let Err(err) = exit.receiver.await {
        error!("Error on exit signal handle {}", err);
    }
    if let Err(err) = exit.handle.await {
        error!("Error on exit tunnel handle {}", err);
    }
}
