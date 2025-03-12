// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use futures::{stream::BoxStream, StreamExt};
use nym_vpn_network_config::{Network, NetworkCompatibility};
use tokio::sync::{broadcast, mpsc::UnboundedSender};

use nym_vpn_api_client::types::{GatewayMinPerformance, ScoreThresholds};
use nym_vpn_lib_types::TunnelEvent;
use nym_vpn_proto::{
    conversions::ConversionError, nym_vpnd_server::NymVpnd, AccountError,
    ConfirmZkNymDownloadedRequest, ConfirmZkNymDownloadedResponse, ConnectRequest, ConnectResponse,
    DisconnectResponse, ForgetAccountResponse, GetAccountIdentityResponse, GetAccountLinksRequest,
    GetAccountLinksResponse, GetAccountStateResponse, GetAccountUsageResponse,
    GetAvailableTicketsResponse, GetDeviceIdentityResponse, GetDeviceZkNymsResponse,
    GetDevicesResponse, GetFeatureFlagsResponse, GetNetworkCompatibilityResponse,
    GetSystemMessagesResponse, GetZkNymByIdRequest, GetZkNymByIdResponse,
    GetZkNymsAvailableForDownloadResponse, InfoResponse, IsAccountStoredResponse,
    ListCountriesRequest, ListCountriesResponse, ListGatewaysRequest, ListGatewaysResponse,
    RefreshAccountStateResponse, RegisterDeviceResponse, RequestZkNymResponse,
    ResetDeviceIdentityRequest, ResetDeviceIdentityResponse, SetNetworkRequest, SetNetworkResponse,
    StoreAccountRequest, StoreAccountResponse, TunnelState,
};
use zeroize::Zeroizing;

use super::{
    connection_handler::CommandInterfaceConnectionHandler,
    error::CommandInterfaceError,
    helpers::{parse_entry_point, parse_exit_point, threshold_into_percent},
};
use crate::{
    command_interface::protobuf::info_response::into_proto_available_tickets,
    service::{ConnectOptions, VpnServiceCommand},
};

enum ListenerType {
    Path(PathBuf),
    Uri(#[allow(unused)] SocketAddr),
}

pub(super) struct CommandInterface {
    // Send commands to the VPN service
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,

    // Broadcast tunnel events to our API endpoint listeners
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,

    listener: ListenerType,

    network_env: Network,
}

impl CommandInterface {
    pub(super) fn new_with_path(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
        socket_path: &Path,
        network_env: Network,
    ) -> Self {
        Self {
            vpn_command_tx,
            tunnel_event_rx,
            listener: ListenerType::Path(socket_path.to_path_buf()),
            network_env,
        }
    }

    pub(super) fn new_with_uri(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
        uri: SocketAddr,
        network_env: Network,
    ) -> Self {
        Self {
            vpn_command_tx,
            tunnel_event_rx,
            listener: ListenerType::Uri(uri),
            network_env,
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
        _request: tonic::Request<()>,
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

    async fn get_system_messages(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetSystemMessagesResponse>, tonic::Status> {
        tracing::debug!("Got get system messages request");

        let messages = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_system_messages()
            .await?;

        let messages = messages.into_current_iter().map(|m| m.into()).collect();
        let response = GetSystemMessagesResponse { messages };

        Ok(tonic::Response::new(response))
    }

    async fn get_network_compatibility(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetNetworkCompatibilityResponse>, tonic::Status> {
        tracing::debug!("Got get system messages request");

        let compatibility = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_network_compatibility()
            .await?;

        let compatibility = compatibility.map(NetworkCompatibility::into);
        let response = GetNetworkCompatibilityResponse {
            messages: compatibility,
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_feature_flags(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetFeatureFlagsResponse>, tonic::Status> {
        tracing::debug!("Got get feature flags request");

        let feature_flags = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_feature_flags()
            .await?
            .ok_or(tonic::Status::not_found("Feature flags not found"))?;

        Ok(tonic::Response::new(feature_flags.into()))
    }

    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        let connect_request = request.into_inner();
        tracing::debug!("Got connect request: {connect_request:?}");

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

        let options = ConnectOptions::try_from(connect_request).map_err(|err| {
            tracing::error!("Failed to parse connect options: {:?}", err);
            tonic::Status::invalid_argument("Invalid connect options")
        })?;

        let status = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_connect(entry, exit, options)
            .await?;

        let response = match status {
            Ok(()) => ConnectResponse {
                success: true,
                error: None,
            },
            Err(err) => {
                tracing::debug!("Connect request error: {:?}", err);
                ConnectResponse {
                    success: false,
                    error: Some(nym_vpn_proto::ConnectRequestError::from(err)),
                }
            }
        };

        tracing::debug!("Returning connect response: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn vpn_disconnect(
        &self,
        _request: tonic::Request<()>,
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

    async fn get_tunnel_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<TunnelState>, tonic::Status> {
        let tunnel_state = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_status()
            .await
            .map(TunnelState::from)?;

        tracing::debug!("Returning tunnel state: {:?}", tunnel_state);
        Ok(tonic::Response::new(tunnel_state))
    }

    type ListenToTunnelStateStream = BoxStream<'static, Result<TunnelState, tonic::Status>>;
    async fn listen_to_tunnel_state(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::ListenToTunnelStateStream>, tonic::Status> {
        tracing::debug!("Got connection status stream request: {request:?}");

        let rx = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_subscribe_to_tunnel_state()
            .await?;
        let stream = tokio_stream::wrappers::WatchStream::new(rx)
            .map(|new_state| Ok(TunnelState::from(new_state)));
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToTunnelStateStream
        ))
    }

    type ListenToEventsStream =
        BoxStream<'static, Result<nym_vpn_proto::TunnelEvent, tonic::Status>>;
    async fn listen_to_events(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::ListenToEventsStream>, tonic::Status> {
        tracing::debug!("Got daemon events stream request: {request:?}");

        let rx = self.tunnel_event_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|event| {
            event.map(nym_vpn_proto::TunnelEvent::from).map_err(|err| {
                tracing::error!("Failed to receive tunnel event: {:?}", err);
                tonic::Status::internal("Failed to receive tunnel event")
            })
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToEventsStream
        ))
    }

    async fn list_gateways(
        &self,
        request: tonic::Request<ListGatewaysRequest>,
    ) -> Result<tonic::Response<ListGatewaysResponse>, tonic::Status> {
        tracing::debug!("Got list gateways request: {:?}", request);

        let request = request.into_inner();

        let gw_type = nym_vpn_proto::GatewayType::try_from(request.kind)
            // .and_then(crate::command_interface::protobuf::gateway::into_gateway_type)
            // TODO: do this conversion in one step instead
            .map_err(|err| ConversionError::Generic(err.to_string()))
            .and_then(nym_vpn_lib::gateway_directory::GatewayType::try_from)
            .map_err(|_err| {
                let msg = format!("Failed to parse gateway type: {}", request.kind);
                tracing::error!(msg);
                tonic::Status::invalid_argument(msg)
            })?;

        let user_agent = request
            .user_agent
            .map(nym_vpn_lib::UserAgent::from)
            .unwrap_or_else(crate::util::construct_user_agent);

        let min_mixnet_performance = request.min_mixnet_performance.map(threshold_into_percent);
        let min_vpn_performance = request.min_vpn_performance.map(threshold_into_percent);

        let min_gateway_performance = Some(GatewayMinPerformance {
            mixnet_min_performance: min_mixnet_performance,
            vpn_min_performance: min_vpn_performance,
        });
        let mix_score_thresholds =
            self.network_env
                .system_configuration
                .map(|sc| ScoreThresholds {
                    high: sc.mix_thresholds.high,
                    medium: sc.mix_thresholds.medium,
                    low: sc.mix_thresholds.low,
                });
        let wg_score_thresholds = self
            .network_env
            .system_configuration
            .map(|sc| ScoreThresholds {
                high: sc.wg_thresholds.high,
                medium: sc.wg_thresholds.medium,
                low: sc.wg_thresholds.low,
            });
        let directory_config = nym_vpn_lib::gateway_directory::Config {
            nyxd_url: self.network_env.nyxd_url(),
            api_url: self.network_env.api_url(),
            nym_vpn_api_url: Some(self.network_env.vpn_api_url()),
            min_gateway_performance,
            mix_score_thresholds,
            wg_score_thresholds,
        };

        let gateways = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_list_gateways(gw_type, user_agent, directory_config)
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
            .map_err(|err| ConversionError::Generic(err.to_string()))
            .and_then(nym_vpn_lib::gateway_directory::GatewayType::try_from)
            .map_err(|_err| {
                let msg = format!("Failed to parse list countries kind: {}", request.kind);
                tracing::error!(msg);
                tonic::Status::invalid_argument(msg)
            })?;

        let user_agent = request
            .user_agent
            .map(nym_vpn_lib::UserAgent::from)
            .unwrap_or_else(crate::util::construct_user_agent);

        let min_mixnet_performance = request.min_mixnet_performance.map(threshold_into_percent);
        let min_vpn_performance = request.min_vpn_performance.map(threshold_into_percent);

        let min_gateway_performance = Some(GatewayMinPerformance {
            mixnet_min_performance: min_mixnet_performance,
            vpn_min_performance: min_vpn_performance,
        });
        let mix_score_thresholds =
            self.network_env
                .system_configuration
                .map(|sc| ScoreThresholds {
                    high: sc.mix_thresholds.high,
                    medium: sc.mix_thresholds.medium,
                    low: sc.mix_thresholds.low,
                });
        let wg_score_thresholds = self
            .network_env
            .system_configuration
            .map(|sc| ScoreThresholds {
                high: sc.wg_thresholds.high,
                medium: sc.wg_thresholds.medium,
                low: sc.wg_thresholds.low,
            });
        let directory_config = nym_vpn_lib::gateway_directory::Config {
            nyxd_url: self.network_env.nyxd_url(),
            api_url: self.network_env.api_url(),
            nym_vpn_api_url: Some(self.network_env.vpn_api_url()),
            min_gateway_performance,
            mix_score_thresholds,
            wg_score_thresholds,
        };

        let countries = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_list_countries(gw_type, user_agent, directory_config)
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
        let account = Zeroizing::new(request.into_inner().mnemonic);

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
        _request: tonic::Request<()>,
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

    async fn forget_account(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<ForgetAccountResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_forget_account()
            .await?;

        let response = match result {
            Ok(()) => ForgetAccountResponse {
                success: true,
                error: None,
            },
            Err(err) => ForgetAccountResponse {
                success: false,
                error: Some(nym_vpn_proto::AccountError::from(err)),
            },
        };

        tracing::debug!("Returning forget account response");
        Ok(tonic::Response::new(response))
    }

    async fn get_account_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAccountIdentityResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_identity()
            .await
            .map_err(|err| {
                tracing::error!("Failed to get account identity: {:?}", err);
                tonic::Status::internal("Failed to get account identity")
            })?;

        let response = match result {
            Ok(identity) => GetAccountIdentityResponse {
                id: Some(
                    nym_vpn_proto::get_account_identity_response::Id::AccountIdentity(
                        nym_vpn_proto::AccountIdentity::from(identity),
                    ),
                ),
            },
            Err(err) => GetAccountIdentityResponse {
                id: Some(nym_vpn_proto::get_account_identity_response::Id::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_account_links(
        &self,
        request: tonic::Request<GetAccountLinksRequest>,
    ) -> Result<tonic::Response<GetAccountLinksResponse>, tonic::Status> {
        let locale = request.into_inner().locale;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_links(locale)
            .await?;

        let response = match result {
            Ok(account_links) => GetAccountLinksResponse {
                res: Some(nym_vpn_proto::get_account_links_response::Res::Links(
                    account_links.into(),
                )),
            },
            Err(err) => {
                tracing::error!("Failed to get account links: {:?}", err);
                GetAccountLinksResponse {
                    res: Some(nym_vpn_proto::get_account_links_response::Res::Error(
                        nym_vpn_proto::AccountError::from(err),
                    )),
                }
            }
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_account_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAccountStateResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_state()
            .await?;

        let response = match result {
            Ok(state) => GetAccountStateResponse {
                result: Some(nym_vpn_proto::get_account_state_response::Result::Account(
                    nym_vpn_proto::get_account_state_response::AccountStateSummary::from(state),
                )),
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

    async fn refresh_account_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<RefreshAccountStateResponse>, tonic::Status> {
        CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_refresh_account_state()
            .await?
            .map_err(|err| {
                tracing::error!("Failed to refresh account state: {:?}", err);
                tonic::Status::internal("Failed to refresh account state")
            })
            .map(|_| tonic::Response::new(RefreshAccountStateResponse {}))
    }

    async fn get_account_usage(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAccountUsageResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_account_usage()
            .await?;

        tracing::info!("Account usage: {:#?}", result);

        let response = match result {
            Ok(usage) => GetAccountUsageResponse {
                result: Some(
                    nym_vpn_proto::get_account_usage_response::Result::AccountUsages(
                        nym_vpn_proto::get_account_usage_response::AccountUsages::from(usage),
                    ),
                ),
            },
            Err(err) => GetAccountUsageResponse {
                result: Some(nym_vpn_proto::get_account_usage_response::Result::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn reset_device_identity(
        &self,
        request: tonic::Request<ResetDeviceIdentityRequest>,
    ) -> Result<tonic::Response<ResetDeviceIdentityResponse>, tonic::Status> {
        let seed: Option<[u8; 32]> = request
            .into_inner()
            .seed
            .map(|seed| {
                seed.as_slice()
                    .try_into()
                    .map_err(|_| tonic::Status::invalid_argument("Seed must be 32 bytes long"))
            })
            .transpose()?;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_reset_device_identity(seed)
            .await?;

        let response = ResetDeviceIdentityResponse {
            success: result.is_ok(),
            error: result.err().map(AccountError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_device_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDeviceIdentityResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_device_identity()
            .await?;

        let response = match result {
            Ok(identity) => GetDeviceIdentityResponse {
                id: Some(nym_vpn_proto::get_device_identity_response::Id::DeviceIdentity(identity)),
            },
            Err(err) => GetDeviceIdentityResponse {
                id: Some(nym_vpn_proto::get_device_identity_response::Id::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn register_device(
        &self,
        _request: tonic::Request<()>,
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

    async fn get_devices(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        let response = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_devices()
            .await?
            .map(|devices| GetDevicesResponse {
                result: Some(nym_vpn_proto::get_devices_response::Result::Devices(
                    nym_vpn_proto::get_devices_response::Devices::from(devices),
                )),
            })
            .unwrap_or_else(|err| GetDevicesResponse {
                result: Some(nym_vpn_proto::get_devices_response::Result::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
            });
        Ok(tonic::Response::new(response))
    }

    async fn get_active_devices(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        let response = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_active_devices()
            .await?
            .map(|devices| GetDevicesResponse {
                result: Some(nym_vpn_proto::get_devices_response::Result::Devices(
                    nym_vpn_proto::get_devices_response::Devices::from(devices),
                )),
            })
            .unwrap_or_else(|err| GetDevicesResponse {
                result: Some(nym_vpn_proto::get_devices_response::Result::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
            });
        Ok(tonic::Response::new(response))
    }

    async fn request_zk_nym(
        &self,
        _request: tonic::Request<()>,
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
        _request: tonic::Request<()>,
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

    async fn get_zk_nyms_available_for_download(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetZkNymsAvailableForDownloadResponse>, tonic::Status> {
        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_zk_nyms_available_for_download()
            .await?;

        let response = match result {
            Ok(response) => GetZkNymsAvailableForDownloadResponse {
                json: serde_json::to_string(&response)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => GetZkNymsAvailableForDownloadResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::debug!("Returning get zk nyms available to download response");
        Ok(tonic::Response::new(response))
    }

    async fn get_zk_nym_by_id(
        &self,
        request: tonic::Request<GetZkNymByIdRequest>,
    ) -> Result<tonic::Response<GetZkNymByIdResponse>, tonic::Status> {
        let id = request.into_inner().id;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_zk_nym_by_id(id)
            .await?;

        let response = match result {
            Ok(response) => GetZkNymByIdResponse {
                json: serde_json::to_string(&response)
                    .unwrap_or_else(|_| "failed to serialize".to_owned()),
                error: None,
            },
            Err(err) => GetZkNymByIdResponse {
                json: err.to_string(),
                error: Some(AccountError::from(err)),
            },
        };

        tracing::debug!("Returning get zk nym by id response");
        Ok(tonic::Response::new(response))
    }

    async fn confirm_zk_nym_downloaded(
        &self,
        request: tonic::Request<ConfirmZkNymDownloadedRequest>,
    ) -> Result<tonic::Response<ConfirmZkNymDownloadedResponse>, tonic::Status> {
        let id = request.into_inner().id;

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_confirm_zk_nym_downloaded(id)
            .await?;

        let response = match result {
            Ok(()) => ConfirmZkNymDownloadedResponse { error: None },
            Err(err) => ConfirmZkNymDownloadedResponse {
                error: Some(AccountError::from(err)),
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_available_tickets(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAvailableTicketsResponse>, tonic::Status> {
        tracing::debug!("Got get available tickets request");

        let result = CommandInterfaceConnectionHandler::new(self.vpn_command_tx.clone())
            .handle_get_available_tickets()
            .await
            .map_err(|err| {
                tracing::error!("Failed to get available tickets: {:?}", err);
                tonic::Status::internal("Failed to get available tickets")
            })?;

        let response = match result {
            Ok(ticketbooks) => GetAvailableTicketsResponse {
                resp: Some(
                    nym_vpn_proto::get_available_tickets_response::Resp::AvailableTickets(
                        into_proto_available_tickets(ticketbooks),
                    ),
                ),
            },
            Err(err) => GetAvailableTicketsResponse {
                resp: Some(nym_vpn_proto::get_available_tickets_response::Resp::Error(
                    nym_vpn_proto::AccountError::from(err),
                )),
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

        let user_agent = request
            .user_agent
            .map(nym_vpn_lib::UserAgent::from)
            .or(Some(crate::util::construct_user_agent()));

        Ok(ConnectOptions {
            dns,
            disable_routing: request.disable_routing,
            enable_two_hop: request.enable_two_hop,
            netstack: request.netstack,
            disable_poisson_rate: request.disable_poisson_rate,
            disable_background_cover_traffic,
            enable_credentials_mode: request.enable_credentials_mode,
            min_mixnode_performance,
            min_gateway_mixnet_performance,
            min_gateway_vpn_performance,
            user_agent,
        })
    }
}
