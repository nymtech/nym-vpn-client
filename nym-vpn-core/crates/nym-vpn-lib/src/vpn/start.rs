// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::{mpsc, oneshot};
use tracing::{debug, error, info};

use super::{NymVpnCtrlMessage, NymVpnExitStatusMessage, SpecificVpn};
use crate::{
    error::Result,
    uniffi_custom_impls::{ExitStatus, StatusEvent},
    Error,
};

/// Starts the Nym VPN client.
///
/// Examples
///
/// ```no_run
/// use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
/// use nym_vpn_lib::NodeIdentity;
///
/// let mut vpn_config = nym_vpn_lib::NymVpn::new_mixnet_vpn(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn(vpn_config.into());
/// ```
///
/// ```no_run
/// use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
/// use nym_vpn_lib::NodeIdentity;
///
/// let mut vpn_config = nym_vpn_lib::NymVpn::new_wireguard_vpn(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn(vpn_config.into());
/// ```
pub fn spawn_nym_vpn(nym_vpn: SpecificVpn) -> Result<NymVpnHandle> {
    let (vpn_ctrl_tx, vpn_ctrl_rx) = mpsc::unbounded();
    let (vpn_status_tx, vpn_status_rx) = mpsc::channel(128);
    let (vpn_exit_tx, vpn_exit_rx) = oneshot::channel();

    tokio::spawn(run_nym_vpn(
        nym_vpn,
        vpn_status_tx,
        vpn_ctrl_rx,
        vpn_exit_tx,
    ));

    Ok(NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    })
}

/// Starts the Nym VPN client, in a separate tokio runtime.
///
/// Examples
///
/// ```no_run
/// use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
/// use nym_vpn_lib::NodeIdentity;
///
/// let mut vpn_config = nym_vpn_lib::NymVpn::new_mixnet_vpn(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(vpn_config.into());
/// ```
///
/// ```no_run
/// use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
/// use nym_vpn_lib::NodeIdentity;
///
/// let mut vpn_config = nym_vpn_lib::NymVpn::new_wireguard_vpn(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(vpn_config.into());
/// ```
pub fn spawn_nym_vpn_with_new_runtime(nym_vpn: SpecificVpn) -> Result<NymVpnHandle> {
    let (vpn_ctrl_tx, vpn_ctrl_rx) = mpsc::unbounded();
    let (vpn_status_tx, vpn_status_rx) = mpsc::channel(128);
    let (vpn_exit_tx, vpn_exit_rx) = oneshot::channel();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio run time");
        rt.block_on(run_nym_vpn(
            nym_vpn,
            vpn_status_tx,
            vpn_ctrl_rx,
            vpn_exit_tx,
        ));
    });

    Ok(NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    })
}

async fn run_nym_vpn(
    mut nym_vpn: SpecificVpn,
    vpn_status_tx: nym_task::StatusSender,
    vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
    vpn_exit_tx: oneshot::Sender<NymVpnExitStatusMessage>,
) {
    match nym_vpn.run(vpn_status_tx, vpn_ctrl_rx).await {
        Ok(()) => {
            info!("Nym VPN has shut down");
            vpn_exit_tx
                .send(NymVpnExitStatusMessage::Stopped)
                .expect("Failed to send exit status");
        }
        Err(err) => {
            error!("Nym VPN returned error: {err}");
            debug!("{err:?}");
            crate::platform::uniffi_set_listener_status(StatusEvent::Exit(
                ExitStatus::TunnelFailure {
                    message: err.to_string()
                }
            ));
            vpn_exit_tx
                .send(NymVpnExitStatusMessage::Failed(err))
                .expect("Failed to send exit status");
        }
    }
}

pub struct NymVpnHandle {
    pub vpn_ctrl_tx: mpsc::UnboundedSender<NymVpnCtrlMessage>,
    pub vpn_status_rx: nym_task::StatusReceiver,
    pub vpn_exit_rx: oneshot::Receiver<NymVpnExitStatusMessage>,
}

impl NymVpnHandle {
    pub fn ctrl_tx(&self) -> mpsc::UnboundedSender<NymVpnCtrlMessage> {
        self.vpn_ctrl_tx.clone()
    }

    pub async fn wait_until_stopped(self) -> Result<()> {
        match self.vpn_exit_rx.await {
            Ok(NymVpnExitStatusMessage::Stopped) => {
                debug!("VPN stopped");
                Ok(())
            }
            Ok(NymVpnExitStatusMessage::Failed(err)) => {
                debug!("VPN exited with error: {:?}", err);
                Err(Error::NymVpnExitWithError(err))
            }
            Err(err) => {
                debug!("VPN unexpected exit with error: {:?}", err);
                Err(Error::NymVpnExitUnexpectedChannelClose)
            }
        }
    }
}
