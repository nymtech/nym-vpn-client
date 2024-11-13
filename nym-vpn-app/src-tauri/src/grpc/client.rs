use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use nym_vpn_proto::{
    health_check_response::ServingStatus, health_client::HealthClient,
    is_account_stored_response::Resp as IsAccountStoredResp, nym_vpnd_client::NymVpndClient,
    ConnectRequest, ConnectionStatus, DisconnectRequest, Dns, Empty, EntryNode, ExitNode,
    FetchRawAccountSummaryRequest, GatewayType, GetSystemMessagesRequest, HealthCheckRequest,
    InfoRequest, InfoResponse, IsAccountStoredRequest, ListCountriesRequest, Location,
    RemoveAccountRequest, SetNetworkRequest, StatusRequest, StatusResponse, StoreAccountRequest,
    UserAgent,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tauri::{AppHandle, Manager, PackageInfo};
use thiserror::Error;
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tonic::transport::Endpoint as TonicEndpoint;
use tonic::{transport::Channel, Request};
use tracing::{debug, error, info, instrument, warn};

use crate::cli::Cli;
use crate::country::Country;
use crate::error::BackendError;
use crate::fs::config::AppConfig;
pub use crate::grpc::system_message::SystemMessage;
pub use crate::grpc::vpnd_status::VpndStatus;
use crate::states::app::ConnectionState;
use crate::vpn_status;
use crate::{events::AppHandleEventEmitter, states::SharedAppState};

const VPND_SERVICE: &str = "nym.vpn.NymVpnd";
#[cfg(target_os = "linux")]
const DEFAULT_SOCKET_PATH: &str = "/run/nym-vpn.sock";
#[cfg(target_os = "macos")]
const DEFAULT_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";
#[cfg(windows)]
const DEFAULT_SOCKET_PATH: &str = r"\\.\pipe\nym-vpn";
const DEFAULT_HTTP_ENDPOINT: &str = "http://[::1]:53181";

#[derive(Clone, Debug)]
enum Transport {
    Http(String),
    Ipc(PathBuf),
}

#[derive(Error, Debug)]
pub enum VpndError {
    #[error("gRPC call error")]
    GrpcError(#[from] tonic::Status),
    #[error("failed to connect to daemon using HTTP transport")]
    FailedToConnectHttp(#[from] tonic::transport::Error),
    #[error("failed to connect to daemon using IPC transport")]
    FailedToConnectIpc(#[from] anyhow::Error),
    #[error("call response error {0}")]
    Response(#[from] BackendError),
}

#[derive(Debug, Default, Clone)]
pub struct GrpcClient {
    transport: Transport,
    user_agent: UserAgent,
}

impl GrpcClient {
    #[instrument(skip_all)]
    pub fn new(config: &AppConfig, cli: &Cli, pkg: &PackageInfo) -> Self {
        let client = GrpcClient {
            transport: Transport::from((config, cli)),
            user_agent: GrpcClient::user_agent(pkg, None),
        };
        match &client.transport {
            Transport::Http(endpoint) => {
                info!("using grpc HTTP transport: {}", endpoint);
            }
            Transport::Ipc(socket) => {
                info!("using grpc IPC transport: {}", socket.display());
            }
        }
        client
    }

    /// Create a user agent
    pub fn user_agent(pkg: &PackageInfo, daemon_info: Option<&InfoResponse>) -> UserAgent {
        let app_git_commit = crate::build_info()
            .version_control
            .as_ref()
            .and_then(|vc| vc.git())
            .map(|g| g.commit_short_id.clone())
            .unwrap_or_default();

        UserAgent {
            application: pkg.name.clone(),
            version: daemon_info.map_or_else(
                || pkg.version.to_string(),
                |info| format!("{} ({})", pkg.version, info.version),
            ),
            platform: format!("{}; {}; {}", OS, tauri_plugin_os::version(), ARCH),
            git_commit: daemon_info.map_or_else(
                || app_git_commit.clone(),
                |info| format!("{} ({})", app_git_commit, info.git_commit),
            ),
        }
    }

    /// Get the Vpnd service client
    #[instrument(skip_all)]
    pub async fn vpnd(&self) -> Result<NymVpndClient<Channel>, VpndError> {
        match &self.transport {
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
        match &self.transport {
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

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&self) -> Result<InfoResponse, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(InfoRequest {});
        let response = vpnd.info(request).await.map_err(|e| {
            error!("grpc info: {}", e);
            VpndError::GrpcError(e)
        })?;

        Ok(response.into_inner())
    }

    /// Update `user_agent` with the daemon info
    // TODO this is dirty, this logic shouldn't be handled in the client side
    #[instrument(skip_all)]
    pub async fn update_agent(&mut self, pkg: &PackageInfo) -> Result<(), VpndError> {
        let d_info = self.vpnd_info().await?;
        self.user_agent = GrpcClient::user_agent(pkg, Some(&d_info));
        info!("vpnd version: {}", d_info.version);
        info!(
            "network env: {}",
            d_info
                .nym_network
                .map(|n| n.network_name)
                .unwrap_or_else(|| "unknown".to_string())
        );
        info!("updated user agent: {:?}", self.user_agent);
        Ok(())
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
        netstack: bool,
        dns: Option<Dns>,
    ) -> Result<(), VpndError> {
        debug!("vpn_connect");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ConnectRequest {
            entry: Some(entry_node),
            exit: Some(exit_node),
            disable_routing: false,
            enable_two_hop: two_hop_mod,
            netstack,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: false,
            dns,
            user_agent: Some(self.user_agent.clone()),
            min_mixnode_performance: None,
            min_gateway_mixnet_performance: None,
            min_gateway_vpn_performance: None,
        });
        let response = vpnd
            .vpn_connect(request)
            .await
            .map_err(|e| {
                error!("grpc vpn_connect: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        if response.success {
            return Ok(());
        }
        Err(VpndError::Response(
            response
                .error
                .inspect(|e| error!("vpn connect error: {:?}", e))
                .map(BackendError::from)
                .ok_or_else(|| {
                    error!("connect bad response: no ConnectRequestError");
                    VpndError::GrpcError(tonic::Status::internal(
                        "connect bad response: no ConnectRequestError".to_string(),
                    ))
                })?,
        ))
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

    /// Store an account
    #[instrument(skip_all)]
    pub async fn store_account(&self, mnemonic: String) -> Result<(), VpndError> {
        debug!("store_account");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(StoreAccountRequest { mnemonic, nonce: 0 });
        let response = vpnd.store_account(request).await.map_err(|e| {
            error!("grpc store_account: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        if response.success {
            return Ok(());
        }
        Err(VpndError::Response(
            response
                .error
                .inspect(|e| warn!("store account error: {:?}", e))
                .map(BackendError::from)
                .ok_or_else(|| {
                    error!("store account bad response: no AccountError");
                    VpndError::GrpcError(tonic::Status::internal(
                        "store account bad response: no AccountError".to_string(),
                    ))
                })?,
        ))
    }

    /// Remove the stored account
    #[instrument(skip_all)]
    pub async fn remove_account(&self) -> Result<(), VpndError> {
        debug!("remove_account");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(RemoveAccountRequest {});
        let response = vpnd.remove_account(request).await.map_err(|e| {
            error!("grpc remove_account: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        if response.success {
            return Ok(());
        }
        Err(VpndError::Response(
            response
                .error
                .inspect(|e| warn!("remove account error: {:?}", e))
                .map(BackendError::from)
                .ok_or_else(|| {
                    error!("remove account bad response: no AccountError");
                    VpndError::GrpcError(tonic::Status::internal(
                        "remove account bad response: no AccountError".to_string(),
                    ))
                })?,
        ))
    }

    /// Check if an account is stored
    #[instrument(skip_all)]
    pub async fn is_account_stored(&self) -> Result<bool, VpndError> {
        debug!("is_account_stored");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(IsAccountStoredRequest {});
        let response = vpnd.is_account_stored(request).await.map_err(|e| {
            error!("grpc is_account_stored: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        match response.resp.ok_or_else(|| {
            error!("failed to get stored account: invalid response");
            VpndError::GrpcError(tonic::Status::internal(
                "failed to get stored account: invalid response",
            ))
        })? {
            IsAccountStoredResp::IsStored(v) => Ok(v),
            IsAccountStoredResp::Error(e) => Err(VpndError::Response(e.into())),
        }
    }

    /// Get account info
    /// Note: if no account is stored yet, the call will fail
    #[instrument(skip_all)]
    pub async fn get_account_summary(&self) -> Result<String, VpndError> {
        debug!("get_account_summary");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(FetchRawAccountSummaryRequest {});
        let response = vpnd
            .fetch_raw_account_summary(request)
            .await
            .map_err(|e| {
                error!("grpc get_account_summary: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        if let Some(error) = response.error {
            error!("get account summary error: {:?}", error);
            return Err(VpndError::Response(error.into()));
        }

        Ok(response.json)
    }

    /// Get the list of available countries for entry gateways
    #[instrument(skip(self))]
    pub async fn countries(&self, gw_type: GatewayType) -> Result<Vec<Country>, VpndError> {
        debug!("countries");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ListCountriesRequest {
            kind: gw_type as i32,
            user_agent: Some(self.user_agent.clone()),
            min_mixnet_performance: None,
            min_vpn_performance: None,
        });
        let response = vpnd.list_countries(request).await.map_err(|e| {
            error!("grpc list_countries: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("countries count: {}", response.get_ref().countries.len());

        let countries: Vec<Country> = response
            .get_ref()
            .countries
            .iter()
            .filter_map(|location| Country::try_from(location).ok())
            .unique()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect();
        debug!("filtered countries count: {}", countries.len());

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

    /// Set the network environment of the daemon.
    /// ⚠ This requires to restart the daemon to take effect.
    #[instrument(skip(self))]
    pub async fn set_network(&self, network: &str) -> Result<(), VpndError> {
        debug!("set_network");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(SetNetworkRequest {
            network: network.to_owned(),
        });
        let response = vpnd
            .set_network(request)
            .await
            .map_err(|e| {
                error!("grpc set_network: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        if let Some(e) = response.error {
            error!("set network env error: {:?}", e);
            return Err(VpndError::Response(e.into()));
        }
        Ok(())
    }

    /// List messages affecting the whole system, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn system_messages(&self) -> Result<Vec<SystemMessage>, VpndError> {
        debug!("system_messages");
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(GetSystemMessagesRequest {});
        let response = vpnd.get_system_messages(request).await.map_err(|e| {
            error!("grpc system_messages: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        Ok(response.messages.iter().map(Into::into).collect())
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

impl TryFrom<&Location> for Country {
    type Error = anyhow::Error;

    fn try_from(location: &Location) -> Result<Country, Self::Error> {
        Country::try_new_from_code(&location.two_letter_iso_country_code).ok_or_else(|| {
            let msg = format!(
                "invalid country code {}",
                location.two_letter_iso_country_code
            );
            warn!(msg);
            anyhow!(msg)
        })
    }
}
