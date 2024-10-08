// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::SystemTime,
};

use futures::{stream::BoxStream, StreamExt};
use nym_vpn_api_client::types::GatewayMinPerformance;
use nym_vpn_proto::{
    nym_vpnd_server::NymVpnd, AccountError, ConnectRequest, ConnectResponse, ConnectionStateChange,
    ConnectionStatusUpdate, DisconnectRequest, DisconnectResponse, Empty, GetAccountSummaryRequest,
    GetAccountSummaryResponse, GetDeviceZkNymsRequest, GetDeviceZkNymsResponse, GetDevicesRequest,
    GetDevicesResponse, ImportUserCredentialRequest, ImportUserCredentialResponse, InfoRequest,
    InfoResponse, ListCountriesRequest, ListCountriesResponse, ListGatewaysRequest,
    ListGatewaysResponse, RegisterDeviceRequest, RegisterDeviceResponse, RemoveAccountRequest,
    RemoveAccountResponse, RequestZkNymRequest, RequestZkNymResponse, StatusRequest,
    StatusResponse, StoreAccountRequest, StoreAccountResponse,
};
use prost_types::Timestamp;
use tokio::sync::{broadcast, mpsc::UnboundedSender};
use tracing::{error, info};

use super::{
    connection_handler::CommandInterfaceConnectionHandler,
    error::CommandInterfaceError,
    helpers::{parse_entry_point, parse_exit_point, threshold_into_percent},
    status_broadcaster::ConnectionStatusBroadcaster,
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
    status_tx: tokio::sync::broadcast::Sender<ConnectionStatusUpdate>,

    listener: ListenerType,
}

impl CommandInterface {
    pub(super) fn new_with_path(
        vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        socket_path: &Path,
    ) -> Self {
        Self {
            vpn_state_changes_rx,
            vpn_command_tx,
            status_tx: tokio::sync::broadcast::channel(10).0,
            listener: ListenerType::Path(socket_path.to_path_buf()),
        }
    }

    pub(super) fn new_with_uri(
        vpn_state_changes_rx: broadcast::Receiver<VpnServiceStateChange>,
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        uri: SocketAddr,
    ) -> Self {
        Self {
            vpn_state_changes_rx,
            vpn_command_tx,
            status_tx: tokio::sync::broadcast::channel(10).0,
            listener: ListenerType::Uri(uri),
        }
    }

    pub(super) fn remove_previous_socket_file(&self) {
        if let ListenerType::Path(ref socket_path) = self.listener {
            match fs::remove_file(socket_path) {
                Ok(_) => info!(
                    "Removed previous command interface socket: {:?}",
                    socket_path
                ),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    error!(
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
        request: tonic::Request<InfoRequest>,
    ) -> Result<tonic::Response<InfoResponse>, tonic::Status> {
        info!("Got info request: {:?}", request);

        let info = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_info()
            .await;

        let response = InfoResponse::from(info);
        info!("Returning info response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        info!("Got connect request: {:?}", request);

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
            error!("Failed to parse connect options: {:?}", err);
            tonic::Status::invalid_argument("Invalid connect options")
        })?;

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect(entry, exit, options, user_agent)
            .await;

        let success = status.is_success();

        // After connecting we start a task that listens for status updates and broadcasts them for
        // listeners to the connection status stream.
        if let VpnServiceConnectResult::Success(connect_handle) = status {
            ConnectionStatusBroadcaster::new(
                self.status_tx.clone(),
                connect_handle.listener_vpn_status_rx,
            )
            .start();
        }

        let response = ConnectResponse { success };
        info!("Returning connect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_disconnect(
        &self,
        request: tonic::Request<DisconnectRequest>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        info!("Got disconnect request: {:?}", request);

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_disconnect()
            .await;

        let response = DisconnectResponse {
            success: status.is_success(),
        };
        info!("Returning disconnect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_status(
        &self,
        request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        info!("Got status request: {:?}", request);

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_status()
            .await;

        let response = StatusResponse::from(status);
        info!("Returning status response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn import_user_credential(
        &self,
        request: tonic::Request<ImportUserCredentialRequest>,
    ) -> Result<tonic::Response<ImportUserCredentialResponse>, tonic::Status> {
        info!("Got import credential request");

        let credential = request.into_inner().credential;

        let response = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_import_credential(credential)
            .await;

        let response = match response {
            Ok(time) => ImportUserCredentialResponse {
                success: true,
                error: None,
                expiry: time.map(|t| Timestamp::from(SystemTime::from(t))),
            },
            Err(err) => ImportUserCredentialResponse {
                success: false,
                error: Some(err.into()),
                expiry: None,
            },
        };
        info!("Returning import credential response: {:?}", response);

        Ok(tonic::Response::new(response))
    }

    type ListenToConnectionStatusStream =
        BoxStream<'static, Result<ConnectionStatusUpdate, tonic::Status>>;

    async fn listen_to_connection_status(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Self::ListenToConnectionStatusStream>, tonic::Status> {
        info!("Got connection status stream request: {request:?}");
        let rx = self.status_tx.subscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|status| {
            status.map_err(|err| {
                error!("Failed to receive connection status update: {:?}", err);
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
        info!("Got connection status stream request: {request:?}");
        let rx = self.vpn_state_changes_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|status| {
            status.map(ConnectionStateChange::from).map_err(|err| {
                error!("Failed to receive connection state change: {:?}", err);
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
        info!("Got list gateways request: {:?}", request);

        let request = request.into_inner();

        let gw_type = nym_vpn_proto::GatewayType::try_from(request.kind)
            .ok()
            .and_then(crate::command_interface::protobuf::gateway::into_gateway_type)
            .ok_or_else(|| {
                let msg = format!("Failed to parse gateway type: {}", request.kind);
                error!(msg);
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
                error!(msg);
                tonic::Status::internal(msg)
            })?;

        let response = ListGatewaysResponse {
            gateways: gateways
                .into_iter()
                .map(nym_vpn_proto::GatewayResponse::from)
                .collect(),
        };

        info!(
            "Returning list gateways response: {} entries",
            response.gateways.len()
        );
        Ok(tonic::Response::new(response))
    }

    async fn list_countries(
        &self,
        request: tonic::Request<ListCountriesRequest>,
    ) -> Result<tonic::Response<ListCountriesResponse>, tonic::Status> {
        info!("Got list entry countries request: {request:?}");

        let request = request.into_inner();

        let gw_type = nym_vpn_proto::GatewayType::try_from(request.kind)
            .ok()
            .and_then(crate::command_interface::protobuf::gateway::into_gateway_type)
            .ok_or_else(|| {
                let msg = format!("Failed to parse list countries kind: {}", request.kind);
                error!(msg);
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
                error!(msg);
                tonic::Status::internal(msg)
            })?;

        let response = nym_vpn_proto::ListCountriesResponse {
            countries: countries
                .into_iter()
                .map(nym_vpn_proto::Location::from)
                .collect(),
        };

        info!(
            "Returning list countries response: {} countries",
            response.countries.len()
        );
        Ok(tonic::Response::new(response))
    }

    async fn store_account(
        &self,
        request: tonic::Request<StoreAccountRequest>,
    ) -> Result<tonic::Response<StoreAccountResponse>, tonic::Status> {
        info!("Got store account request");

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

        info!("Returning store account response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn remove_account(
        &self,
        _request: tonic::Request<RemoveAccountRequest>,
    ) -> Result<tonic::Response<RemoveAccountResponse>, tonic::Status> {
        info!("Got remove account request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_remove_account()
            .await;

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

        info!("Returning remove account response");
        Ok(tonic::Response::new(response))
    }

    async fn get_account_summary(
        &self,
        _request: tonic::Request<GetAccountSummaryRequest>,
    ) -> Result<tonic::Response<GetAccountSummaryResponse>, tonic::Status> {
        info!("Got get account summary request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_summary()
            .await;

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

        info!("Returning get account summary response");
        Ok(tonic::Response::new(response))
    }

    async fn get_devices(
        &self,
        _request: tonic::Request<GetDevicesRequest>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        info!("Got get devices request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_devices()
            .await;

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

        info!("Returning get devices response");
        Ok(tonic::Response::new(response))
    }

    async fn register_device(
        &self,
        _request: tonic::Request<RegisterDeviceRequest>,
    ) -> Result<tonic::Response<RegisterDeviceResponse>, tonic::Status> {
        info!("Got register device request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_register_device()
            .await;

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

        info!("Returning register device response");
        Ok(tonic::Response::new(response))
    }

    async fn request_zk_nym(
        &self,
        _request: tonic::Request<RequestZkNymRequest>,
    ) -> Result<tonic::Response<RequestZkNymResponse>, tonic::Status> {
        info!("Got request zk nym request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_request_zk_nym()
            .await;

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

        info!("Returning request zk nym response");
        Ok(tonic::Response::new(response))
    }

    async fn get_device_zk_nyms(
        &self,
        _request: tonic::Request<GetDeviceZkNymsRequest>,
    ) -> Result<tonic::Response<GetDeviceZkNymsResponse>, tonic::Status> {
        info!("Got get device zk nyms request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_device_zk_nyms()
            .await;

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

        info!("Returning get device zk nyms response");
        Ok(tonic::Response::new(response))
    }

    async fn get_free_passes(
        &self,
        _request: tonic::Request<nym_vpn_proto::GetFreePassesRequest>,
    ) -> Result<tonic::Response<nym_vpn_proto::GetFreePassesResponse>, tonic::Status> {
        info!("Got get free passes request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_free_passes()
            .await;

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

        info!("Returning get free passes response");
        Ok(tonic::Response::new(response))
    }

    async fn apply_freepass(
        &self,
        request: tonic::Request<nym_vpn_proto::ApplyFreepassRequest>,
    ) -> Result<tonic::Response<nym_vpn_proto::ApplyFreepassResponse>, tonic::Status> {
        info!("Got apply freepass request");

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

        info!("Returning apply freepass response");
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
