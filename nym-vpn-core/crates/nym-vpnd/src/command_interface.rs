// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use futures::{StreamExt, stream::BoxStream};
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;

use nym_vpn_lib_types::TunnelEvent;
use nym_vpn_proto::{
    AccountManagement, AvailableTickets, ConfirmZkNymDownloadedRequest,
    ConfirmZkNymDownloadedResponse, ConnectRequest, ForgetAccountResponse,
    GetAccountIdentityResponse, GetAccountLinksRequest, GetAccountStateResponse,
    GetAccountUsageResponse, GetDeviceIdentityResponse, GetDeviceZkNymsResponse,
    GetDevicesResponse, GetFeatureFlagsResponse, GetLogPathResponse,
    GetNetworkCompatibilityResponse, GetSystemMessagesResponse, GetZkNymByIdRequest,
    GetZkNymByIdResponse, GetZkNymsAvailableForDownloadResponse, InfoResponse,
    ListCountriesRequest, ListCountriesResponse, ListGatewaysRequest, ListGatewaysResponse,
    NetworkCompatibility, RefreshAccountStateResponse, RegisterDeviceResponse,
    RequestZkNymResponse, ResetDeviceIdentityRequest, ResetDeviceIdentityResponse,
    StoreAccountRequest, StoreAccountResponse, TunnelState,
    get_account_state_response::AccountStateSummary,
    get_account_usage_response::AccountUsages,
    get_devices_response::Devices,
    nym_vpnd_server::{NymVpnd, NymVpndServer},
};
use nym_vpnd_types::{ConnectArgs, ListCountriesOptions, ListGatewaysOptions};

use crate::service::{SetNetworkError, VpnServiceCommand};

pub struct CommandInterface {
    // Send commands to the VPN service
    vpn_command_tx: UnboundedSender<VpnServiceCommand>,

    // Broadcast tunnel events to our API endpoint listeners
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
}

impl CommandInterface {
    fn new(
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
        request: tonic::Request<String>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let network = request.into_inner();
        let status = self
            .send_and_wait(VpnServiceCommand::SetNetwork, network)
            .await?;

        status.map_err(|e| match e {
            SetNetworkError::NetworkNotFound(network_name) => {
                tonic::Status::not_found(format!("Network not found: {network_name}"))
            }
            e => tonic::Status::internal(e.to_string()),
        })?;

        Ok(tonic::Response::new(()))
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
        let network_compatibility = self
            .send_and_wait(VpnServiceCommand::GetNetworkCompatibility, ())
            .await?
            .map(NetworkCompatibility::from);

        let response = GetNetworkCompatibilityResponse {
            network_compatibility,
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
    ) -> Result<tonic::Response<bool>, tonic::Status> {
        let connect_args = ConnectArgs::try_from(request.into_inner())
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let status = self
            .send_and_wait(VpnServiceCommand::Connect, connect_args)
            .await?;

        Ok(tonic::Response::new(status))
    }

    async fn vpn_disconnect(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>, tonic::Status> {
        let status = self
            .send_and_wait(VpnServiceCommand::Disconnect, ())
            .await?;

        Ok(tonic::Response::new(status.is_ok()))
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
        let store_request = nym_vpnd_types::StoreAccountRequest::from(request.into_inner());
        let result = self
            .send_and_wait(VpnServiceCommand::StoreAccount, store_request)
            .await?;

        let response = StoreAccountResponse {
            error: result.err().map(nym_vpn_proto::StoreAccountError::from),
        };

        Ok(tonic::Response::new(response))
    }

    async fn is_account_stored(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>, tonic::Status> {
        let is_stored = self
            .send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await?;

        Ok(tonic::Response::new(is_stored))
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
    ) -> Result<tonic::Response<bool>, tonic::Status> {
        let success = self
            .send_and_wait(VpnServiceCommand::DeleteLogFile, ())
            .await?;

        Ok(tonic::Response::new(success))
    }

    async fn get_log_path(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<GetLogPathResponse>, tonic::Status> {
        let log_path = self
            .send_and_wait(VpnServiceCommand::GetLogPath, ())
            .await?
            .unwrap_or(crate::logging::default_log_path());

        Ok(tonic::Response::new(log_path.into()))
    }

    async fn is_sentry_enabled(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<bool>, tonic::Status> {
        let result = self
            .send_and_wait(VpnServiceCommand::IsSentryEnabled, ())
            .await?;
        Ok(tonic::Response::new(result))
    }

    async fn enable_sentry(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.send_and_wait(VpnServiceCommand::ToggleSentry, true)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to enable sentry monitoring: {err}");
                tonic::Status::internal("failed to enable sentry")
            })?;
        Ok(tonic::Response::new(()))
    }

    async fn disable_sentry(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.send_and_wait(VpnServiceCommand::ToggleSentry, false)
            .await?
            .map_err(|err| {
                tracing::error!("Failed to disable sentry monitoring: {err}");
                tonic::Status::internal("failed to disable sentry")
            })?;
        Ok(tonic::Response::new(()))
    }
}

pub async fn start_command_interface(
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
    shutdown_token: CancellationToken,
) -> std::io::Result<(JoinHandle<()>, UnboundedReceiver<VpnServiceCommand>)> {
    tracing::debug!("Starting command interface");

    let socket_path = default_socket_path();
    let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();

    // Remove previous socket file in case if the daemon crashed in the prior run and could not clean up the socket file.
    #[cfg(unix)]
    remove_previous_socket_file(&socket_path).await;
    tracing::info!("Starting socket listener on: {}", socket_path.display());

    // Wrap the unix socket or named pipe into a stream that can be used by tonic
    let incoming = nym_ipc::server::create_incoming(socket_path.clone())?;

    let server_handle = tokio::spawn(async move {
        let incoming_shutdown_token = shutdown_token.child_token();
        let socket_listener_handle = tokio::spawn(async move {
            let command_interface = CommandInterface::new(vpn_command_tx, tunnel_event_rx);

            let server = Server::builder().add_service(NymVpndServer::new(command_interface));

            match server
                .serve_with_incoming_shutdown(incoming, incoming_shutdown_token.cancelled_owned())
                .await
            {
                Ok(()) => {
                    tracing::info!("Socket listener has finished");
                }
                Err(e) => {
                    tracing::error!("Socket listener exited with error: {}", e);
                }
            }
        });

        if let Err(e) = socket_listener_handle.await {
            tracing::error!("Failed to join on socket listener: {}", e);
        }

        tracing::info!("Command interface exiting");
    });

    Ok((server_handle, vpn_command_rx))
}

#[cfg(unix)]
async fn remove_previous_socket_file(socket_path: &std::path::Path) {
    match tokio::fs::remove_file(socket_path).await {
        Ok(_) => tracing::info!(
            "Removed previous command interface socket: {}",
            socket_path.display()
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

fn default_socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/var/run/nym-vpn.sock")
    }

    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\nym-vpn")
    }
}
