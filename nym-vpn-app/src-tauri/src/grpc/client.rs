pub use super::{
    account_links::AccountLinks,
    error::VpndError,
    feature_flags::FeatureFlags,
    node::NodeConnect,
    system_message::SystemMessage,
    vpnd_status::{VersionCheck, VpndInfo, VpndStatus},
};
use super::{
    events::MixnetEvent,
    gateway::{Gateway, GatewayType},
    tunnel::TunnelState,
};

use anyhow::{Result, anyhow};
use nym_vpn_proto::proto::{
    AccountControllerState, ConnectRequest, Dns, GetAccountLinksRequest, ListGatewaysRequest,
    Location, StoreVpnAccountRequest, TunnelEvent, TunnelState as PTunnelState, UserAgent,
    nym_vpn_service_client::NymVpnServiceClient, tunnel_event::Event,
};
use once_cell::sync::Lazy;
use std::{
    env::consts::{ARCH, OS},
    path::PathBuf,
    sync::Mutex,
};
use tauri::{AppHandle, Manager, PackageInfo};
use tokio::sync::mpsc::{self, Sender};
use tonic::{
    Request, Streaming,
    transport::{Channel, Endpoint as TonicEndpoint},
};
use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::grpc::network::NetworkCompatVersions;
use crate::{
    cli::Cli,
    country::Country,
    error::BackendError,
    events::AppHandleEventEmitter,
    fs::config::AppConfig,
    grpc::account::{AccountState, log_account_state},
    state::SharedAppState,
};

#[cfg(target_os = "linux")]
const DEFAULT_SOCKET_PATH: &str = "/run/nym-vpn.sock";
#[cfg(target_os = "macos")]
const DEFAULT_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";
#[cfg(windows)]
const DEFAULT_SOCKET_PATH: &str = r"\\.\pipe\nym-vpn";
const DUMMY_HTTP_ENDPOINT: &str = "http://[::1]:53181";

// simple flag to save that "failed to connect to daemon"
// warning has been logged once when vpnd is down
static VPND_DOWN_LOGGED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[derive(Debug, Clone)]
pub struct GrpcClient {
    socket: PathBuf,
    pkg_info: PackageInfo,
    user_agent: UserAgent,
}

impl GrpcClient {
    #[instrument(skip_all)]
    pub fn new(config: &AppConfig, cli: &Cli, pkg: &PackageInfo) -> Self {
        GrpcClient {
            socket: cli.grpc_socket_endpoint.clone().unwrap_or(
                config
                    .grpc_socket_endpoint
                    .clone()
                    .unwrap_or(DEFAULT_SOCKET_PATH.into()),
            ),
            pkg_info: pkg.clone(),
            user_agent: GrpcClient::user_agent(pkg, None),
        }
    }

    /// Create a user agent
    pub fn user_agent(pkg: &PackageInfo, daemon_info: Option<&VpndInfo>) -> UserAgent {
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
    pub async fn vpnd(&self) -> Result<NymVpnServiceClient<Channel>, VpndError> {
        let channel = connect(self.socket.clone()).await.map_err(|e| {
            let mut logged = VPND_DOWN_LOGGED.lock().unwrap();
            if !*logged {
                warn!("failed to connect to the daemon: {}", e);
                *logged = true;
            } else {
                trace!("failed to connect to the daemon: {}", e);
            }
            VpndError::FailedToConnectIpc(e)
        })?;
        Ok(NymVpnServiceClient::new(channel))
    }

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&mut self) -> Result<VpndInfo, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd
            .info(())
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();

        let info = VpndInfo::from(&response);
        info!("vpnd UP");
        info!(
            "vpnd version: {}, network env: {}",
            info.version, info.network
        );
        self.user_agent = GrpcClient::user_agent(&self.pkg_info, Some(&info));
        info!("user agent: {:?}", self.user_agent);
        Ok(info)
    }

    /// Get daemon log path
    #[instrument(skip_all)]
    pub async fn vpnd_log_path(&self) -> Result<String, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd
            .get_log_path(())
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();

        debug!("vpnd log path: {:?}", response);
        Ok(response.dir)
    }

    /// Get the current tunnel state and update the app state
    #[instrument(skip_all)]
    pub async fn tunnel_state(&self, app: &AppHandle) -> Result<TunnelState, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let res = vpnd.get_tunnel_state(()).await?;
        let Some(tun_state) = res.into_inner().state else {
            error!("no tunnel state data");
            return Err(VpndError::internal("no tunnel state data"));
        };
        let tunnel = TunnelState::from_proto(tun_state).map_err(|e| {
            error!("failed to parse tunnel state: {}", e);
            VpndError::internal("failed to parse tunnel state")
        })?;
        info!("tunnel state [{}]", tunnel);
        if let TunnelState::Error(e) = &tunnel {
            warn!("tunnel error: {:?}", e);
        }
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_tunnel(app, tunnel.clone()).await?;

        Ok(tunnel)
    }

    /// Watch tunnel state and account state updates
    #[instrument(skip_all)]
    pub async fn watch_events(&self, app: &AppHandle) -> Result<()> {
        let mut vpnd = self.vpnd().await?;

        let tunnel_stream = vpnd
            .listen_to_events(())
            .await
            .inspect_err(|e| {
                error!("listen to events failed: {}", e);
            })?
            .into_inner();

        let (tx, mut rx) = mpsc::channel(32);
        GrpcClient::listen_to_stream(tunnel_stream, tx).await;

        while let Some(event) = rx.recv().await {
            self.handle_event(app, event).await.ok();
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_event(&self, app: &AppHandle, event: TunnelEvent) -> Result<()> {
        let Some(event) = event.event else {
            warn!("no event data, ignoring…");
            return Ok(());
        };
        match event {
            Event::TunnelState(state) => {
                debug!("tunnel state event {:?}", state);
                GrpcClient::handle_tunnel_update(app, state).await.ok();
            }
            Event::MixnetEvent(event) => {
                if let Some(e) = MixnetEvent::from_proto(event) {
                    trace!("mixnet event [{}]", e.as_ref());
                    app.emit_mixnet_event(e);
                } else {
                    warn!("failed to parse mixnet event");
                }
            }
            Event::AccountState(account_state) => {
                GrpcClient::handle_account_update(app, account_state)
                    .await
                    .ok();
            }
            _ => warn!("ignoring unknown tunnel event: {event:?}"),
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_tunnel_update(app: &AppHandle, tun_state: PTunnelState) -> Result<()> {
        if let Some(s) = tun_state.state {
            let tunnel = TunnelState::from_proto(s).map_err(|e| {
                error!("failed to parse tunnel state: {}", e);
                VpndError::internal("failed to parse tunnel state")
            })?;
            info!("tunnel state [{}]", tunnel);
            if let TunnelState::Error(e) = &tunnel {
                warn!("tunnel error: {:?}", e);
            }
            let s_state = app.state::<SharedAppState>();
            let mut app_state = s_state.lock().await;
            app_state.update_tunnel(app, tunnel).await?;
        } else {
            // this should never happen, right?
            warn!("no tunnel state data, ignoring…");
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_account_update(app: &AppHandle, update: AccountControllerState) -> Result<()> {
        let Some(state) = update.state else {
            warn!("no state data, ignoring…");
            return Ok(());
        };
        log_account_state(&state);
        let account_state = AccountState::from_proto(state);
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_account_state(app, account_state).await?;
        Ok(())
    }

    /// Connect to the VPN
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn vpn_connect(
        &self,
        entry_node: NodeConnect,
        exit_node: NodeConnect,
        two_hop_mod: bool,
        credentials_mode: bool,
        netstack: bool,
        dns: Option<Dns>,
        disable_ipv6: bool,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ConnectRequest {
            entry: Some(entry_node.into()),
            exit: Some(exit_node.into()),
            enable_two_hop: two_hop_mod,
            enable_bridges: false,
            netstack,
            disable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: credentials_mode,
            dns,
            user_agent: Some(self.user_agent.clone()),
            disable_ipv6,
        });
        vpnd.connect_tunnel(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        Ok(())
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disconnect_tunnel(()).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;

        Ok(())
    }

    /// Store an account
    #[instrument(skip_all)]
    pub async fn store_account(&self, mnemonic: String) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(StoreVpnAccountRequest { mnemonic });
        let response = vpnd.store_account(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        if let Some(error) = response.error.map(BackendError::from) {
            return Err(VpndError::Response(error));
        }
        Ok(())
    }

    /// Removes everything related to the account, including the device identity,
    /// credential storage, mixnet keys, gateway registrations
    #[instrument(skip_all)]
    pub async fn forget_account(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd.forget_account(()).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        if let Some(error) = response.error.map(BackendError::from) {
            return Err(VpndError::Response(error));
        }
        Ok(())
    }

    /// Check if an account is stored
    #[instrument(skip_all)]
    pub async fn is_account_stored(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd.is_account_stored(()).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        let is_stored = response.into_inner();
        debug!("grpc response: {:?}", is_stored);
        Ok(is_stored)
    }

    /// Get the account identity \
    /// public key derived from the mnemonic
    #[instrument(skip_all)]
    pub async fn account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd
            .get_account_identity(())
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        Ok(response.account_identity)
    }

    /// Get the device identity
    #[instrument(skip_all)]
    pub async fn device_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd
            .get_device_identity(())
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();
        debug!("grpc response: {:?}", response);
        Ok(response.device_identity)
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
        debug!("grpc response: {:?}", response);
        Ok(response.into())
    }

    /// Get the list of available gateways
    #[instrument(skip(self))]
    pub async fn gateways(&self, gw_type: GatewayType) -> Result<Vec<Gateway>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ListGatewaysRequest {
            kind: nym_vpn_proto::proto::GatewayType::from(gw_type) as i32,
            user_agent: Some(self.user_agent.clone()),
        });
        let response = vpnd.list_gateways(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!(
            "proto gateways count: {}",
            response.get_ref().gateways.len()
        );

        let gateways: Vec<Gateway> = response
            .into_inner()
            .gateways
            .into_iter()
            .filter_map(|gateway| {
                Gateway::from_proto(gateway, gw_type)
                    .inspect_err(|e| warn!("failed to parse gateway from proto: {e}"))
                    .ok()
            })
            .collect();
        debug!("parsed gateway count: {}", gateways.len());

        Ok(gateways)
    }

    #[instrument(skip(self, app))]
    pub async fn update_vpnd_state(
        &mut self,
        info: VpndInfo,
        app: &AppHandle,
    ) -> Result<(), VpndError> {
        let net_compat = self.network_compat().await.ok();

        let app_state = app.state::<SharedAppState>();
        let mut state = app_state.lock().await;
        state.vpnd_info = Some(info.clone());
        state.set_vpnd_status(&info);
        state.set_network_compat(net_compat, &self.pkg_info.version, &info);
        app.emit_vpnd_status(state.vpnd_status.clone());
        Ok(())
    }

    /// Set the network environment of the daemon.
    /// ⚠ This requires to restart the daemon to take effect.
    #[instrument(skip(self))]
    pub async fn set_network(&self, network: &str) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_network(network.to_owned()).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;

        Ok(())
    }

    /// Get messages affecting the whole system, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn system_messages(&self) -> Result<Vec<SystemMessage>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd.get_system_messages(()).await.map_err(|e| {
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

        let response = vpnd.get_feature_flags(()).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        let response = response.into_inner();
        Ok(FeatureFlags::from(&response))
    }

    /// Get the network compatibility versions of supported vpn-core and tauri client
    #[instrument(skip_all)]
    pub async fn network_compat(&self) -> Result<NetworkCompatVersions, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd.get_network_compatibility(()).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        response
            .into_inner()
            .network_compatibility
            .map(NetworkCompatVersions::from)
            .ok_or_else(|| {
                error!("no network compatibility data");
                VpndError::internal("no network compatibility data")
            })
    }

    /// Is sentry enabled at daemon level
    #[instrument(skip_all)]
    pub async fn sentry_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let enabled = vpnd
            .is_sentry_enabled(())
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();

        debug!("vpnd sentry enabled: {}", enabled);
        if enabled {
            info!("⚠ vpnd sentry monitoring is enabled ⚠");
        }
        Ok(enabled)
    }

    /// Enable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn enable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.enable_sentry(()).await.map_err(|e| {
            error!("failed to enable sentry: {}", e);
            VpndError::GrpcError(e)
        })?;

        debug!("enabled vpnd sentry");
        info!("restart vpnd (service) required for the change to take effect");
        Ok(())
    }

    /// Disable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn disable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_sentry(()).await.map_err(|e| {
            error!("failed to disable sentry: {}", e);
            VpndError::GrpcError(e)
        })?;

        debug!("disabled vpnd sentry");
        info!("restart vpnd (service) recommended");
        Ok(())
    }

    /// Is network statistics collection enabled at daemon level
    #[instrument(skip_all)]
    pub async fn netstats_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let enabled = vpnd
            .is_collect_network_stats_enabled(())
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();

        debug!("vpnd network statistics collection enabled: {}", enabled);
        if enabled {
            info!("⚠ vpnd network statistics collection enabled ⚠");
        }
        Ok(enabled)
    }

    /// Enable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn enable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.enable_collect_network_stats(()).await.map_err(|e| {
            error!("failed to enable network statistics collection: {}", e);
            VpndError::GrpcError(e)
        })?;

        debug!("enabled vpnd network statistics collection");
        info!("restart vpnd (service) required for the change to take effect");
        Ok(())
    }

    /// Disable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn disable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_collect_network_stats(()).await.map_err(|e| {
            error!("failed to disable network statistics collection: {}", e);
            VpndError::GrpcError(e)
        })?;

        debug!("disabled vpnd network statistics collection");
        info!("restart vpnd (service) required for the change to take effect");
        Ok(())
    }

    async fn listen_to_stream<S>(mut stream: Streaming<S>, tx: Sender<TunnelEvent>)
    where
        S: Into<TunnelEvent> + Send + 'static,
    {
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(message)) => {
                        tx.send(message.into()).await.unwrap();
                    }
                    Ok(None) => {
                        let mut logged = VPND_DOWN_LOGGED.lock().unwrap();
                        if !*logged {
                            warn!("vpnd DOWN: stream closed");
                            *logged = true;
                        }
                        return;
                    }
                    Err(e) => {
                        warn!("grpc error: {}", e);
                    }
                }
            }
        });
    }

    pub fn reset_log_flag() {
        let mut logged = VPND_DOWN_LOGGED.lock().unwrap();
        *logged = false;
    }
}

async fn connect(socket_path: PathBuf) -> Result<Channel> {
    // NOTE the uri here is ignored
    Ok(TonicEndpoint::from_static(DUMMY_HTTP_ENDPOINT)
        .connect_with_connector(tower::service_fn(move |_| {
            nym_ipc::client::connect(socket_path.clone())
        }))
        .await?)
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
