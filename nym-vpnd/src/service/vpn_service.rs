// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use nym_vpn_lib::gateway_directory;
use nym_vpn_lib::nym_config::OptionalSet;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;
use tracing::info;

use super::exit_listener::VpnServiceExitListener;
use super::status_listener::VpnServiceStatusListener;

#[derive(Debug, Clone)]
pub enum VpnState {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

#[derive(Debug)]
pub enum VpnServiceCommand {
    Connect(oneshot::Sender<VpnServiceConnectResult>),
    Disconnect(oneshot::Sender<VpnServiceDisconnectResult>),
    Status(oneshot::Sender<VpnServiceStatusResult>),
}

#[derive(Debug)]
pub enum VpnServiceConnectResult {
    Success,
    #[allow(unused)]
    Fail(String),
}

#[derive(Debug)]
pub enum VpnServiceDisconnectResult {
    Success,
    NotRunning,
    #[allow(unused)]
    Fail(String),
}

#[derive(Copy, Clone, Debug)]
pub enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected,
    Disconnecting,
}

pub(super) struct NymVpnService {
    shared_vpn_state: Arc<std::sync::Mutex<VpnState>>,
    vpn_command_rx: UnboundedReceiver<VpnServiceCommand>,
    vpn_ctrl_sender: Option<UnboundedSender<nym_vpn_lib::NymVpnCtrlMessage>>,
}

impl NymVpnService {
    pub(super) fn new(vpn_command_rx: UnboundedReceiver<VpnServiceCommand>) -> Self {
        Self {
            shared_vpn_state: Arc::new(std::sync::Mutex::new(VpnState::NotConnected)),
            vpn_command_rx,
            vpn_ctrl_sender: None,
        }
    }

    async fn handle_connect(&mut self) -> VpnServiceConnectResult {
        self.set_shared_state(VpnState::Connecting);

        // TODO: read from config file
        let mut nym_vpn = nym_vpn_lib::NymVpn::new(
            gateway_directory::EntryPoint::Random,
            gateway_directory::ExitPoint::Random,
        );

        nym_vpn.gateway_config = gateway_directory::Config::default()
            .with_optional_env(
                gateway_directory::Config::with_custom_api_url,
                None,
                "NYM_API",
            )
            .with_optional_env(
                gateway_directory::Config::with_custom_explorer_url,
                None,
                "EXPLORER_API",
            );

        let handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(nym_vpn).unwrap();

        let nym_vpn_lib::NymVpnHandle {
            vpn_ctrl_tx,
            vpn_status_rx,
            vpn_exit_rx,
        } = handle;

        self.vpn_ctrl_sender = Some(vpn_ctrl_tx);

        VpnServiceStatusListener::new(self.shared_vpn_state.clone())
            .start(vpn_status_rx)
            .await;

        VpnServiceExitListener::new(self.shared_vpn_state.clone())
            .start(vpn_exit_rx)
            .await;

        VpnServiceConnectResult::Success
    }

    fn set_shared_state(&self, state: VpnState) {
        *self.shared_vpn_state.lock().unwrap() = state;
    }

    async fn handle_disconnect(&mut self) -> VpnServiceDisconnectResult {
        // To handle the mutable borrow we set the state separate from the sending the stop
        // message, including the logical check for the ctrl sender twice.
        let is_running = self.vpn_ctrl_sender.is_some();

        if is_running {
            self.set_shared_state(VpnState::Disconnecting);
        }

        if let Some(ref mut vpn_ctrl_sender) = self.vpn_ctrl_sender {
            let _ = vpn_ctrl_sender
                .send(nym_vpn_lib::NymVpnCtrlMessage::Stop)
                .await;
            VpnServiceDisconnectResult::Success
        } else {
            VpnServiceDisconnectResult::NotRunning
        }
    }

    async fn handle_status(&self) -> VpnServiceStatusResult {
        match *self.shared_vpn_state.lock().unwrap() {
            VpnState::NotConnected => VpnServiceStatusResult::NotConnected,
            VpnState::Connecting => VpnServiceStatusResult::Connecting,
            VpnState::Connected => VpnServiceStatusResult::Connected,
            VpnState::Disconnecting => VpnServiceStatusResult::Disconnecting,
        }
    }

    pub(super) async fn run(mut self) {
        while let Some(command) = self.vpn_command_rx.recv().await {
            info!("VPN: Received command: {:?}", command);
            match command {
                VpnServiceCommand::Connect(tx) => {
                    let result = self.handle_connect().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Disconnect(tx) => {
                    let result = self.handle_disconnect().await;
                    tx.send(result).unwrap();
                }
                VpnServiceCommand::Status(tx) => {
                    let result = self.handle_status().await;
                    tx.send(result).unwrap();
                }
            }
        }
    }
}
