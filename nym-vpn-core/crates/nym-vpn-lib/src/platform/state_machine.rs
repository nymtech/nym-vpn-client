// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_common::ErrorExt;
use nym_vpn_account_controller::AccountCommandSender;
use nym_vpn_network_config::Network;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

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
    account_controller_tx: AccountCommandSender,
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

fn setup_statistics_recipient(
    config: &VPNConfig,
    network_env: &Network,
) -> Option<Box<nym_gateway_directory::Recipient>> {
    // The statistics recipient can be set in the system configuration
    let statistics_recipient_from_system = network_env
        .system_configuration
        .as_ref()
        .and_then(|sc| sc.statistics_recipient)
        .map(Box::new);

    // The statistics recipient can also be set in the app configuration
    let statistics_recipient_from_app = config
        .statistics_recipient
        .clone()
        .map(nym_gateway_directory::Recipient::try_from_base58_string)
        .transpose()
        .inspect_err(|err| {
            err.trace_chain_with_msg("Failed to parse statistics recipient");
        })
        .unwrap_or_default()
        .map(Box::new);

    // We use the statistics recipient from the app configuration if it is set, otherwise we use
    // the one from the system configuration
    statistics_recipient_from_app.or(statistics_recipient_from_system)
}

pub(super) async fn start_state_machine(
    config: VPNConfig,
    network_env: Network,
    enable_credentials_mode: bool,
    account_controller_tx: AccountCommandSender,
) -> Result<StateMachineHandle, VpnError> {
    let tunnel_type = if config.enable_two_hop {
        TunnelType::Wireguard
    } else {
        TunnelType::Mixnet
    };

    let statistics_recipient = setup_statistics_recipient(&config, &network_env);

    let entry_point = nym_gateway_directory::EntryPoint::from(config.entry_gateway);
    let exit_point = nym_gateway_directory::ExitPoint::from(config.exit_router);

    // Bootstrap the state machines gateway client with the static gateway client, so that we can
    // use the existing cached directory data.
    let static_gw_client = super::init_static_gateway_client(config.user_agent.clone()).await?;
    let gateway_directory_client =
        nym_gateway_directory::CachingGatewayClient::new_from_existing(&static_gw_client).await;
    let gateway_config = gateway_directory_client.get_config().await;

    let nym_config = NymConfig {
        config_path: config.config_path,
        data_path: config.credential_data_path,
        gateway_config: gateway_config.clone(),
        network_env,
    };

    let user_agent = nym_sdk::UserAgent::from(config.user_agent.clone());

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
        user_agent: Some(user_agent.clone()),
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

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let route_handler = crate::tunnel_state_machine::RouteHandler::new()
        .await
        .map_err(crate::tunnel_state_machine::Error::CreateRouteHandler)?;

    let connectivity_handle = nym_offline_monitor::spawn_monitor(
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        route_handler.inner_handle(),
        #[cfg(target_os = "android")]
        crate::tunnel_state_machine::AndroidConnectivityAdapter::new(config.tun_provider.clone()),
        #[cfg(target_os = "linux")]
        Some(crate::tunnel_state_machine::TUNNEL_FWMARK),
    )
    .await;

    gateway_directory_client
        .set_connectivity_handle(offline_monitor.clone())
        .await;

    let shutdown_token = CancellationToken::new();
    let state_machine_handle = TunnelStateMachine::spawn(
        command_receiver,
        event_sender,
        nym_config,
        tunnel_settings,
        account_controller_tx,
        gateway_directory_client,
        connectivity_handle,
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        route_handler,
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
            tracing::error!("Failed to send tunnel command: {}", e);
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
