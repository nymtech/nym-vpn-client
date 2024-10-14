// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnDevicesResponse, NymVpnSubscription,
        NymVpnSubscriptionsResponse,
    },
    types::GatewayMinPerformance,
};
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint, GatewayClient, GatewayType};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::{debug, info, warn};

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

    // TODO: generalise this function to be used for all commands
    async fn vpn_command_send<T, E, F>(
        &self,
        command: F,
    ) -> Result<Result<T, E>, VpnCommandSendError>
    where
        F: FnOnce(oneshot::Sender<Result<T, E>>) -> VpnServiceCommand,
    {
        let (tx, rx) = oneshot::channel();
        if let Err(err) = self.vpn_command_tx.send(command(tx)) {
            tracing::error!("Failed to send command to VPN: {:?}", err);
            return Err(VpnCommandSendError::Send);
        }
        rx.await.map_err(|err| {
            tracing::error!("Failed to receive response from VPN: {:?}", err);
            VpnCommandSendError::Receive
        })
    }

    pub(crate) async fn handle_connect(
        &self,
        entry: Option<EntryPoint>,
        exit: Option<ExitPoint>,
        options: ConnectOptions,
        user_agent: nym_vpn_lib::UserAgent,
    ) -> VpnServiceConnectResult {
        info!("Starting VPN");
        let (tx, rx) = oneshot::channel();
        let connect_args = ConnectArgs {
            entry,
            exit,
            options,
        };
        self.vpn_command_tx
            .send(VpnServiceCommand::Connect(tx, connect_args, user_agent))
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
        debug!("VPN store account result: {:?}", result);
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
    ) -> Result<Result<NymVpnDevice, AccountError>, VpnCommandSendError> {
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
        info!("VPN apply freepass result: {:#?}", result);
        result
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
