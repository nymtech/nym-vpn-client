// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::{AccountStateSummary, AvailableTicketbooks, ReadyToConnect};
use nym_vpn_network_config::{FeatureFlags, ParsedAccountLinks, SystemMessages};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use nym_vpn_api_client::{
    response::{NymVpnAccountSummaryResponse, NymVpnDevicesResponse},
    types::GatewayMinPerformance,
};
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint, GatewayClient, GatewayType};

use crate::{
    service::{
        AccountError, ConnectArgs, ConnectOptions, SetNetworkError, VpnServiceCommand,
        VpnServiceConnectError, VpnServiceDisconnectError, VpnServiceInfo, VpnServiceStatus,
    },
    types::gateway,
};

use super::protobuf::error::VpnCommandSendError;

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

    pub(crate) async fn handle_info(&self) -> Result<VpnServiceInfo, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::Info, ()).await
    }

    pub(crate) async fn handle_set_network(
        &self,
        network: String,
    ) -> Result<Result<(), SetNetworkError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::SetNetwork, network)
            .await
    }

    pub(crate) async fn handle_get_system_messages(
        &self,
    ) -> Result<SystemMessages, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetSystemMessages, ())
            .await
    }

    pub(crate) async fn handle_get_feature_flags(
        &self,
    ) -> Result<Option<FeatureFlags>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetFeatureFlags, ())
            .await
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

    pub(crate) async fn handle_status(&self) -> Result<VpnServiceStatus, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::Status, ()).await
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

    pub(crate) async fn handle_get_account_identity(
        &self,
    ) -> Result<Result<String, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetAccountIdentity, ())
            .await
    }

    pub(crate) async fn handle_get_account_links(
        &self,
        locale: String,
    ) -> Result<Result<ParsedAccountLinks, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetAccountLinks, locale)
            .await
    }

    pub(crate) async fn handle_get_account_state(
        &self,
    ) -> Result<Result<AccountStateSummary, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetAccountState, ())
            .await
    }

    pub(crate) async fn handle_refresh_account_state(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::RefreshAccountState, ())
            .await
    }

    pub(crate) async fn handle_is_ready_to_connect(
        &self,
    ) -> Result<Result<ReadyToConnect, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::IsReadyToConnect, ())
            .await
    }

    pub(crate) async fn handle_reset_device_identity(
        &self,
        seed: Option<[u8; 32]>,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::ResetDeviceIdentity, seed)
            .await
    }

    pub(crate) async fn handle_get_device_identity(
        &self,
    ) -> Result<Result<String, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetDeviceIdentity, ())
            .await
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

    pub(crate) async fn handle_get_zk_nyms_available_for_download(
        &self,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetZkNymsAvailableForDownload, ())
            .await
    }

    pub(crate) async fn handle_get_zk_nym_by_id(
        &self,
        id: String,
    ) -> Result<Result<(), AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetZkNymById, id)
            .await
    }

    pub(crate) async fn handle_get_available_tickets(
        &self,
    ) -> Result<Result<AvailableTicketbooks, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::GetAvailableTickets, ())
            .await
    }

    pub(crate) async fn handle_fetch_raw_account_summary(
        &self,
    ) -> Result<Result<NymVpnAccountSummaryResponse, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::FetchRawAccountSummary, ())
            .await
    }

    pub(crate) async fn handle_fetch_raw_devices(
        &self,
    ) -> Result<Result<NymVpnDevicesResponse, AccountError>, VpnCommandSendError> {
        self.send_and_wait(VpnServiceCommand::FetchRawDevices, ())
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
