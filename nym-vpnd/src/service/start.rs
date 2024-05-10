// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc::UnboundedReceiver;
use tracing::info;

use super::vpn_service::NymVpnService;
use super::VpnServiceCommand;

pub(crate) fn start_vpn_service(
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    mut task_client: nym_task::TaskClient,
) -> std::thread::JoinHandle<()> {
    info!("Starting VPN service");

    // TODO: join up the task handling in vpn library with the daemon
    task_client.disarm();

    std::thread::spawn(move || {
        let vpn_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        vpn_rt.block_on(async {
            NymVpnService::new(vpn_command_rx).run().await.ok();
        });
    })
}
