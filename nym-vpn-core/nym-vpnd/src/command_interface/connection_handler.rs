// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::VpnApiClientError;
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
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
    #[error("network endpoints not configured")]
    NetworkEndpointsNotConfigured,

    #[error("network environment missing api url")]
    NetworkEnvironmentMissingApiUrl,

    #[error("failed to get gateways from nym api")]
    FailedToGetGatewaysFromNymApi {
        error: nym_validator_client::ValidatorClientError,
    },

    #[error("failed to get entry gateways: {error}")]
    FailedToGetEntryGatewaysFromNymVpnApi { error: VpnApiClientError },

    #[error("failed to get exit gateways: {error}")]
    FailedToGetExitGatewaysFromNymVpnApi { error: VpnApiClientError },
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
        let user_agent = nym_vpn_lib::nym_bin_common::bin_info_local_vergen!().into();
        let nym_network_details =
            nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env();

        if nym_network_details.network_name == "mainnet" {
            nym_vpn_api_client::get_entry_gateways(user_agent)
                .await
                .map(|gateways| gateways.into_iter().map(gateway::Gateway::from).collect())
                .map_err(|error| ListGatewayError::FailedToGetEntryGatewaysFromNymVpnApi { error })
        } else {
            let nym_api_client =
                nym_validator_client::NymApiClient::new_with_user_agent(api_url()?, user_agent);

            nym_api_client
                .get_cached_described_gateways()
                .await
                .map_err(|error| ListGatewayError::FailedToGetGatewaysFromNymApi { error })
                .map(|g| g.into_iter().map(gateway::Gateway::from).collect())
        }
    }

    pub(crate) async fn handle_list_exit_gateways(
        &self,
    ) -> Result<Vec<gateway::Gateway>, ListGatewayError> {
        let user_agent = nym_vpn_lib::nym_bin_common::bin_info_local_vergen!().into();
        let nym_network_details =
            nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env();

        if nym_network_details.network_name == "mainnet" {
            nym_vpn_api_client::get_exit_gateways(user_agent)
                .await
                .map(|gateways| gateways.into_iter().map(gateway::Gateway::from).collect())
                .map_err(|error| ListGatewayError::FailedToGetExitGatewaysFromNymVpnApi { error })
        } else {
            let nym_api_client =
                nym_validator_client::NymApiClient::new_with_user_agent(api_url()?, user_agent);

            let gateways = nym_api_client
                .get_cached_described_gateways()
                .await
                .map_err(|error| ListGatewayError::FailedToGetGatewaysFromNymApi { error })?;

            // We check the existence of ip_packet_router to determine if the gateway is an exit
            // gateway. In the future we check the role field.
            let is_exit_gateway = |g: &nym_validator_client::models::DescribedGateway| {
                g.self_described
                    .as_ref()
                    .and_then(|d| d.ip_packet_router.as_ref())
                    .is_some()
            };

            Ok(gateways
                .into_iter()
                .filter(is_exit_gateway)
                .map(gateway::Gateway::from)
                .collect())
        }
    }
}

fn api_url() -> Result<url::Url, ListGatewayError> {
    nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env()
        .endpoints
        .first()
        .ok_or(ListGatewayError::NetworkEndpointsNotConfigured)?
        .api_url()
        .ok_or(ListGatewayError::NetworkEnvironmentMissingApiUrl)
}
