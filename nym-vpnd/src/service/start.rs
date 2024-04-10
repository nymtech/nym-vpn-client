// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc::Receiver;
use tracing::info;

use super::vpn_service::NymVpnService;
use super::VpnServiceCommand;

pub(crate) fn start_vpn_service(
    vpn_command_rx: Receiver<VpnServiceCommand>,
    mut task_client: nym_task::TaskClient,
) -> std::thread::JoinHandle<()> {
    info!("Starting VPN handler");
    task_client.mark_as_success();
    std::thread::spawn(move || {
        let vpn_rt = tokio::runtime::Runtime::new().unwrap();
        vpn_rt.block_on(async {
            // Listen to the command channel
            info!("VPN: Listening for commands");
            let vpn_service = NymVpnService::new(vpn_command_rx);
            vpn_service.run().await;
        });
    })
    // TEMP
}
