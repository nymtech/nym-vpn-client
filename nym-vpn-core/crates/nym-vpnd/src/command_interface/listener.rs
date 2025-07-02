// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::{StreamExt, stream::BoxStream};
use nym_vpn_api_client::NetworkCompatibility;
use tokio::sync::{broadcast, mpsc::UnboundedSender, oneshot};

use nym_vpn_lib_types::TunnelEvent;
use nym_vpn_proto::{
    AccountManagement, AvailableTickets, ConfirmZkNymDownloadedRequest,
    ConfirmZkNymDownloadedResponse, ConnectRequest, ConnectResponse, DeleteLogFileResponse,
    DisableSentryResponse, DisconnectResponse, EnableSentryResponse, ForgetAccountResponse,
    GetAccountIdentityResponse, GetAccountLinksRequest, GetAccountStateResponse,
    GetAccountUsageResponse, GetDeviceIdentityResponse, GetDeviceZkNymsResponse,
    GetDevicesResponse, GetFeatureFlagsResponse, GetLogPathResponse,
    GetNetworkCompatibilityResponse, GetSystemMessagesResponse, GetZkNymByIdRequest,
    GetZkNymByIdResponse, GetZkNymsAvailableForDownloadResponse, InfoResponse,
    IsAccountStoredResponse, IsSentryEnabledResponse, ListCountriesRequest, ListCountriesResponse,
    ListGatewaysRequest, ListGatewaysResponse, RefreshAccountStateResponse, RegisterDeviceResponse,
    RequestZkNymResponse, ResetDeviceIdentityRequest, ResetDeviceIdentityResponse,
    SetNetworkRequest, SetNetworkResponse, StoreAccountRequest, StoreAccountResponse, TunnelState,
    conversions::ConversionError, get_account_state_response::AccountStateSummary,
    get_account_usage_response::AccountUsages, get_devices_response::Devices,
    nym_vpnd_server::NymVpnd,
};
use zeroize::Zeroizing;

use super::{
    error::CommandInterfaceError,
    helpers::{parse_entry_point, parse_exit_point},
};
use crate::{
    logging::LogPath,
    service::{
        ConnectArgs, ConnectOptions, ListCountriesOptions, ListGatewaysOptions, VpnServiceCommand,
    },
};

pub(super) struct CommandInterface {
    // Send commands to the VPN service
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,

    // Broadcast tunnel events to our API endpoint listeners
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
}

impl CommandInterface {
    pub(super) fn new(
        vpn_command_tx: UnboundedSender<VpnServiceCommand>,
        tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    ) -> Self {
        Self {
            vpn_command_tx,
            tunnel_event_rx,
        }
    }

    async fn send_and_wait<R, F, O>(&self, command: F, opts: O) -> Result<R, tonic::Status>
    where
        F: FnOnce(oneshot::Sender<R>, O) -> VpnServiceCommand,
    {
        let (tx, rx) = oneshot::channel();

        self.vpn_command_tx
            .send(command(tx, opts))
            .map_err(|_| tonic::Status::internal("Command channel is closed"))?;

        rx.await
            .map_err(|_| tonic::Status::internal("Response channel is closed"))
    }
}

#[tonic::async_trait]
impl NymVpnd for CommandInterface {
    async fn info(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<InfoResponse>, tonic::Status> {
        let info = self.send_and_wait(VpnServiceCommand::Info, ()).await?;
        let response = InfoResponse::from(info);

        Ok(tonic::Response::new(response))
    }

    async fn set_network(
        &self,
        request: tonic::Request<SetNetworkRequest>,
    ) -> Result<tonic::Response<SetNetworkResponse>, tonic::Status> {
        let network = request.into_inner().network;

        let status = self
            .send_and_wait(VpnServiceCommand::SetNetwork, network)
            .await?;

        let response = nym_vpn_proto::SetNetworkResponse {
            error: status
                .err()
                .map(nym_vpn_proto::SetNetworkRequestError::from),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_system_messages(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetSystemMessagesResponse>, tonic::Status> {
        let messages = self
            .send_and_wait(VpnServiceCommand::GetSystemMessages, ())
            .await?;

        let messages = messages.into_current_iter().map(|m| m.into()).collect();
        let response = GetSystemMessagesResponse { messages };

        Ok(tonic::Response::new(response))
    }

    async fn get_network_compatibility(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetNetworkCompatibilityResponse>, tonic::Status> {
        let compatibility = self
            .send_and_wait(VpnServiceCommand::GetNetworkCompatibility, ())
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
        let feature_flags = self
            .send_and_wait(VpnServiceCommand::GetFeatureFlags, ())
            .await?
            .ok_or(tonic::Status::not_found("Feature flags not found"))?;

        Ok(tonic::Response::new(feature_flags.into()))
    }

    async fn vpn_connect(
        &self,
        request: tonic::Request<ConnectRequest>,
    ) -> Result<tonic::Response<ConnectResponse>, tonic::Status> {
        let connect_request = request.into_inner();
        let entry = connect_request
            .entry
            .clone()
            .and_then(|e| e.entry_node_enum)
            .map(parse_entry_point)
            .transpose()
            .map_err(|err| *err)?;

        let exit = connect_request
            .exit
            .clone()
            .and_then(|e| e.exit_node_enum)
            .map(parse_exit_point)
            .transpose()
            .map_err(|err| *err)?;

        let options = ConnectOptions::try_from(connect_request).map_err(|err| {
            tonic::Status::invalid_argument(format!("Invalid connect options: {err}"))
        })?;

        let connect_args = ConnectArgs {
            entry,
            exit,
            options,
        };

        let status = self
            .send_and_wait(VpnServiceCommand::Connect, connect_args)
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

        Ok(tonic::Response::new(response))
    }

    async fn vpn_disconnect(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<DisconnectResponse>, tonic::Status> {
        let status = self
            .send_and_wait(VpnServiceCommand::Disconnect, ())
            .await?;

        let response = DisconnectResponse {
            success: status.is_ok(),
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_tunnel_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<TunnelState>, tonic::Status> {
        let tunnel_state = self
            .send_and_wait(VpnServiceCommand::GetTunnelState, ())
            .await
            .map(TunnelState::from)?;

        Ok(tonic::Response::new(tunnel_state))
    }

    type ListenToTunnelStateStream = BoxStream<'static, Result<TunnelState, tonic::Status>>;
    async fn listen_to_tunnel_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::ListenToTunnelStateStream>, tonic::Status> {
        let rx = self
            .send_and_wait(VpnServiceCommand::SubscribeToTunnelState, ())
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
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::ListenToEventsStream>, tonic::Status> {
        let rx = self.tunnel_event_rx.resubscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|event| {
            event
                .map(nym_vpn_proto::TunnelEvent::from)
                .map_err(|_| tonic::Status::internal("Failed to receive tunnel event"))
        });
        Ok(tonic::Response::new(
            Box::pin(stream) as Self::ListenToEventsStream
        ))
    }

    async fn list_gateways(
        &self,
        request: tonic::Request<ListGatewaysRequest>,
    ) -> Result<tonic::Response<ListGatewaysResponse>, tonic::Status> {
        let options = ListGatewaysOptions::try_from(request.into_inner())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let gateways = self
            .send_and_wait(VpnServiceCommand::ListGateways, options)
            .await?
            .map_err(|err| tonic::Status::internal(format!("Failed to list gateways: {err}")))?;

        let response = ListGatewaysResponse {
            gateways: gateways
                .into_iter()
                .map(nym_vpn_proto::GatewayResponse::from)
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn list_countries(
        &self,
        request: tonic::Request<ListCountriesRequest>,
    ) -> Result<tonic::Response<ListCountriesResponse>, tonic::Status> {
        let options = ListCountriesOptions::try_from(request.into_inner())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let countries = self
            .send_and_wait(VpnServiceCommand::ListCountries, options)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to list entry countries: {err}"))
            })?;

        let response = ListCountriesResponse {
            countries: countries
                .into_iter()
                .map(nym_vpn_proto::Location::from)
                .collect(),
        };

        Ok(tonic::Response::new(response))
    }

    async fn store_account(
        &self,
        request: tonic::Request<StoreAccountRequest>,
    ) -> Result<tonic::Response<StoreAccountResponse>, tonic::Status> {
        let account = Zeroizing::new(request.into_inner().mnemonic);

        let result = self
            .send_and_wait(VpnServiceCommand::StoreAccount, account)
            .await?;

        let response = StoreAccountResponse {
            error: result.err().map(nym_vpn_proto::StoreAccountError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn is_account_stored(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<IsAccountStoredResponse>, tonic::Status> {
        let is_stored = self
            .send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await?;

        Ok(tonic::Response::new(IsAccountStoredResponse { is_stored }))
    }

    async fn forget_account(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<ForgetAccountResponse>, tonic::Status> {
        let result = self
            .send_and_wait(VpnServiceCommand::ForgetAccount, ())
            .await?;

        let response = ForgetAccountResponse {
            error: result.err().map(nym_vpn_proto::ForgetAccountError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_account_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAccountIdentityResponse>, tonic::Status> {
        let account_identity = self
            .send_and_wait(VpnServiceCommand::GetAccountIdentity, ())
            .await?;

        Ok(tonic::Response::new(GetAccountIdentityResponse {
            account_identity,
        }))
    }

    async fn get_account_links(
        &self,
        request: tonic::Request<GetAccountLinksRequest>,
    ) -> Result<tonic::Response<AccountManagement>, tonic::Status> {
        let locale = request.into_inner().locale;

        let account_links = self
            .send_and_wait(VpnServiceCommand::GetAccountLinks, locale)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account links: {err}"))
            })?;

        Ok(tonic::Response::new(AccountManagement::from(account_links)))
    }

    async fn get_account_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAccountStateResponse>, tonic::Status> {
        let account_state_summary = self
            .send_and_wait(VpnServiceCommand::GetAccountState, ())
            .await?;

        Ok(tonic::Response::new(GetAccountStateResponse {
            account: Some(AccountStateSummary::from(account_state_summary)),
        }))
    }

    async fn refresh_account_state(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<RefreshAccountStateResponse>, tonic::Status> {
        self.send_and_wait(VpnServiceCommand::RefreshAccountState, ())
            .await?;

        Ok(tonic::Response::new(RefreshAccountStateResponse {}))
    }

    async fn get_account_usage(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetAccountUsageResponse>, tonic::Status> {
        let account_usage = self
            .send_and_wait(VpnServiceCommand::GetAccountUsage, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get account usage: {err}"))
            })?;

        Ok(tonic::Response::new(GetAccountUsageResponse {
            account_usages: Some(AccountUsages::from(account_usage)),
        }))
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

        self.send_and_wait(VpnServiceCommand::ResetDeviceIdentity, seed)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to reset device identity: {err}"))
            })?;

        Ok(tonic::Response::new(ResetDeviceIdentityResponse {}))
    }

    async fn get_device_identity(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDeviceIdentityResponse>, tonic::Status> {
        let device_identity = self
            .send_and_wait(VpnServiceCommand::GetDeviceIdentity, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get device identity: {err}"))
            })?;

        Ok(tonic::Response::new(GetDeviceIdentityResponse {
            device_identity,
        }))
    }

    async fn register_device(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<RegisterDeviceResponse>, tonic::Status> {
        self.send_and_wait(VpnServiceCommand::RegisterDevice, ())
            .await?;
        Ok(tonic::Response::new(RegisterDeviceResponse {}))
    }

    async fn get_devices(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        let devices = self
            .send_and_wait(VpnServiceCommand::GetDevices, ())
            .await?
            .map_err(|err| tonic::Status::internal(format!("Failed to get devices: {err}")))?;

        Ok(tonic::Response::new(GetDevicesResponse {
            devices: Some(Devices::from(devices)),
        }))
    }

    async fn get_active_devices(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDevicesResponse>, tonic::Status> {
        let devices = self
            .send_and_wait(VpnServiceCommand::GetActiveDevices, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get active devices: {err}"))
            })?;

        Ok(tonic::Response::new(GetDevicesResponse {
            devices: Some(Devices::from(devices)),
        }))
    }

    async fn request_zk_nym(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<RequestZkNymResponse>, tonic::Status> {
        self.send_and_wait(VpnServiceCommand::RequestZkNym, ())
            .await?;
        Ok(tonic::Response::new(RequestZkNymResponse {}))
    }

    async fn get_device_zk_nyms(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetDeviceZkNymsResponse>, tonic::Status> {
        // Internal command where returning the result is not yet implemented. It's primary
        // implementation is to trigger the command interface.
        let _ = self
            .send_and_wait(VpnServiceCommand::GetDeviceZkNyms, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get device zk nyms: {err}"))
            })?;

        Ok(tonic::Response::new(GetDeviceZkNymsResponse {}))
    }

    async fn get_zk_nyms_available_for_download(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<GetZkNymsAvailableForDownloadResponse>, tonic::Status> {
        // Internal command where returning the result is not yet implemented. It's primary
        // purpose is to trigger the command interface.
        let _ = self
            .send_and_wait(VpnServiceCommand::GetZkNymsAvailableForDownload, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!(
                    "Failed to get zknyms available for download: {err}",
                ))
            })?;

        Ok(tonic::Response::new(
            GetZkNymsAvailableForDownloadResponse {},
        ))
    }

    async fn get_zk_nym_by_id(
        &self,
        request: tonic::Request<GetZkNymByIdRequest>,
    ) -> Result<tonic::Response<GetZkNymByIdResponse>, tonic::Status> {
        let id = request.into_inner().id;

        // This is an internal command, and returning the ID is not yet implemented. It's primary
        // purpose is to trigger the command interface.
        let _ = self
            .send_and_wait(VpnServiceCommand::GetZkNymById, id)
            .await?
            .map_err(|err| tonic::Status::internal(format!("Failed to get zknym by id: {err}")))?;
        Ok(tonic::Response::new(GetZkNymByIdResponse {}))
    }

    async fn confirm_zk_nym_downloaded(
        &self,
        request: tonic::Request<ConfirmZkNymDownloadedRequest>,
    ) -> Result<tonic::Response<ConfirmZkNymDownloadedResponse>, tonic::Status> {
        let id = request.into_inner().id;

        self.send_and_wait(VpnServiceCommand::ConfirmZkNymIdDownloaded, id)
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to confirm zk nym downloaded: {err}"))
            })?;

        Ok(tonic::Response::new(ConfirmZkNymDownloadedResponse {}))
    }

    async fn get_available_tickets(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<AvailableTickets>, tonic::Status> {
        let available_ticketbooks = self
            .send_and_wait(VpnServiceCommand::GetAvailableTickets, ())
            .await?
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get available tickets: {err}"))
            })?;

        let available_tickets = nym_vpn_lib_types::AvailableTickets::from(available_ticketbooks);
        let response = AvailableTickets::from(available_tickets);

        Ok(tonic::Response::new(response))
    }

    async fn delete_log_file(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<DeleteLogFileResponse>, tonic::Status> {
        let result = self
            .send_and_wait(VpnServiceCommand::DeleteLogFile, ())
            .await
            .map_err(|err| {
                tonic::Status::internal(format!("Failed to get available tickets: {err}"))
            })?;

        let response = match result {
            Ok(_) => DeleteLogFileResponse {
                success: true,
                error: None,
            },
            Err(err) => DeleteLogFileResponse {
                success: false,
                error: Some(nym_vpn_proto::DeleteLogFileError::from(err)),
            },
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_log_path(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<GetLogPathResponse>, tonic::Status> {
        let log_path = self
            .send_and_wait(VpnServiceCommand::GetLogPath, ())
            .await?
            .unwrap_or_default();

        Ok(tonic::Response::new(log_path.into()))
    }

    async fn is_sentry_enabled(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<IsSentryEnabledResponse>, tonic::Status> {
        let result = self
            .send_and_wait(VpnServiceCommand::IsSentryEnabled, ())
            .await?;
        let response = IsSentryEnabledResponse { enabled: result };
        Ok(tonic::Response::new(response))
    }

    async fn enable_sentry(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<EnableSentryResponse>, tonic::Status> {
        let result = self
            .send_and_wait(VpnServiceCommand::ToggleSentry, true)
            .await?
            .inspect_err(|err| {
                tracing::error!("Failed to enable sentry monitoring: {err}");
            });
        let response = EnableSentryResponse {
            success: result.is_ok(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn disable_sentry(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<DisableSentryResponse>, tonic::Status> {
        let result = self
            .send_and_wait(VpnServiceCommand::ToggleSentry, false)
            .await?
            .inspect_err(|err| {
                tracing::error!("Failed to disable sentry monitoring: {err}");
            });
        let response = DisableSentryResponse {
            success: result.is_ok(),
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
                    .map_err(|err| CommandInterfaceError::ParseDnsIp {
                        ip: dns.ip.clone(),
                        source: err,
                    })
            })
            .transpose()?;

        let disable_background_cover_traffic = if request.enable_two_hop {
            // If two-hop is enabled, we always disable background cover traffic
            true
        } else {
            request.disable_background_cover_traffic
        };

        let user_agent = request.user_agent.map(nym_vpn_lib::UserAgent::from);

        Ok(ConnectOptions {
            dns,
            enable_two_hop: request.enable_two_hop,
            netstack: request.netstack,
            disable_poisson_rate: request.disable_poisson_rate,
            disable_background_cover_traffic,
            enable_credentials_mode: request.enable_credentials_mode,
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
            user_agent,
        })
    }
}

impl From<LogPath> for GetLogPathResponse {
    fn from(log_path: LogPath) -> Self {
        GetLogPathResponse {
            path: log_path.dir.to_string_lossy().to_string(),
            filename: log_path.filename.clone(),
        }
    }
}

impl TryFrom<ListGatewaysRequest> for ListGatewaysOptions {
    type Error = ConversionError;

    fn try_from(value: ListGatewaysRequest) -> Result<Self, Self::Error> {
        let gw_type = nym_vpn_proto::GatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("ListGatewaysRequest.kind", err))
            .and_then(nym_vpn_lib::gateway_directory::GatewayType::try_from)?;

        let user_agent = value.user_agent.map(nym_vpn_lib::UserAgent::from);

        Ok(Self {
            gw_type,
            user_agent,
        })
    }
}

impl TryFrom<ListCountriesRequest> for ListCountriesOptions {
    type Error = ConversionError;

    fn try_from(value: ListCountriesRequest) -> Result<Self, Self::Error> {
        let gw_type = nym_vpn_proto::GatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("ListCountriesRequest.kind", err))
            .and_then(nym_vpn_lib::gateway_directory::GatewayType::try_from)?;

        let user_agent = value.user_agent.map(nym_vpn_lib::UserAgent::from);

        Ok(Self {
            gw_type,
            user_agent,
        })
    }
}
