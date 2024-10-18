// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::AccountState;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevicesResponse, NymVpnSubscription,
        NymVpnSubscriptionsResponse,
    },
    types::GatewayMinPerformance,
};
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint, GatewayClient, GatewayType};

use crate::{
    service::{
        AccountError, ConnectArgs, ConnectOptions, VpnServiceCommand, VpnServiceConnectResult,
        VpnServiceDisconnectResult, VpnServiceInfoResult, VpnServiceStatusResult,
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
    ) -> VpnServiceConnectResult {
        tracing::info!("Starting VPN");
        let connect_args = ConnectArgs {
            entry,
            exit,
            options,
        };

        let (tx, rx) = oneshot::channel();
        self.send_and_wait(VpnServiceCommand::Connect(tx, connect_args, user_agent), rx)
            .await
            .err()
            .map(|e| VpnServiceConnectResult::Fail(e.to_string()))
            .unwrap_or(VpnServiceConnectResult::Success)
    }

    pub(crate) async fn handle_disconnect(&self) -> VpnServiceDisconnectResult {
        let (tx, rx) = oneshot::channel();
        self.send_and_wait(VpnServiceCommand::Disconnect(tx), rx)
            .await
            .err()
            .map(|e| VpnServiceDisconnectResult::Fail(e.to_string()))
            .unwrap_or(VpnServiceDisconnectResult::Success)
    }

    pub(crate) async fn handle_info(&self) -> VpnServiceInfoResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Info(tx))
            .unwrap();
        tracing::debug!("Sent info command to VPN");
        tracing::debug!("Waiting for response");
        let info = rx.await.unwrap();
        tracing::debug!("VPN info: {:?}", info);
        info
    }

    pub(crate) async fn handle_status(&self) -> VpnServiceStatusResult {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::Status(tx))
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

    pub(crate) async fn handle_store_account(&self, account: String) -> Result<(), AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::StoreAccount(tx, account))
            .unwrap();
        let result = rx.await.unwrap();
        tracing::debug!("VPN store account result: {:?}", result);
        result
    }

    pub(crate) async fn handle_is_account_stored(
        &self,
    ) -> Result<Result<bool, AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::IsAccountStored)
            .await
    }

    pub(crate) async fn handle_remove_account(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::RemoveAccount)
            .await
    }

    pub(crate) async fn handle_get_local_account_state(
        &self,
    ) -> Result<Result<AccountState, AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::GetLocalAccountState)
            .await
    }

    pub(crate) async fn handle_get_account_summary(
        &self,
    ) -> Result<Result<NymVpnAccountSummaryResponse, AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::GetAccountSummary)
            .await
    }

    pub(crate) async fn handle_get_devices(
        &self,
    ) -> Result<Result<NymVpnDevicesResponse, AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::GetDevices).await
    }

    pub(crate) async fn handle_register_device(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::RegisterDevice)
            .await
    }

    pub(crate) async fn handle_request_zk_nym(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::RequestZkNym).await
    }

    pub(crate) async fn handle_get_device_zk_nyms(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::GetDeviceZkNyms)
            .await
    }

    pub(crate) async fn handle_get_free_passes(
        &self,
    ) -> Result<Result<NymVpnSubscriptionsResponse, AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::GetFreePasses)
            .await
    }

    pub(crate) async fn handle_apply_freepass(
        &self,
        code: String,
    ) -> Result<NymVpnSubscription, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::ApplyFreepass(tx, code))
            .unwrap();
        let result = rx.await.unwrap();
        tracing::info!("VPN apply freepass result: {:#?}", result);
        result
    }

    pub(crate) async fn handle_is_ready_to_connect(
        &self,
    ) -> Result<Result<bool, AccountError>, VpnCommandSendError> {
        self.vpn_command_send(VpnServiceCommand::IsReadyToConnect)
            .await
    }

    // TODO: generalise this function to be used for all commands
    async fn vpn_command_send<T, E, F>(
        &self,
        command: F,
    ) -> Result<Result<T, E>, VpnCommandSendError>
    where
        F: FnOnce(oneshot::Sender<Result<T, E>>) -> VpnServiceCommand,
    {
        let (tx, rx) = oneshot::channel();

        self.vpn_command_tx.send(command(tx)).map_err(|err| {
            tracing::error!("Failed to send command to VPN: {:?}", err);
            VpnCommandSendError::Send
        })?;

        rx.await.map_err(|err| {
            tracing::error!("Failed to receive response from VPN: {:?}", err);
            VpnCommandSendError::Receive
        })
    }

    async fn send_and_wait<T>(
        &self,
        command: VpnServiceCommand,
        rx: oneshot::Receiver<T>,
    ) -> Result<T, VpnCommandSendError> {
        self.vpn_command_tx.send(command).map_err(|e| {
            tracing::error!("Failed to send command to VPN: {:?}", e);
            VpnCommandSendError::Send
        })?;
        rx.await.map_err(|e| {
            tracing::error!("Failed to receive response from VPN: {:?}", e);
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
