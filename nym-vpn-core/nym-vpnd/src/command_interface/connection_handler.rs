// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::VpnApiClientError;
use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
use time::OffsetDateTime;
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tracing::{debug, info, warn};

use crate::service::{
    ConnectArgs, ConnectOptions, ImportCredentialError, VpnServiceCommand, VpnServiceConnectResult,
    VpnServiceDisconnectResult, VpnServiceInfoResult, VpnServiceStatusResult,
};

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
    ) -> Result<Vec<nym_vpn_api_client::Gateway>, VpnApiClientError> {
        let user_agent = nym_vpn_lib::nym_bin_common::bin_info_local_vergen!().into();

        let nym_network_details =
            nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env();
        let network_name = nym_network_details.network_name;

        dbg!(&network_name);

        if network_name == "mainnet" {
            nym_vpn_api_client::get_entry_gateways(user_agent).await
        } else {
            // TODO: do these at startup so we can validate
            let api_url = nym_network_details
                .endpoints
                .first()
                .expect("network environment endpoints not correctly configured")
                .api_url
                .clone()
                .expect("network environment missing api_url")
                .parse()
                .expect("network environment api_url not parseable");

            let nym_api_client =
                nym_validator_client::NymApiClient::new_with_user_agent(api_url, user_agent);

            // let config = nym_vpn_lib::gateway_directory::Config::new_from_env();
            // let gw_client =
            // nym_vpn_lib::gateway_directory::GatewayClient::new(config, user_agent).unwrap();
            // let gateways = gw_client.lookup_described_gateways().await.unwrap();
            let gateways = nym_api_client
                .get_cached_described_gateways()
                .await
                .unwrap();
            let g: Vec<_> = gateways
                .into_iter()
                .map(|gw| {
                    let id = gw.bond.identity().clone();
                    let location = nym_vpn_api_client::Location {
                        two_letter_iso_country_code: "".to_string(),
                        latitude: 0f64,
                        longitude: 0f64,
                    };
                    nym_vpn_api_client::Gateway {
                        identity_key: id,
                        location,
                        last_probe: None,
                    }
                })
                .collect();
            Ok(g)
        }
    }
}
