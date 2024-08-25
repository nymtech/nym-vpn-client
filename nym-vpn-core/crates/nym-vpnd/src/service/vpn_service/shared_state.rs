// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::info;

use crate::service::ConnectionFailedError;

use super::VpnState;

#[derive(Clone)]
pub(crate) struct SharedVpnState {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>,
}

impl SharedVpnState {
    pub(super) fn new(vpn_state_changes_tx: broadcast::Sender<VpnServiceStateChange>) -> Self {
        Self {
            shared_vpn_state: Arc::new(std::sync::Mutex::new(VpnState::NotConnected)),
            vpn_state_changes_tx,
        }
    }

    pub(crate) fn set(&self, state: VpnState) {
        info!("VPN: Setting shared state to {}", state);
        *self.shared_vpn_state.lock().unwrap() = state.clone();
        self.vpn_state_changes_tx.send(state.into()).ok();
    }

    pub(super) fn get(&self) -> VpnState {
        self.shared_vpn_state.lock().unwrap().clone()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum VpnServiceStateChange {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
    ConnectionFailed(ConnectionFailedError),
}

impl From<VpnState> for VpnServiceStateChange {
    fn from(state: VpnState) -> Self {
        match state {
            VpnState::NotConnected => VpnServiceStateChange::NotConnected,
            VpnState::Connecting => VpnServiceStateChange::Connecting,
            VpnState::Connected { .. } => VpnServiceStateChange::Connected,
            VpnState::Disconnecting => VpnServiceStateChange::Disconnecting,
            VpnState::ConnectionFailed(reason) => VpnServiceStateChange::ConnectionFailed(reason),
        }
    }
}
