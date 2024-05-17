// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::oneshot;
use tracing::{debug, error, info};

use super::{
    error::ConnectionFailedError,
    vpn_service::{SharedVpnState, VpnState},
};

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
                        debug!("VPN exit: fail: {err:?}");

                        // Inspect the error so we can set a strongly typed error state
                        let vpn_lib_err = err
                            .downcast_ref::<nym_vpn_lib::error::Error>()
                            .map(ConnectionFailedError::from);

                        let connection_failed_err = match vpn_lib_err {
                            Some(vpn_lib_err) => vpn_lib_err,
                            None => ConnectionFailedError::Unhandled(err.to_string()),
                        };

                        self.shared_vpn_state
                            .set(VpnState::ConnectionFailed(connection_failed_err));
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
