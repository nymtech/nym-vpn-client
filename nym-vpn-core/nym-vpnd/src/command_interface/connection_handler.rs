// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::{
    gateway_directory::{EntryPoint, ExitPoint},
    nym_bin_common::bin_info_local_vergen,
};
use time::OffsetDateTime;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::{debug, info, warn};

use crate::{
    service::{
        ConnectArgs, ConnectOptions, ImportCredentialError, VpnServiceCommand,
        VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceInfoResult,
        VpnServiceStatusResult,
    },
    types::gateway,
};

#[derive(Debug, thiserror::Error)]
pub enum ListGatewayError {
    #[error("failed to create gateway directory client: {error}")]
    CreateGatewayDirectoryClient {
        error: nym_vpn_lib::gateway_directory::Error,
    },

    #[error("failed to get entry gateways: {error}")]
    GetEntryGateways {
        error: nym_vpn_lib::gateway_directory::Error,
    },

    #[error("failed to get exit gateways: {error}")]
    GetExitGateways {
        error: nym_vpn_lib::gateway_directory::Error,
    },
}

pub(super) struct CommandInterfaceConnectionHandler {
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
}

impl CommandInterfaceConnectionHandler {
    pub(super) fn new(vpn_command_tx: UnboundedSender<VpnServiceCommand>) -> Self {
        Self { vpn_command_tx }
    }

    pub(crate) async fn handle_connect(
        &self,
        entry: Option<EntryPoint>,
        exit: Option<ExitPoint>,
        options: ConnectOptions,
    ) -> VpnServiceConnectResult {
        info!("Starting VPN");
        let (tx, rx) = oneshot::channel();
        let connect_args = ConnectArgs {
            entry,
            exit,
            options,
        };
        self.vpn_command_tx
            .send(VpnServiceCommand::Connect(tx, connect_args))
            .unwrap();
        debug!("Sent start command to VPN");
        debug!("Waiting for response");
        let result = rx.await.unwrap();
        match result {
            VpnServiceConnectResult::Success(ref _connect_handle) => {
                info!("VPN started successfully");
            }
            VpnServiceConnectResult::Fail(ref err) => {
                info!("VPN failed to start: {err}");
            }
        };
        result
    }

    pub(crate) async fn handle_disconnect(&self) -> VpnServiceDisconnectResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Disconnect(tx))
            .unwrap();
        debug!("Sent stop command to VPN");
        debug!("Waiting for response");
        let result = rx.await.unwrap();
        match result {
            VpnServiceDisconnectResult::Success => {
                debug!("VPN disconnect command sent successfully");
            }
            VpnServiceDisconnectResult::NotRunning => {
                info!("VPN can't stop - it's not running");
            }
            VpnServiceDisconnectResult::Fail(ref err) => {
                warn!("VPN failed to send disconnect command: {err}");
            }
        };
        result
    }

    pub(crate) async fn handle_info(&self) -> VpnServiceInfoResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Info(tx))
            .unwrap();
        debug!("Sent info command to VPN");
        debug!("Waiting for response");
        let info = rx.await.unwrap();
        debug!("VPN info: {:?}", info);
        info
    }

    pub(crate) async fn handle_status(&self) -> VpnServiceStatusResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Status(tx))
            .unwrap();
        debug!("Sent status command to VPN");
        debug!("Waiting for response");
        let status = rx.await.unwrap();
        debug!("VPN status: {}", status);
        status
    }

    pub(crate) async fn handle_import_credential(
        &self,
        credential: Vec<u8>,
    ) -> Result<Option<OffsetDateTime>, ImportCredentialError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::ImportCredential(tx, credential))
            .unwrap();
        debug!("Sent import credential command to VPN");
        debug!("Waiting for response");
        let result = rx.await.unwrap();
        debug!("VPN import credential result: {:?}", result);
        result
    }

    pub(crate) async fn handle_list_entry_gateways(
        &self,
    ) -> Result<Vec<gateway::Gateway>, ListGatewayError> {
        let user_agent = bin_info_local_vergen!().into();
        let directory_config = nym_vpn_lib::gateway_directory::Config::new_from_env();
        let directory_client =
            nym_vpn_lib::gateway_directory::GatewayClient::new(directory_config, user_agent)
                .map_err(|error| ListGatewayError::CreateGatewayDirectoryClient { error })?;

        let gateways = directory_client
            .lookup_entry_gateways()
            .await
            .map_err(|error| ListGatewayError::GetEntryGateways { error })?;

        Ok(gateways.into_iter().map(gateway::Gateway::from).collect())
    }

    pub(crate) async fn handle_list_exit_gateways(
        &self,
    ) -> Result<Vec<gateway::Gateway>, ListGatewayError> {
        let user_agent = bin_info_local_vergen!().into();
        let directory_config = nym_vpn_lib::gateway_directory::Config::new_from_env();
        let directory_client =
            nym_vpn_lib::gateway_directory::GatewayClient::new(directory_config, user_agent)
                .map_err(|error| ListGatewayError::CreateGatewayDirectoryClient { error })?;

        let gateways = directory_client
            .lookup_exit_gateways()
            .await
            .map_err(|error| ListGatewayError::GetExitGateways { error })?;

        Ok(gateways.into_iter().map(gateway::Gateway::from).collect())
    }
}
