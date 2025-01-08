use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use nym_vpn_proto::{
    get_account_identity_response::Id as AccountIdRes,
    get_account_links_response::Res as AccountLinkRes,
    get_device_identity_response::Id as DeviceIdRes, health_check_response::ServingStatus,
    health_client::HealthClient, is_account_stored_response::Resp as IsAccountStoredResp,
    nym_vpnd_client::NymVpndClient, ConnectRequest, ConnectionStatus, DisconnectRequest, Dns,
    Empty, EntryNode, ExitNode, ForgetAccountRequest, GatewayType, GetAccountIdentityRequest,
    GetAccountLinksRequest, GetDeviceIdentityRequest, GetFeatureFlagsRequest,
    GetSystemMessagesRequest, HealthCheckRequest, InfoRequest, InfoResponse,
    IsAccountStoredRequest, IsReadyToConnectRequest, ListCountriesRequest, Location,
    SetNetworkRequest, StatusRequest, StatusResponse, StoreAccountRequest, UserAgent,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tauri::{AppHandle, Manager, PackageInfo};
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tonic::transport::Endpoint as TonicEndpoint;
use tonic::{transport::Channel, Request};
use tracing::{debug, error, info, instrument, warn};

pub use super::account_links::AccountLinks;
pub use super::error::VpndError;
pub use super::feature_flags::FeatureFlags;
pub use super::ready_to_connect::ReadyToConnect;
pub use super::system_message::SystemMessage;
use super::version_check::VersionCheck;
pub use super::vpnd_status::{VpndInfo, VpndStatus};
use crate::cli::Cli;
use crate::country::Country;
use crate::env::VPND_COMPAT_REQ;
use crate::error::BackendError;
use crate::fs::config::AppConfig;
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

#[derive(Debug, Clone)]
pub struct GrpcClient {
    transport: Transport,
    pkg_info: PackageInfo,
    user_agent: UserAgent,
}

impl GrpcClient {
    #[instrument(skip_all)]
    pub fn new(config: &AppConfig, cli: &Cli, pkg: &PackageInfo) -> Self {
        let client = GrpcClient {
            transport: Transport::from((config, cli)),
            pkg_info: pkg.clone(),
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
        let serving = response.status();

        let mut state = app_state.lock().await;
        let status = self.get_vpnd_status(serving, state.vpnd_info.as_ref());
        state.vpnd_status = status.clone();

        Ok(status)
    }

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&self, app: &AppHandle) -> Result<InfoResponse, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(InfoRequest {});
        let response = vpnd
            .info(request)
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();

        let app_state = app.state::<SharedAppState>();
        let mut state = app_state.lock().await;
        state.vpnd_info = Some(VpndInfo::from(&response));
        Ok(response)
    }

    /// Update daemon info and user agent
    #[instrument(skip_all)]
    pub async fn update_vpnd_info(&mut self, app: &AppHandle) -> Result<VpndInfo, VpndError> {
        let res = self.vpnd_info(app).await?;
        let vpnd_info = VpndInfo::from(&res);
        self.user_agent = GrpcClient::user_agent(&self.pkg_info, Some(&res));
        info!("vpnd version: {}", res.version);
        info!(
            "network env: {}",
            res.nym_network
                .map(|n| n.network_name)
                .unwrap_or_else(|| "unknown".to_string())
        );
        info!("updated user agent: {:?}", self.user_agent);

        Ok(vpnd_info)
    }

    /// Get VPN status
    #[instrument(skip_all)]
    pub async fn vpn_status(&self) -> Result<StatusResponse, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(StatusRequest {});
        let response = vpnd.vpn_status(request).await.map_err(|e| {
            error!("grpc: {}", e);
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
        credentials_mode: bool,
        netstack: bool,
        dns: Option<Dns>,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        if credentials_mode {
            info!("credentials mode enabled");
        }
        let request = Request::new(ConnectRequest {
            entry: Some(entry_node),
            exit: Some(exit_node),
            disable_routing: false,
            enable_two_hop: two_hop_mod,
            netstack,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: credentials_mode,
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
                error!("grpc: {}", e);
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
                    VpndError::internal("connect bad response: no ConnectRequestError")
                })?,
        ))
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(DisconnectRequest {});
        let response = vpnd.vpn_disconnect(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().success)
    }

    /// Store an account
    #[instrument(skip_all)]
    pub async fn store_account(&self, mnemonic: String) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(StoreAccountRequest { mnemonic, nonce: 0 });
        let response = vpnd.store_account(request).await.map_err(|e| {
            error!("grpc: {}", e);
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
                    VpndError::internal("store account bad response: no AccountError")
                })?,
        ))
    }

    /// Removes everything related to the account, including the device identity,
    /// credential storage, mixnet keys, gateway registrations
    #[instrument(skip_all)]
    pub async fn forget_account(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ForgetAccountRequest {});
        let response = vpnd.forget_account(request).await.map_err(|e| {
            error!("grpc: {}", e);
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
                .inspect(|e| warn!("forget account error: {:?}", e))
                .map(BackendError::from)
                .ok_or_else(|| {
                    error!("forget account bad response: no AccountError");
                    VpndError::internal("forget account bad response: no AccountError")
                })?,
        ))
    }

    /// Check if an account is stored
    #[instrument(skip_all)]
    pub async fn is_account_stored(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(IsAccountStoredRequest {});
        let response = vpnd.is_account_stored(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        match response.resp.ok_or_else(|| {
            error!("failed to get stored account: invalid response");
            VpndError::internal("failed to get stored account: invalid response")
        })? {
            IsAccountStoredResp::IsStored(v) => Ok(v),
            IsAccountStoredResp::Error(e) => Err(VpndError::Response(e.into())),
        }
    }

    /// Check the local account state and device info, if it is ready to connect
    #[instrument(skip_all)]
    pub async fn is_ready_to_connect(&self) -> Result<ReadyToConnect, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(IsReadyToConnectRequest {});
        let response = vpnd.is_ready_to_connect(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        let response = response.into_inner();
        debug!("grpc response: {:?}", response);
        response.kind().try_into().map_err(|e: String| {
            error!("{e}");
            VpndError::internal(&e)
        })
    }

    /// Get the account identity \
    /// public key derived from the mnemonic
    #[instrument(skip_all)]
    pub async fn account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(GetAccountIdentityRequest {});
        let response = vpnd
            .get_account_identity(request)
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        match response.id.ok_or_else(|| {
            error!("failed to get account id: invalid response");
            VpndError::internal("failed to get account id: invalid response")
        })? {
            AccountIdRes::AccountIdentity(id) => Ok(id.account_identity),
            AccountIdRes::Error(e) => Err(VpndError::Response(e.into())),
        }
    }

    /// Get the device identity
    #[instrument(skip_all)]
    pub async fn device_id(&self) -> Result<String, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(GetDeviceIdentityRequest {});
        let response = vpnd
            .get_device_identity(request)
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        match response.id.ok_or_else(|| {
            error!("failed to get device id: invalid response");
            VpndError::internal("failed to get device id: invalid response")
        })? {
            DeviceIdRes::DeviceIdentity(id) => Ok(id),
            DeviceIdRes::Error(e) => Err(VpndError::Response(e.into())),
        }
    }

    /// Get the account links
    #[instrument(skip_all)]
    pub async fn account_links(&self, _locale: &str) -> Result<AccountLinks, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(GetAccountLinksRequest {
            // TODO use the locale set at app level once website is i18n ready
            locale: "en".to_string(),
        });
        let response = vpnd.get_account_links(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        let response = response.into_inner();
        debug!("grpc response: {:?}", response.res);
        match response.res.ok_or_else(|| {
            error!("failed to get account links: invalid response");
            VpndError::internal("failed to get account links: invalid response")
        })? {
            AccountLinkRes::Links(l) => Ok(l.into()),
            AccountLinkRes::Error(e) => Err(VpndError::Response(e.into())),
        }
    }

    /// Get the list of available countries for entry gateways
    #[instrument(skip(self))]
    pub async fn countries(&self, gw_type: GatewayType) -> Result<Vec<Country>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ListCountriesRequest {
            kind: gw_type as i32,
            user_agent: Some(self.user_agent.clone()),
            min_mixnet_performance: None,
            min_vpn_performance: None,
        });
        let response = vpnd.list_countries(request).await.map_err(|e| {
            error!("grpc: {}", e);
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
    pub async fn watch(&mut self, app: &AppHandle) -> Result<()> {
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

        while let Some(serving) = rx.recv().await {
            debug!("health check status: {:?}", serving);
            let mut vpnd_info = None;
            if serving == ServingStatus::Serving {
                vpnd_info = self.update_vpnd_info(app).await.ok();
            }
            let status = self.get_vpnd_status(serving, vpnd_info.as_ref());
            app.emit_vpnd_status(status.clone());
            let mut state = app_state.lock().await;
            state.vpnd_status = status;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    fn get_vpnd_status(&self, serving: ServingStatus, vpnd_info: Option<&VpndInfo>) -> VpndStatus {
        if serving != ServingStatus::Serving {
            return VpndStatus::NotOk;
        }

        let Some(ver_req) = VPND_COMPAT_REQ else {
            warn!("env variable `VPND_COMPAT_REQ` is not set, skipping vpnd version compatibility check");
            return VpndStatus::Ok(None);
        };
        let Some(info) = vpnd_info else {
            // very unlikely to happen
            error!("no vpnd info available, skipping vpnd version compatibility check");
            return VpndStatus::Ok(None);
        };
        let Ok(ver) = VersionCheck::new(ver_req) else {
            warn!("skipping vpnd version compatibility check");
            return VpndStatus::Ok(Some(info.to_owned()));
        };
        let Ok(is_ok) = ver.check(&info.version) else {
            warn!("skipping vpnd version compatibility check");
            return VpndStatus::Ok(Some(info.to_owned()));
        };

        if !is_ok {
            warn!(
                "daemon version is not compatible with the client, required [{}], version [{}]",
                ver_req, info.version
            );
            return VpndStatus::NonCompat {
                current: info.clone(),
                requirement: ver_req.to_string(),
            };
        }

        info!("daemon version compatibility check OK");
        VpndStatus::Ok(Some(info.to_owned()))
    }

    /// Set the network environment of the daemon.
    /// ⚠ This requires to restart the daemon to take effect.
    #[instrument(skip(self))]
    pub async fn set_network(&self, network: &str) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(SetNetworkRequest {
            network: network.to_owned(),
        });
        let response = vpnd
            .set_network(request)
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
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

    /// Get messages affecting the whole system, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn system_messages(&self) -> Result<Vec<SystemMessage>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(GetSystemMessagesRequest {});
        let response = vpnd.get_system_messages(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        Ok(response.messages.iter().map(Into::into).collect())
    }

    /// Get the feature flags, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn feature_flags(&self) -> Result<FeatureFlags, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(GetFeatureFlagsRequest {});
        let response = vpnd.get_feature_flags(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        Ok(FeatureFlags::from(&response))
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
