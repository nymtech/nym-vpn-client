// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot;
use tracing::{error, info};

use super::vpn_service::{SharedVpnState, VpnState};

pub(super) struct VpnServiceExitListener {
    shared_vpn_state: SharedVpnState,
}

impl VpnServiceExitListener {
    pub(super) fn new(shared_vpn_state: SharedVpnState) -> Self {
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
                        self.shared_vpn_state.set(VpnState::NotConnected);
                        listener_vpn_exit_tx.send(exit_res).ok();
                    }
                    nym_vpn_lib::NymVpnExitStatusMessage::Failed(ref err) => {
                        error!("VPN exit: fail: {err}");
                        self.shared_vpn_state
                            .set(VpnState::ConnectionFailed(err.to_string()));
                        listener_vpn_exit_tx.send(exit_res).ok();
                    }
                },
                Err(err) => {
                    error!("Exit listener: {err}");
                }
            }
        });
    }
}
