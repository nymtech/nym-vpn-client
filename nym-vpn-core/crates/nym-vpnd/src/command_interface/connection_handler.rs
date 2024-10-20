// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::{AccountState, ReadyToConnect};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use nym_vpn_api_client::{
    response::{NymVpnAccountSummaryResponse, NymVpnDevicesResponse},
    types::GatewayMinPerformance,
};
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint, GatewayClient, GatewayType};

use crate::{
    service::{
        AccountError, ConnectArgs, ConnectOptions, VpnServiceCommand, VpnServiceConnectError,
        VpnServiceDisconnectError, VpnServiceInfo, VpnServiceStatusResult,
    },
    types::gateway,
};

#[derive(Debug, thiserror::Error)]
pub enum ListGatewayError {
    #[error("failed to create gateway directory client: {source}")]
    CreateGatewayDirectoryClient {
        source: nym_vpn_lib::gateway_directory::Error,
    },

    #[error("failed to get gateways ({gw_type}): {source}")]
    GetGateways {
        gw_type: GatewayType,
        source: nym_vpn_lib::gateway_directory::Error,
    },

    #[error("failed to get countries ({gw_type}): {source}")]
    GetCountries {
        gw_type: GatewayType,
        source: nym_vpn_lib::gateway_directory::Error,
    },
}

pub(super) struct CommandInterfaceConnectionHandler {
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum VpnCommandSendError {
    #[error("failed to send command to VPN")]
    Send,

    #[error("failed to receive response from VPN")]
    Receive,
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
        user_agent: nym_vpn_lib::UserAgent,
    ) -> Result<Result<(), VpnServiceConnectError>, VpnCommandSendError> {
        tracing::info!("Starting VPN");
        let connect_args = ConnectArgs {
            entry,
            exit,
            options,
        };

        self.send_and_wait(VpnServiceCommand::Connect, (connect_args, user_agent))
            .await
    }

    pub(crate) async fn handle_disconnect(
        &self,
    ) -> Result<Result<(), VpnServiceDisconnectError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::Disconnect, ()).await
    }

    pub(crate) async fn handle_info(&self) -> Result<VpnServiceInfo, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::Info, ()).await
    }

    pub(crate) async fn handle_status(&self) -> VpnServiceStatusResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Status(tx, ()))
            .unwrap();
        tracing::debug!("Sent status command to VPN");
        tracing::debug!("Waiting for response");
        let status = rx.await.unwrap();
        tracing::debug!("VPN status: {}", status);
        status
    }

    pub(crate) async fn handle_list_gateways(
        &self,
        gw_type: GatewayType,
        user_agent: nym_vpn_lib::UserAgent,
        min_gateway_performance: GatewayMinPerformance,
    ) -> Result<Vec<gateway::Gateway>, ListGatewayError> {
        let gateways = directory_client(user_agent, min_gateway_performance)?
            .lookup_gateways(gw_type.clone())
            .await
            .map_err(|source| ListGatewayError::GetGateways { gw_type, source })?;

        Ok(gateways.into_iter().map(gateway::Gateway::from).collect())
    }

    pub(crate) async fn handle_list_countries(
        &self,
        gw_type: GatewayType,
        user_agent: nym_vpn_lib::UserAgent,
        min_gateway_performance: GatewayMinPerformance,
    ) -> Result<Vec<gateway::Country>, ListGatewayError> {
        let gateways = directory_client(user_agent, min_gateway_performance)?
            .lookup_countries(gw_type.clone())
            .await
            .map_err(|source| ListGatewayError::GetCountries { gw_type, source })?;

        Ok(gateways.into_iter().map(gateway::Country::from).collect())
    }

    pub(crate) async fn handle_store_account(
        &self,
        account: String,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::StoreAccount, account)
            .await
    }

    pub(crate) async fn handle_is_account_stored(
        &self,
    ) -> Result<Result<bool, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await
    }

    pub(crate) async fn handle_remove_account(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::RemoveAccount, ())
            .await
    }

    pub(crate) async fn handle_get_local_account_state(
        &self,
    ) -> Result<Result<AccountState, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetLocalAccountState, ())
            .await
    }

    pub(crate) async fn handle_get_account_summary(
        &self,
    ) -> Result<Result<NymVpnAccountSummaryResponse, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetAccountSummary, ())
            .await
    }

    pub(crate) async fn handle_get_devices(
        &self,
    ) -> Result<Result<NymVpnDevicesResponse, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetDevices, ()).await
    }

    pub(crate) async fn handle_register_device(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::RegisterDevice, ())
            .await
    }

    pub(crate) async fn handle_request_zk_nym(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::RequestZkNym, ())
            .await
    }

    pub(crate) async fn handle_get_device_zk_nyms(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetDeviceZkNyms, ())
            .await
    }

    pub(crate) async fn handle_is_ready_to_connect(
        &self,
    ) -> Result<Result<ReadyToConnect, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::IsReadyToConnect, ())
            .await
    }

    async fn send_and_wait<R, F, O>(&self, command: F, opts: O) -> Result<R, VpnCommandSendError>
    where
        F: FnOnce(oneshot::Sender<R>, O) -> VpnServiceCommand,
    {
        let (tx, rx) = oneshot::channel();

        self.vpn_command_tx.send(command(tx, opts)).map_err(|err| {
            tracing::error!("Failed to send command to VPN: {:?}", err);
            VpnCommandSendError::Send
        })?;

        rx.await.map_err(|err| {
            tracing::error!("Failed to receive response from VPN: {:?}", err);
            VpnCommandSendError::Receive
        })
    }
}

fn directory_client(
    user_agent: nym_vpn_lib::UserAgent,
    min_gateway_performance: GatewayMinPerformance,
) -> Result<GatewayClient, ListGatewayError> {
    let directory_config = nym_vpn_lib::gateway_directory::Config::new_from_env()
        .with_min_gateway_performance(min_gateway_performance);
    GatewayClient::new(directory_config, user_agent)
        .map_err(|source| ListGatewayError::CreateGatewayDirectoryClient { source })
}
