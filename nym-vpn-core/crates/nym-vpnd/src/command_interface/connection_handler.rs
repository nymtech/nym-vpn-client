// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::{
    response::{
        NymVpnAccountSummaryResponse, NymVpnDevice, NymVpnDevicesResponse, NymVpnSubscription,
        NymVpnSubscriptionsResponse, NymVpnZkNym, NymVpnZkNymResponse,
    },
    types::GatewayMinPerformance,
};
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint, GatewayClient, GatewayType};
use time::OffsetDateTime;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::{debug, info, warn};

use crate::{
    service::{
        AccountError, ConnectArgs, ConnectOptions, ImportCredentialError, VpnServiceCommand,
        VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceInfoResult,
        VpnServiceStatusResult,
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

    pub(crate) async fn handle_remove_account(&self) -> Result<(), AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::RemoveAccount(tx))
            .unwrap();
        let result = rx.await.unwrap();
        debug!("VPN remove account result: {:?}", result);
        result
    }

    pub(crate) async fn handle_get_account_summary(
        &self,
    ) -> Result<NymVpnAccountSummaryResponse, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::GetAccountSummary(tx))
            .unwrap();
        let result = rx.await.unwrap();
        info!("VPN get account summary result: {:#?}", result);
        result
    }

    pub(crate) async fn handle_get_devices(&self) -> Result<NymVpnDevicesResponse, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::GetDevices(tx))
            .unwrap();
        let result = rx.await.unwrap();
        info!("VPN get devices result: {:#?}", result);
        result
    }

    pub(crate) async fn handle_register_device(&self) -> Result<NymVpnDevice, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::RegisterDevice(tx))
            .unwrap();
        let result = rx.await.unwrap();
        info!("VPN register device result: {:#?}", result);
        result
    }

    pub(crate) async fn handle_request_zk_nym(&self) -> Result<NymVpnZkNym, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::RequestZkNym(tx))
            .unwrap();
        let result = rx.await.unwrap();
        info!("VPN request zk nym result: {:#?}", result);
        result
    }

    pub(crate) async fn handle_get_device_zk_nyms(
        &self,
    ) -> Result<NymVpnZkNymResponse, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::GetDeviceZkNyms(tx))
            .unwrap();
        let result = rx.await.unwrap();
        info!("VPN get device zk nyms result: {:#?}", result);
        result
    }

    pub(crate) async fn handle_get_free_passes(
        &self,
    ) -> Result<NymVpnSubscriptionsResponse, AccountError> {
        let (tx, rx) = oneshot::channel();
        self.vpn_command_tx
            .send(VpnServiceCommand::GetFreePasses(tx))
            .unwrap();
        let result = rx.await.unwrap();
        info!("VPN get free passes result: {:#?}", result);
        result
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
