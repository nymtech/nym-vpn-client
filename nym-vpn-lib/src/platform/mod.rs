// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::{spawn_nym_vpn, NymVpn, NymVpnCtrlMessage, NymVpnExitError, NymVpnExitStatusMessage};
use futures::StreamExt;
use lazy_static::lazy_static;
use log::*;
use nym_task::manager::TaskStatus;
use std::sync::Arc;
use talpid_core::mpsc::Sender;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, Notify};

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "macos")]
pub mod macos;

lazy_static! {
    static ref VPN_SHUTDOWN_HANDLE: Mutex<Option<Arc<Notify>>> = Mutex::new(None);
    static ref VPN: Mutex<Option<NymVpn>> = Mutex::new(None);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

#[derive(Eq, PartialEq, Debug)]
pub enum ClientState {
    Uninitialised,
    Connected,
    Disconnected,
}

async fn is_vpn_inited() -> bool {
    let guard = VPN.lock().await;
    guard.is_some()
}

async fn take_vpn() -> Option<NymVpn> {
    let mut guard = VPN.lock().await;
    guard.take()
}

async fn is_shutdown_handle_set() -> bool {
    VPN_SHUTDOWN_HANDLE.lock().await.is_some()
}

pub async fn get_vpn_state() -> ClientState {
    if !is_vpn_inited().await {
        ClientState::Uninitialised
    } else if is_shutdown_handle_set().await {
        ClientState::Connected
    } else {
        ClientState::Disconnected
    }
}

async fn set_shutdown_handle(handle: Arc<Notify>) {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if guard.is_some() {
        panic!("vpn wasn't properly stopped")
    }
    *guard = Some(handle)
}

async fn set_inited_vpn(vpn: NymVpn) {
    let mut guard = VPN.lock().await;
    if guard.is_some() {
        panic!("vpn was already inited");
    }
    *guard = Some(vpn)
}

async fn stop_and_reset_shutdown_handle() {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if let Some(sh) = &*guard {
        sh.notify_waiters()
    } else {
        panic!("client wasn't properly started")
    }

    *guard = None
}

async fn _async_run_vpn(vpn: NymVpn) -> crate::error::Result<()> {
    let stop_handle = Arc::new(Notify::new());
    set_shutdown_handle(stop_handle.clone()).await;

    let mut handle = spawn_nym_vpn(vpn)?;

    match handle
        .vpn_status_rx
        .next()
        .await
        .ok_or(crate::Error::NotStarted)?
        .downcast_ref::<TaskStatus>()
        .ok_or(crate::Error::NotStarted)?
    {
        TaskStatus::Ready => debug!("Started Nym VPN"),
    }

    // wait for notify to be set...
    stop_handle.notified().await;
    handle.vpn_ctrl_tx.send(NymVpnCtrlMessage::Stop)?;
    match handle.vpn_exit_rx.await? {
        NymVpnExitStatusMessage::Failed(error) => {
            error!(
                "{:?}",
                error
                    .downcast_ref::<NymVpnExitError>()
                    .ok_or(crate::Error::StopError)?
            );
        }
        NymVpnExitStatusMessage::Stopped => debug!("Stopped Nym VPN"),
    }

    Ok(())
}
