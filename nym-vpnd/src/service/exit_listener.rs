// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use futures::channel::oneshot;
use tracing::{error, info};

use super::vpn_service::VpnState;

pub(super) struct VpnServiceExitListener {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
}

impl VpnServiceExitListener {
    pub(super) fn new(shared_vpn_state: Arc<std::sync::Mutex<VpnState>>) -> Self {
        Self { shared_vpn_state }
    }

    pub(super) async fn start(
        self,
        vpn_exit_rx: oneshot::Receiver<nym_vpn_lib::NymVpnExitStatusMessage>,
        listener_vpn_exit_tx: oneshot::Sender<nym_vpn_lib::NymVpnExitStatusMessage>,
    ) {
        tokio::spawn(async move {
            match vpn_exit_rx.await {
                Ok(exit_res) => match exit_res {
                    nym_vpn_lib::NymVpnExitStatusMessage::Stopped => {
                        info!("VPN exit: stopped");
                        self.set_shared_state(VpnState::NotConnected);
                        if let Err(err) = listener_vpn_exit_tx.send(exit_res) {
                            error!("Exit listener: failed to send exit status: {err:?}");
                        }
                    }
                    nym_vpn_lib::NymVpnExitStatusMessage::Failed(ref err) => {
                        error!("VPN exit: fail: {err}");
                        self.set_shared_state(VpnState::ConnectionFailed(err.to_string()));
                        if let Err(err) = listener_vpn_exit_tx.send(exit_res) {
                            error!("Exit listener: failed to send exit status: {err:?}");
                        }
                    }
                },
                Err(err) => {
                    error!("Exit listener: {err}");
                }
            }
        });
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }
}
