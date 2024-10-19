// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use futures::{stream::BoxStream, StreamExt};
use nym_vpn_account_controller::ReadyToConnect;
use tokio::sync::{broadcast, mpsc::UnboundedSender};

use nym_vpn_api_client::types::GatewayMinPerformance;
use nym_vpn_lib::tunnel_state_machine::MixnetEvent;
use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, AccountError, ConnectRequest, ConnectRequestError, ConnectResponse,
    ConnectionStateChange, ConnectionStatusUpdate, DisconnectRequest, DisconnectResponse, Empty,
    GetAccountSummaryRequest, GetAccountSummaryResponse, GetDeviceZkNymsRequest,
    GetDeviceZkNymsResponse, GetDevicesRequest, GetDevicesResponse, GetLocalAccountStateRequest,
    GetLocalAccountStateResponse, InfoRequest, InfoResponse, IsAccountStoredRequest,
    IsAccountStoredResponse, IsReadyToConnectRequest, IsReadyToConnectResponse,
    ListCountriesRequest, ListCountriesResponse, ListGatewaysRequest, ListGatewaysResponse,
    RegisterDeviceRequest, RegisterDeviceResponse, RemoveAccountRequest, RemoveAccountResponse,
    RequestZkNymRequest, RequestZkNymResponse, StatusRequest, StatusResponse, StoreAccountRequest,
    StoreAccountResponse,
};

use super::{
    connection_handler::CommandInterfaceConnectionHandler,
    error::CommandInterfaceError,
    helpers::{parse_entry_point, parse_exit_point, threshold_into_percent},
};
use crate::{
    command_interface::protobuf::gateway::into_user_agent,
    service::{ConnectOptions, VpnServiceCommand, VpnServiceConnectResult, VpnServiceStateChange},
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
            .await;

        let response = InfoResponse::from(info);
        tracing::info!("Returning info response: {:?}", response);
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
            .await;

        let response = match status {
            VpnServiceConnectResult::Success => ConnectResponse {
                success: true,
                error: None,
            },
            VpnServiceConnectResult::Fail(err) => ConnectResponse {
                success: false,
                error: Some(ConnectRequestError {
                    kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::NotReady
                        as i32,
                    message: err,
                }),
            },
        };

        tracing::info!("Returning connect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_disconnect(
        &self,
        _request: tonic::Request<DisconnectRequest>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_disconnect()
            .await;

        let response = DisconnectResponse {
            success: status.is_success(),
        };
        tracing::info!("Returning disconnect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_status(
        &self,
        _request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_status()
            .await;

        let response = StatusResponse::from(status);
        tracing::info!("Returning status response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    type ListenToConnectionStatusStream =
        BoxStream<'static, Result<ConnectionStatusUpdate, tonic::Status>>;

    async fn listen_to_connection_status(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::ListenToConnectionStatusStream>, tonic::Status> {
        tracing::info!("Got connection status stream request: {request:?}");
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
        tracing::info!("Got connection status stream request: {request:?}");
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
        tracing::info!("Got list gateways request: {:?}", request);

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

        tracing::info!(
            "Returning list gateways response: {} entries",
            response.gateways.len()
        );
        Ok(tonic::Response::new(response))
    }

    async fn list_countries(
        &self,
        request: tonic::Request<ListCountriesRequest>,
    ) -> Result<tonic::Response<ListCountriesResponse>, tonic::Status> {
        tracing::info!("Got list entry countries request: {request:?}");

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

        tracing::info!(
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
            .await;

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

        tracing::info!("Returning store account response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn is_account_stored(
        &self,
        _request: tonic::Request<IsAccountStoredRequest>,
    ) -> Result<tonic::Response<IsAccountStoredResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_is_account_stored()
            .await
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to check if account is stored: {err}"))
            })?;

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

        tracing::info!("Returning is account stored response");
        Ok(tonic::Response::new(response))
    }

    async fn remove_account(
        &self,
        _request: tonic::Request<RemoveAccountRequest>,
    ) -> Result<tonic::Response<RemoveAccountResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_remove_account()
            .await
            .map_err(|err| tonic::Status::internal(format!("Failed to remove account: {err}")))?;

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

        tracing::info!("Returning remove account response");
        Ok(tonic::Response::new(response))
    }

    async fn get_local_account_state(
        &self,
        _request: tonic::Request<GetLocalAccountStateRequest>,
    ) -> Result<tonic::Response<GetLocalAccountStateResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_local_account_state()
            .await
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get local account state: {err}"))
            })?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get local account state: {err}"))
            })?;

        let response = GetLocalAccountStateResponse {
            json: serde_json::to_string(&result).unwrap(),
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_account_summary(
        &self,
        _request: tonic::Request<GetAccountSummaryRequest>,
    ) -> Result<tonic::Response<GetAccountSummaryResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_summary()
            .await
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account summary: {err}"))
            })?;

        let response = match result {
            Ok(summary) => GetAccountSummaryResponse {
                json: serde_json::to_string(&summary).unwrap(),
                error: None,
            },
            Err(err) => GetAccountSummaryResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning get account summary response");
        Ok(tonic::Response::new(response))
    }

    async fn get_devices(
        &self,
        _request: tonic::Request<GetDevicesRequest>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_devices()
            .await
            .map_err(|err| tonic::Status::internal(format!("Failed to get devices: {err}")))?;

        let response = match result {
            Ok(devices) => GetDevicesResponse {
                json: serde_json::to_string(&devices).unwrap(),
                error: None,
            },
            Err(err) => GetDevicesResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning get devices response");
        Ok(tonic::Response::new(response))
    }

    async fn register_device(
        &self,
        _request: tonic::Request<RegisterDeviceRequest>,
    ) -> Result<tonic::Response<RegisterDeviceResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_register_device()
            .await
            .map_err(|err| tonic::Status::internal(format!("Failed to register device: {err}")))?;

        let response = match result {
            Ok(device) => RegisterDeviceResponse {
                json: serde_json::to_string(&device).unwrap(),
                error: None,
            },
            Err(err) => RegisterDeviceResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning register device response");
        Ok(tonic::Response::new(response))
    }

    async fn request_zk_nym(
        &self,
        _request: tonic::Request<RequestZkNymRequest>,
    ) -> Result<tonic::Response<RequestZkNymResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_request_zk_nym()
            .await
            .map_err(|err| tonic::Status::internal(format!("Failed to request zk nym: {err}")))?;

        let response = match result {
            Ok(response) => RequestZkNymResponse {
                json: serde_json::to_string(&response).unwrap(),
                error: None,
            },
            Err(err) => RequestZkNymResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning request zk nym response");
        Ok(tonic::Response::new(response))
    }

    async fn get_device_zk_nyms(
        &self,
        _request: tonic::Request<GetDeviceZkNymsRequest>,
    ) -> Result<tonic::Response<GetDeviceZkNymsResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_device_zk_nyms()
            .await
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get device zk nyms: {err}"))
            })?;

        let response = match result {
            Ok(response) => GetDeviceZkNymsResponse {
                json: serde_json::to_string(&response).unwrap(),
                error: None,
            },
            Err(err) => GetDeviceZkNymsResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning get device zk nyms response");
        Ok(tonic::Response::new(response))
    }

    async fn get_free_passes(
        &self,
        _request: tonic::Request<nym_vpn_proto::GetFreePassesRequest>,
    ) -> Result<tonic::Response<nym_vpn_proto::GetFreePassesResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_free_passes()
            .await
            .map_err(|err| tonic::Status::internal(format!("Failed to get free passes: {err}")))?;

        let response = match result {
            Ok(response) => nym_vpn_proto::GetFreePassesResponse {
                json: serde_json::to_string(&response).unwrap(),
                error: None,
            },
            Err(err) => nym_vpn_proto::GetFreePassesResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning get free passes response");
        Ok(tonic::Response::new(response))
    }

    async fn apply_freepass(
        &self,
        request: tonic::Request<nym_vpn_proto::ApplyFreepassRequest>,
    ) -> Result<tonic::Response<nym_vpn_proto::ApplyFreepassResponse>, tonic::Status> {
        let code = request.into_inner().code;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_apply_freepass(code)
            .await;

        let response = match result {
            Ok(response) => nym_vpn_proto::ApplyFreepassResponse {
                json: serde_json::to_string(&response).unwrap(),
                error: None,
            },
            Err(err) => nym_vpn_proto::ApplyFreepassResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::info!("Returning apply freepass response");
        Ok(tonic::Response::new(response))
    }

    async fn is_ready_to_connect(
        &self,
        _request: tonic::Request<IsReadyToConnectRequest>,
    ) -> Result<tonic::Response<IsReadyToConnectResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_is_ready_to_connect()
            .await
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to check if ready to connect: {err}"))
            })?
            .map(|ready| ready == ReadyToConnect::Ready);

        let response = IsReadyToConnectResponse {
            is_ready_to_connect: result.unwrap_or(false),
        };

        tracing::info!("Returning is ready to connect response");
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
            enable_poisson_rate: request.enable_poisson_rate,
            disable_background_cover_traffic,
            enable_credentials_mode: request.enable_credentials_mode,
            min_mixnode_performance,
            min_gateway_mixnet_performance,
            min_gateway_vpn_performance,
        })
    }
}
