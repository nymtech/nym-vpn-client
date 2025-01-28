// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::AccountControllerCommander;
use nym_vpn_network_config::Network;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use nym_gateway_directory::Config as GatewayDirectoryConfig;

use super::TunnelEvent as PlatformTunnelEvent;
use crate::tunnel_state_machine::{
    DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, NymConfig, TunnelCommand,
    TunnelSettings, TunnelStateMachine, WireguardTunnelOptions,
};
use nym_vpn_lib_types::TunnelType;

use super::{error::VpnError, VPNConfig, STATE_MACHINE_HANDLE};

pub(super) async fn init_state_machine(
    config: VPNConfig,
    network_env: Network,
    enable_credentials_mode: bool,
    account_controller_tx: AccountControllerCommander,
) -> Result<(), VpnError> {
    let mut guard = STATE_MACHINE_HANDLE.lock().await;

    if guard.is_none() {
        let state_machine_handle = start_state_machine(
            config,
            network_env,
            enable_credentials_mode,
            account_controller_tx,
        )
        .await?;
        state_machine_handle.send_command(TunnelCommand::Connect);
        *guard = Some(state_machine_handle);
        Ok(())
    } else {
        Err(VpnError::InvalidStateError {
            details: "State machine is already running.".to_owned(),
        })
    }
}

pub(super) async fn start_state_machine(
    config: VPNConfig,
    network_env: Network,
    enable_credentials_mode: bool,
    account_controller_tx: AccountControllerCommander,
) -> Result<StateMachineHandle, VpnError> {
    let tunnel_type = if config.enable_two_hop {
        TunnelType::Wireguard
    } else {
        TunnelType::Mixnet
    };

    let statistics_recipient = config
        .statistics_recipient
        .map(nym_gateway_directory::Recipient::try_from_base58_string)
        .transpose()
        .inspect_err(|err| {
            tracing::error!("Failed to parse statistics recipient: {}", err);
        })
        .unwrap_or_default()
        .map(Box::new);

    let entry_point = nym_gateway_directory::EntryPoint::from(config.entry_gateway);
    let exit_point = nym_gateway_directory::ExitPoint::from(config.exit_router);

    let api_url = network_env.api_url().ok_or(VpnError::InternalError {
        details: "API URL not found".to_string(),
    })?;
    let nyxd_url = network_env.nyxd_url().ok_or(VpnError::InternalError {
        details: "Nyxd URL not found".to_string(),
    })?;
    let nym_vpn_api_url = Some(network_env.vpn_api_url());

    let gateway_config = GatewayDirectoryConfig {
        nyxd_url,
        api_url,
        nym_vpn_api_url,
        ..Default::default()
    };

    let nym_config = NymConfig {
        data_path: config.credential_data_path,
        gateway_config,
    };

    let tunnel_settings = TunnelSettings {
        tunnel_type,
        enable_credentials_mode,
        statistics_recipient,
        mixnet_tunnel_options: MixnetTunnelOptions::default(),
        wireguard_tunnel_options: WireguardTunnelOptions::default(),
        gateway_performance_options: GatewayPerformanceOptions::default(),
        mixnet_client_config: None,
        entry_point: Box::new(entry_point),
        exit_point: Box::new(exit_point),
        dns: DnsOptions::default(),
        user_agent: Some(config.user_agent.into()),
    };

    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

    let state_listener = config.tun_status_listener;
    let event_broadcaster_handler = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            if let Some(ref state_listener) = state_listener {
                let platform_event = PlatformTunnelEvent::from(event);
                (*state_listener).on_event(platform_event);
            }
        }
    });

    let shutdown_token = CancellationToken::new();
    let state_machine_handle = TunnelStateMachine::spawn(
        command_receiver,
        event_sender,
        nym_config,
        tunnel_settings,
        account_controller_tx,
        #[cfg(any(target_os = "ios", target_os = "android"))]
        config.tun_provider,
        shutdown_token.child_token(),
    )
    .await?;

    Ok(StateMachineHandle {
        state_machine_handle,
        event_broadcaster_handler,
        command_sender,
        shutdown_token,
    })
}

pub(super) struct StateMachineHandle {
    state_machine_handle: JoinHandle<()>,
    event_broadcaster_handler: JoinHandle<()>,
    command_sender: mpsc::UnboundedSender<TunnelCommand>,
    shutdown_token: CancellationToken,
}

impl StateMachineHandle {
    fn send_command(&self, command: TunnelCommand) {
        if let Err(e) = self.command_sender.send(command) {
            tracing::error!("Failed to send comamnd: {}", e);
        }
    }

    pub(super) async fn shutdown_and_wait(self) {
        self.shutdown_token.cancel();

        if let Err(e) = self.state_machine_handle.await {
            tracing::error!("Failed to join on state machine handle: {}", e);
        }

        if let Err(e) = self.event_broadcaster_handler.await {
            tracing::error!("Failed to join on event broadcaster handle: {}", e);
        }
    }
}
