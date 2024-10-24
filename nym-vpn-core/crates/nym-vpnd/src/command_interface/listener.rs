// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use futures::{stream::BoxStream, StreamExt};
use tokio::sync::{broadcast, mpsc::UnboundedSender};

use nym_vpn_api_client::types::GatewayMinPerformance;
use nym_vpn_lib::tunnel_state_machine::MixnetEvent;
use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, AccountError, ConnectRequest, ConnectResponse, ConnectionStateChange,
    ConnectionStatusUpdate, DisconnectRequest, DisconnectResponse, Empty,
    FetchRawAccountSummaryRequest, FetchRawAccountSummaryResponse, FetchRawDevicesRequest,
    FetchRawDevicesResponse, GetAccountStateRequest, GetAccountStateResponse,
    GetDeviceZkNymsRequest, GetDeviceZkNymsResponse, InfoRequest, InfoResponse,
    IsAccountStoredRequest, IsAccountStoredResponse, IsReadyToConnectRequest,
    IsReadyToConnectResponse, ListCountriesRequest, ListCountriesResponse, ListGatewaysRequest,
    ListGatewaysResponse, RegisterDeviceRequest, RegisterDeviceResponse, RemoveAccountRequest,
    RemoveAccountResponse, RequestZkNymRequest, RequestZkNymResponse, ResetDeviceIdentityRequest,
    ResetDeviceIdentityResponse, SetNetworkRequest, SetNetworkResponse, StatusRequest,
    StatusResponse, StoreAccountRequest, StoreAccountResponse,
};

use super::{
    connection_handler::CommandInterfaceConnectionHandler,
    error::CommandInterfaceError,
    helpers::{parse_entry_point, parse_exit_point, threshold_into_percent},
};
use crate::{
    command_interface::protobuf::{
        connection_state::into_is_ready_to_connect_response_type, gateway::into_user_agent,
    },
    service::{ConnectOptions, VpnServiceCommand, VpnServiceStateChange},
};

enum ListenerType {
    Path(PathBuf),
    Uri(#[allow(unused)] SocketAddr),
}

pub(super) struct CommandInterface {
    // Listen to state changes from the VPN service
    vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,

    // Send commands to the VPN service
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,

    // Broadcast connection status updates to our API endpoint listeners
    status_rx: broadcast::Receiver<MixnetEvent>,

    listener: ListenerType,
}

impl CommandInterface {
    pub(super) fn new_with_path(
        vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        status_rx: broadcast::Receiver<MixnetEvent>,
        socket_path: &Path,
    ) -> Self {
        Self {
            vpn_state_changes_rx,
            vpn_command_tx,
            status_rx,
            listener: ListenerType::Path(socket_path.to_path_buf()),
        }
    }

    pub(super) fn new_with_uri(
        vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        status_rx: broadcast::Receiver<MixnetEvent>,
        uri: SocketAddr,
    ) -> Self {
        Self {
            vpn_state_changes_rx,
            vpn_command_tx,
            status_rx,
            listener: ListenerType::Uri(uri),
        }
    }

    pub(super) fn remove_previous_socket_file(&self) {
        if let ListenerType::Path(ref socket_path) = self.listener {
            match fs::remove_file(socket_path) {
                Ok(_) => tracing::info!(
                    "Removed previous command interface socket: {:?}",
                    socket_path
                ),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    tracing::error!(
                        "Failed to remove previous command interface socket: {:?}",
                        err
                    );
                }
            }
        }
    }
}

impl Drop for CommandInterface {
    fn drop(&mut self) {
        self.remove_previous_socket_file();
    }
}

#[tonic::async_trait]
impl NymVpnd for CommandInterface {
    async fn info(
        &self,
        _request: tonic::Request<InfoRequest>,
    ) -> Result<tonic::Response<InfoResponse>, tonic::Status> {
        let info = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_info()
            .await?;

        let response = InfoResponse::from(info);
        tracing::debug!("Returning info response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn set_network(
        &self,
        request: tonic::Request<SetNetworkRequest>,
    ) -> Result<tonic::Response<SetNetworkResponse>, tonic::Status> {
        let network = request.into_inner().network;

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_set_network(network)
            .await?;

        let response = nym_vpn_proto::SetNetworkResponse {
            error: status
                .err()
                .map(nym_vpn_proto::SetNetworkRequestError::from),
        };
        tracing::debug!("Returning set network response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        tracing::info!("Got connect request: {:?}", request);

        let connect_request = request.into_inner();

        let entry = connect_request
            .entry
            .clone()
            .and_then(|e| e.entry_node_enum)
            .map(parse_entry_point)
            .transpose()?;

        let exit = connect_request
            .exit
            .clone()
            .and_then(|e| e.exit_node_enum)
            .map(parse_exit_point)
            .transpose()?;

        let user_agent = connect_request
            .user_agent
            .clone()
            .map(into_user_agent)
            .unwrap_or_else(crate::util::construct_user_agent);

        let options = ConnectOptions::try_from(connect_request).map_err(|err| {
            tracing::error!("Failed to parse connect options: {:?}", err);
            tonic::Status::invalid_argument("Invalid connect options")
        })?;

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect(entry, exit, options, user_agent)
            .await?;

        let response = match status {
            Ok(()) => ConnectResponse {
                success: true,
                error: None,
            },
            Err(err) => ConnectResponse {
                success: false,
                error: Some(nym_vpn_proto::ConnectRequestError::from(err)),
            },
        };

        tracing::debug!("Returning connect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_disconnect(
        &self,
        _request: tonic::Request<DisconnectRequest>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_disconnect()
            .await?;

        let response = DisconnectResponse {
            success: status.is_ok(),
        };
        tracing::debug!("Returning disconnect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_status(
        &self,
        _request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_status()
            .await?;

        let response = StatusResponse::from(status);
        tracing::debug!("Returning status response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    type ListenToConnectionStatusStream =
        BoxStream<'static, Result<ConnectionStatusUpdate, tonic::Status>>;

    async fn listen_to_connection_status(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::ListenToConnectionStatusStream>, tonic::Status> {
        tracing::debug!("Got connection status stream request: {request:?}");
        let rx = self.status_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|status| {
            status
                .map(crate::command_interface::protobuf::status_update::status_update_from_event)
                .map_err(|err| {
                    tracing::error!("Failed to receive connection status update: {:?}", err);
                    tonic::Status::internal("Failed to receive connection status update")
                })
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToConnectionStatusStream
        ))
    }

    type ListenToConnectionStateChangesStream =
        BoxStream<'static, Result<ConnectionStateChange, tonic::Status>>;

    async fn listen_to_connection_state_changes(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::ListenToConnectionStateChangesStream>, tonic::Status> {
        tracing::debug!("Got connection status stream request: {request:?}");
        let rx = self.vpn_state_changes_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|status| {
            status.map(ConnectionStateChange::from).map_err(|err| {
                tracing::error!("Failed to receive connection state change: {:?}", err);
                tonic::Status::internal("Failed to receive connection state change")
            })
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToConnectionStateChangesStream
        ))
    }

    async fn list_gateways(
        &self,
        request: tonic::Request<ListGatewaysRequest>,
    ) -> Result<tonic::Response<ListGatewaysResponse>, tonic::Status> {
        tracing::debug!("Got list gateways request: {:?}", request);

        let request = request.into_inner();

        let gw_type = nym_vpn_proto::GatewayType::try_from(request.kind)
            .ok()
            .and_then(crate::command_interface::protobuf::gateway::into_gateway_type)
            .ok_or_else(|| {
                let msg = format!("Failed to parse gateway type: {}", request.kind);
                tracing::error!(msg);
                tonic::Status::invalid_argument(msg)
            })?;

        let user_agent = request
            .user_agent
            .map(into_user_agent)
            .unwrap_or_else(crate::util::construct_user_agent);

        let min_mixnet_performance = request.min_mixnet_performance.map(threshold_into_percent);
        let min_vpn_performance = request.min_vpn_performance.map(threshold_into_percent);

        let min_gateway_performance = GatewayMinPerformance {
            mixnet_min_performance: min_mixnet_performance,
            vpn_min_performance: min_vpn_performance,
        };

        let gateways = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_list_gateways(gw_type, user_agent, min_gateway_performance)
            .await
            .map_err(|err| {
                let msg = format!("Failed to list gateways: {:?}", err);
                tracing::error!(msg);
                tonic::Status::internal(msg)
            })?;

        let response = ListGatewaysResponse {
            gateways: gateways
                .into_iter()
                .map(nym_vpn_proto::GatewayResponse::from)
                .collect(),
        };

        tracing::debug!(
            "Returning list gateways response: {} entries",
            response.gateways.len()
        );
        Ok(tonic::Response::new(response))
    }

    async fn list_countries(
        &self,
        request: tonic::Request<ListCountriesRequest>,
    ) -> Result<tonic::Response<ListCountriesResponse>, tonic::Status> {
        tracing::debug!("Got list entry countries request: {request:?}");

        let request = request.into_inner();

        let gw_type = nym_vpn_proto::GatewayType::try_from(request.kind)
            .ok()
            .and_then(crate::command_interface::protobuf::gateway::into_gateway_type)
            .ok_or_else(|| {
                let msg = format!("Failed to parse list countries kind: {}", request.kind);
                tracing::error!(msg);
                tonic::Status::invalid_argument(msg)
            })?;

        let user_agent = request
            .user_agent
            .map(into_user_agent)
            .unwrap_or_else(crate::util::construct_user_agent);

        let min_mixnet_performance = request.min_mixnet_performance.map(threshold_into_percent);
        let min_vpn_performance = request.min_vpn_performance.map(threshold_into_percent);

        let min_gateway_performance = GatewayMinPerformance {
            mixnet_min_performance: min_mixnet_performance,
            vpn_min_performance: min_vpn_performance,
        };

        let countries = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_list_countries(gw_type, user_agent, min_gateway_performance)
            .await
            .map_err(|err| {
                let msg = format!("Failed to list entry countries: {:?}", err);
                tracing::error!(msg);
                tonic::Status::internal(msg)
            })?;

        let response = nym_vpn_proto::ListCountriesResponse {
            countries: countries
                .into_iter()
                .map(nym_vpn_proto::Location::from)
                .collect(),
        };

        tracing::debug!(
            "Returning list countries response: {} countries",
            response.countries.len()
        );
        Ok(tonic::Response::new(response))
    }

    async fn store_account(
        &self,
        request: tonic::Request<StoreAccountRequest>,
    ) -> Result<tonic::Response<StoreAccountResponse>, tonic::Status> {
        let account = request.into_inner().mnemonic;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_store_account(account)
            .await?;

        let response = match result {
            Ok(()) => StoreAccountResponse {
                success: true,
                error: None,
            },
            Err(err) => StoreAccountResponse {
                success: false,
                error: Some(nym_vpn_proto::AccountError::from(err)),
            },
        };

        tracing::debug!("Returning store account response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn is_account_stored(
        &self,
        _request: tonic::Request<IsAccountStoredRequest>,
    ) -> Result<tonic::Response<IsAccountStoredResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_is_account_stored()
            .await?;

        let response = match result {
            Ok(is_stored) => IsAccountStoredResponse {
                resp: Some(nym_vpn_proto::is_account_stored_response::Resp::IsStored(
                    is_stored,
                )),
            },
            Err(err) => IsAccountStoredResponse {
                resp: Some(nym_vpn_proto::is_account_stored_response::Resp::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
            },
        };

        tracing::debug!("Returning is account stored response");
        Ok(tonic::Response::new(response))
    }

    async fn remove_account(
        &self,
        _request: tonic::Request<RemoveAccountRequest>,
    ) -> Result<tonic::Response<RemoveAccountResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_remove_account()
            .await?;

        let response = match result {
            Ok(()) => RemoveAccountResponse {
                success: true,
                error: None,
            },
            Err(err) => RemoveAccountResponse {
                success: false,
                error: Some(nym_vpn_proto::AccountError::from(err)),
            },
        };

        tracing::debug!("Returning remove account response");
        Ok(tonic::Response::new(response))
    }

    async fn get_account_state(
        &self,
        _request: tonic::Request<GetAccountStateRequest>,
    ) -> Result<tonic::Response<GetAccountStateResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_state()
            .await?;

        let response = match result {
            Ok(state) => GetAccountStateResponse {
                result: Some(
                    nym_vpn_proto::get_account_state_response::Result::AccountSummary(
                        super::protobuf::account::into_account_summary(state),
                    ),
                ),
            },
            Err(err) => {
                // TODO: consider proper error handling for AccountError in this context
                return Err(tonic::Status::internal(format!(
                    "Failed to get account state: {err}"
                )));
            }
        };

        Ok(tonic::Response::new(response))
    }

    async fn is_ready_to_connect(
        &self,
        _request: tonic::Request<IsReadyToConnectRequest>,
    ) -> Result<tonic::Response<IsReadyToConnectResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_is_ready_to_connect()
            .await?;

        let response = match result {
            Ok(ready) => IsReadyToConnectResponse {
                kind: into_is_ready_to_connect_response_type(ready) as i32,
            },
            Err(err) => {
                // TODO: consider proper error handling for AccountError in this context
                tracing::error!("Failed to check if ready to connect: {:?}", err);
                return Err(tonic::Status::internal(
                    "Failed to check if ready to connect",
                ));
            }
        };

        tracing::debug!("Returning is ready to connect response");
        Ok(tonic::Response::new(response))
    }
    async fn reset_device_identity(
        &self,
        request: tonic::Request<ResetDeviceIdentityRequest>,
    ) -> Result<tonic::Response<ResetDeviceIdentityResponse>, tonic::Status> {
        let seed: [u8; 32] = request
            .into_inner()
            .seed
            .as_slice()
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("Seed must be 32 bytes long"))?;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_reset_device_identity(Some(seed))
            .await?;

        let response = ResetDeviceIdentityResponse {
            success: result.is_ok(),
            error: result.err().map(AccountError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn register_device(
        &self,
        _request: tonic::Request<RegisterDeviceRequest>,
    ) -> Result<tonic::Response<RegisterDeviceResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_register_device()
            .await?;

        let response = match result {
            Ok(device) => RegisterDeviceResponse {
                json: serde_json::to_string(&device)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => RegisterDeviceResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::debug!("Returning register device response");
        Ok(tonic::Response::new(response))
    }

    async fn request_zk_nym(
        &self,
        _request: tonic::Request<RequestZkNymRequest>,
    ) -> Result<tonic::Response<RequestZkNymResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_request_zk_nym()
            .await?;

        let response = match result {
            Ok(response) => RequestZkNymResponse {
                json: serde_json::to_string(&response)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => RequestZkNymResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::debug!("Returning request zk nym response");
        Ok(tonic::Response::new(response))
    }

    async fn get_device_zk_nyms(
        &self,
        _request: tonic::Request<GetDeviceZkNymsRequest>,
    ) -> Result<tonic::Response<GetDeviceZkNymsResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_device_zk_nyms()
            .await?;

        let response = match result {
            Ok(response) => GetDeviceZkNymsResponse {
                json: serde_json::to_string(&response)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => GetDeviceZkNymsResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::debug!("Returning get device zk nyms response");
        Ok(tonic::Response::new(response))
    }

    async fn fetch_raw_account_summary(
        &self,
        _request: tonic::Request<FetchRawAccountSummaryRequest>,
    ) -> Result<tonic::Response<FetchRawAccountSummaryResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_fetch_raw_account_summary()
            .await?;

        let response = match result {
            Ok(summary) => FetchRawAccountSummaryResponse {
                json: serde_json::to_string(&summary)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => FetchRawAccountSummaryResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn fetch_raw_devices(
        &self,
        _request: tonic::Request<FetchRawDevicesRequest>,
    ) -> Result<tonic::Response<FetchRawDevicesResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_fetch_raw_devices()
            .await?;

        let response = match result {
            Ok(devices) => FetchRawDevicesResponse {
                json: serde_json::to_string(&devices)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => FetchRawDevicesResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        Ok(tonic::Response::new(response))
    }
}

impl TryFrom<ConnectRequest> for ConnectOptions {
    type Error = CommandInterfaceError;

    fn try_from(request: ConnectRequest) -> Result<Self, Self::Error> {
        // Parse the inner DNS IP address if it exists, but make sure to keep the outer Option.
        let dns = request
            .dns
            .map(|dns| {
                dns.ip
                    .parse()
                    .map_err(|err| CommandInterfaceError::FailedToParseDnsIp {
                        ip: dns.ip.clone(),
                        source: err,
                    })
            })
            .transpose()?;

        let min_mixnode_performance = request.min_mixnode_performance.map(threshold_into_percent);
        let min_gateway_mixnet_performance = request
            .min_gateway_mixnet_performance
            .map(threshold_into_percent);
        let min_gateway_vpn_performance = request
            .min_gateway_vpn_performance
            .map(threshold_into_percent);

        let disable_background_cover_traffic = if request.enable_two_hop {
            // If two-hop is enabled, we always disable background cover traffic
            true
        } else {
            request.disable_background_cover_traffic
        };

        Ok(ConnectOptions {
            dns,
            disable_routing: request.disable_routing,
            enable_two_hop: request.enable_two_hop,
            disable_poisson_rate: request.disable_poisson_rate,
            disable_background_cover_traffic,
            enable_credentials_mode: request.enable_credentials_mode,
            min_mixnode_performance,
            min_gateway_mixnet_performance,
            min_gateway_vpn_performance,
        })
    }
}
