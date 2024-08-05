use std::path::PathBuf;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use nym_vpn_proto::{
    health_check_response::ServingStatus, health_client::HealthClient,
    nym_vpnd_client::NymVpndClient, ConnectionStatus, DisconnectRequest, EntryGateway, ExitGateway,
    HealthCheckRequest, ListEntryGatewaysRequest, ListExitGatewaysRequest, Location, StatusRequest,
};
use nym_vpn_proto::{
    ConnectRequest, Dns, Empty, EntryNode, ExitNode, ImportUserCredentialRequest,
    ImportUserCredentialResponse, StatusResponse,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use thiserror::Error;
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tonic::transport::Endpoint as TonicEndpoint;
use tonic::{transport::Channel, Request};
use tracing::{debug, error, info, instrument, warn};
use ts_rs::TS;

use crate::cli::Cli;
use crate::country::Country;
use crate::error::BackendError;
use crate::fs::config::AppConfig;
use crate::states::app::ConnectionState;
use crate::vpn_status;
use crate::{events::AppHandleEventEmitter, states::SharedAppState};

const VPND_SERVICE: &str = "nym.vpn.NymVpnd";
#[cfg(not(windows))]
const DEFAULT_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";
#[cfg(windows)]
const DEFAULT_SOCKET_PATH: &str = r"\\.\pipe\nym-vpn";
const DEFAULT_HTTP_ENDPOINT: &str = "http://[::1]:53181";

#[derive(Clone, Debug)]
enum Transport {
    Http(String),
    Ipc(PathBuf),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, TS)]
pub enum VpndStatus {
    Ok,
    #[default]
    NotOk,
}

#[derive(Error, Debug)]
pub enum VpndError {
    #[error("gRPC call error")]
    GrpcError(#[from] tonic::Status),
    #[error("failed to connect to daemon using HTTP transport")]
    FailedToConnectHttp(#[from] tonic::transport::Error),
    #[error("failed to connect to daemon using IPC transport")]
    FailedToConnectIpc(#[from] anyhow::Error),
}

#[derive(Debug, Default, Clone)]
pub struct GrpcClient(Transport);

impl GrpcClient {
    #[instrument(skip_all)]
    pub fn new(config: &AppConfig, cli: &Cli) -> Self {
        let client = GrpcClient(Transport::from((config, cli)));
        match &client.0 {
            Transport::Http(endpoint) => {
                info!("using grpc HTTP transport: {}", endpoint);
            }
            Transport::Ipc(socket) => {
                info!("using grpc IPC transport: {}", socket.display());
            }
        }
        client
    }

    /// Get the Vpnd service client
    #[instrument(skip_all)]
    pub async fn vpnd(&self) -> Result<NymVpndClient<Channel>, VpndError> {
        match &self.0 {
            Transport::Http(endpoint) => {
                NymVpndClient::connect(endpoint.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    VpndError::FailedToConnectHttp(e)
                })
            }
            Transport::Ipc(socket) => {
                let channel = get_channel(socket.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    VpndError::FailedToConnectIpc(e)
                })?;
                Ok(NymVpndClient::new(channel))
            }
        }
    }

    /// Get the Health service client
    #[instrument(skip_all)]
    pub async fn health(&self) -> Result<HealthClient<Channel>, VpndError> {
        match &self.0 {
            Transport::Http(endpoint) => {
                HealthClient::connect(endpoint.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    VpndError::FailedToConnectHttp(e)
                })
            }
            Transport::Ipc(socket) => {
                let channel = get_channel(socket.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    VpndError::FailedToConnectIpc(e)
                })?;
                Ok(HealthClient::new(channel))
            }
        }
    }

    /// Check the connection with the grpc server
    #[instrument(skip_all)]
    pub async fn check(&self, app_state: &SharedAppState) -> Result<VpndStatus> {
        let mut health = self.health().await?;

        let request = Request::new(HealthCheckRequest {
            service: VPND_SERVICE.into(),
        });
        let response = health
            .check(request)
            .await
            .inspect_err(|e| {
                error!("health check failed: {}", e);
            })?
            .into_inner();
        let status = response.status();
        let mut state = app_state.lock().await;
        state.vpnd_status = status.into();

        Ok(status.into())
    }

    /// Get VPN status
    #[instrument(skip_all)]
    pub async fn vpn_status(&self) -> Result<StatusResponse, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(StatusRequest {});
        let response = vpnd.vpn_status(request).await.map_err(|e| {
            error!("grpc vpn_status: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner())
    }

    /// Refresh VPN status
    #[instrument(skip_all)]
    pub async fn refresh_vpn_status(&self, app: &AppHandle) -> Result<(), VpndError> {
        let res = self.vpn_status().await?;
        debug!("vpn status update {:?}", res.status());
        if let Some(e) = res.error.as_ref() {
            warn!("vpn status error: {}", e.message);
        }
        let connection_time = res.details.clone().and_then(|d| {
            d.since.map(|s| {
                OffsetDateTime::from_unix_timestamp(s.seconds)
                    .inspect_err(|e| error!("failed to parse timestamp: {:?}", e))
                    .unwrap_or(OffsetDateTime::now_utc())
            })
        });

        let status = res.status();
        vpn_status::update(
            app,
            ConnectionState::from(status),
            res.error.map(BackendError::from),
            connection_time,
            status == ConnectionStatus::ConnectionFailed,
        )
        .await?;
        Ok(())
    }

    /// Watch VPN state updates
    #[instrument(skip_all)]
    pub async fn watch_vpn_state(&self, app: &AppHandle) -> Result<()> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(Empty {});
        let mut stream = vpnd
            .listen_to_connection_state_changes(request)
            .await
            .inspect_err(|e| {
                error!("listen_to_connection_state_changes failed: {}", e);
            })?
            .into_inner();

        let (tx, mut rx) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(update)) => {
                        tx.send(update).await.unwrap();
                    }
                    Ok(None) => {
                        warn!("watch vpn state stream closed by the server");
                        return;
                    }
                    Err(e) => {
                        warn!("watch vpn state stream get a grpc error: {}", e);
                    }
                }
            }
        });

        while let Some(state) = rx.recv().await {
            debug!("vpn state update {:?}", state.status());
            if let Some(e) = state.error.as_ref() {
                warn!("vpn status error: {}", e.message);
            }
            let status = state.status();
            vpn_status::update(
                app,
                ConnectionState::from(state.status()),
                state.error.map(BackendError::from),
                None,
                status == ConnectionStatus::ConnectionFailed,
            )
            .await?;
        }

        Ok(())
    }

    /// Watch VPN connection status updates
    #[instrument(skip_all)]
    pub async fn watch_vpn_connection_updates(&self, app: &AppHandle) -> Result<()> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(Empty {});
        let mut stream = vpnd
            .listen_to_connection_status(request)
            .await
            .inspect_err(|e| {
                error!("listen_to_connection_status failed: {}", e);
            })?
            .into_inner();

        let (tx, mut rx) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(update)) => {
                        tx.send(update).await.unwrap();
                    }
                    Ok(None) => {
                        warn!("watch vpn connection status stream closed by the server");
                        return;
                    }
                    Err(e) => {
                        warn!("watch vpn connection status stream get a grpc error: {}", e);
                    }
                }
            }
        });

        while let Some(update) = rx.recv().await {
            vpn_status::connection_update(app, update).await?;
        }

        Ok(())
    }

    /// Connect to the VPN
    #[instrument(skip_all)]
    pub async fn vpn_connect(
        &self,
        entry_node: EntryNode,
        exit_node: ExitNode,
        two_hop_mod: bool,
        dns: Option<Dns>,
    ) -> Result<bool, VpndError> {
        debug!("vpn_connect");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ConnectRequest {
            entry: Some(entry_node),
            exit: Some(exit_node),
            disable_routing: false,
            enable_two_hop: two_hop_mod,
            enable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: false,
            dns,
            min_mixnode_performance: None,
        });
        let response = vpnd.vpn_connect(request).await.map_err(|e| {
            error!("grpc vpn_connect: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().success)
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<bool, VpndError> {
        debug!("vpn_disconnect");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(DisconnectRequest {});
        let response = vpnd.vpn_disconnect(request).await.map_err(|e| {
            error!("grpc vpn_disconnect: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().success)
    }

    /// Import user credential from base58 encoded string
    #[instrument(skip_all)]
    pub async fn import_credential(
        &self,
        credential: Vec<u8>,
    ) -> Result<ImportUserCredentialResponse, VpndError> {
        debug!("import_credential");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ImportUserCredentialRequest { credential });
        let response = vpnd.import_user_credential(request).await.map_err(|e| {
            error!("grpc import_user_credential: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner())
    }

    /// Get the list of available countries for entry gateways
    #[instrument(skip_all)]
    pub async fn entry_countries(&self) -> Result<Vec<Country>, VpndError> {
        debug!("entry_countries");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ListEntryGatewaysRequest {});
        let response = vpnd.list_entry_gateways(request).await.map_err(|e| {
            error!("grpc entry_gateways: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("gateways count: {}", response.get_ref().gateways.len());

        let countries = response
            .get_ref()
            .gateways
            .iter()
            .filter_map(|gateway| {
                // TODO comment this for now has daemon is not returning any probe data
                // let check = gateway
                //     .last_probe
                //     .as_ref()
                //     .and_then(|probe| probe.outcome.as_ref())
                //     .and_then(|probe| probe.as_entry.as_ref())
                //     .map(|v| {
                //         if v.can_connect && v.can_route {
                //             return true;
                //         }
                //         false
                //     })
                //     .unwrap_or(false);
                // if !check {
                //     return None;
                // }

                Country::try_from(gateway).ok()
            })
            .dedup()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect();

        Ok(countries)
    }

    /// Get the list of available countries for exit gateways
    #[instrument(skip_all)]
    pub async fn exit_countries(&self) -> Result<Vec<Country>, VpndError> {
        debug!("exit_countries");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ListExitGatewaysRequest {});
        let response = vpnd.list_exit_gateways(request).await.map_err(|e| {
            error!("grpc entry_gateways: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("gateways count: {}", response.get_ref().gateways.len());

        let countries = response
            .get_ref()
            .gateways
            .iter()
            .filter_map(|gateway| {
                // TODO add probe data check
                Country::try_from(gateway).ok()
            })
            .dedup()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect();

        Ok(countries)
    }

    /// Watch the connection with the grpc server
    #[instrument(skip_all)]
    pub async fn watch(&self, app: &AppHandle) -> Result<()> {
        let mut health = self.health().await?;
        let app_state = app.state::<SharedAppState>();

        let request = Request::new(HealthCheckRequest {
            service: VPND_SERVICE.into(),
        });
        let mut stream = health
            .watch(request)
            .await
            .inspect_err(|e| {
                error!("health check failed: {}", e);
            })?
            .into_inner();

        let (tx, mut rx) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(res)) => {
                        tx.send(res.status()).await.unwrap();
                    }
                    Ok(None) => {
                        warn!("watch health stream closed by the server");
                        tx.send(ServingStatus::NotServing).await.unwrap();
                        return;
                    }
                    Err(e) => {
                        warn!("watch health stream get a grpc error: {}", e);
                    }
                }
            }
        });

        while let Some(status) = rx.recv().await {
            debug!("health check status: {:?}", status);
            app.emit_vpnd_status(status.into());
            let mut state = app_state.lock().await;
            state.vpnd_status = status.into();
        }

        Ok(())
    }
}

impl From<ServingStatus> for VpndStatus {
    fn from(status: ServingStatus) -> Self {
        match status {
            ServingStatus::Serving => VpndStatus::Ok,
            _ => VpndStatus::NotOk,
        }
    }
}

async fn get_channel(socket_path: PathBuf) -> anyhow::Result<Channel> {
    // NOTE the uri here is ignored
    Ok(TonicEndpoint::from_static(DEFAULT_HTTP_ENDPOINT)
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path.clone())
        }))
        .await?)
}

impl Default for Transport {
    fn default() -> Self {
        Transport::Ipc(DEFAULT_SOCKET_PATH.into())
    }
}

impl From<(&AppConfig, &Cli)> for Transport {
    fn from((config, cli): (&AppConfig, &Cli)) -> Self {
        let http_mode = if cli.grpc_http_mode {
            true
        } else {
            config.grpc_http_mode.unwrap_or(false)
        };
        if http_mode {
            Transport::Http(
                cli.grpc_http_endpoint.clone().unwrap_or(
                    config
                        .grpc_http_endpoint
                        .clone()
                        .unwrap_or(DEFAULT_HTTP_ENDPOINT.into()),
                ),
            )
        } else {
            Transport::Ipc(
                cli.grpc_socket_endpoint.clone().unwrap_or(
                    config
                        .grpc_socket_endpoint
                        .clone()
                        .unwrap_or(DEFAULT_SOCKET_PATH.into()),
                ),
            )
        }
    }
}

fn location_to_country(location: &Option<Location>) -> Result<Country> {
    if let Some(location) = location {
        Country::try_new_from_code(&location.two_letter_iso_country_code).ok_or_else(|| {
            warn!(
                "invalid country code {}",
                location.two_letter_iso_country_code
            );
            anyhow!(
                "invalid country code {}",
                location.two_letter_iso_country_code
            )
        })
    } else {
        Err(anyhow!("no location found"))
    }
}

impl TryFrom<&EntryGateway> for Country {
    type Error = anyhow::Error;

    fn try_from(gateway: &EntryGateway) -> Result<Country, Self::Error> {
        location_to_country(&gateway.location)
    }
}

impl TryFrom<&ExitGateway> for Country {
    type Error = anyhow::Error;

    fn try_from(gateway: &ExitGateway) -> Result<Country, Self::Error> {
        location_to_country(&gateway.location)
    }
}
