use std::env::consts::{ARCH, OS};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use nym_vpn_proto::{
    get_account_identity_response::Id as AccountIdRes,
    get_account_links_response::Res as AccountLinkRes,
    get_device_identity_response::Id as DeviceIdRes,
    is_account_stored_response::Resp as IsAccountStoredResp, nym_vpnd_client::NymVpndClient,
    tunnel_event::Event, ConnectRequest, Dns, GetAccountLinksRequest, ListGatewaysRequest,
    Location, SetNetworkRequest, StoreAccountRequest, UserAgent,
};
use tauri::{AppHandle, Manager, PackageInfo};
use tokio::sync::mpsc;
use tonic::transport::Endpoint as TonicEndpoint;
use tonic::{transport::Channel, Request};
use tracing::{debug, error, info, instrument, trace, warn};

pub use super::account_links::AccountLinks;
pub use super::error::VpndError;
use super::events::MixnetEvent;
pub use super::feature_flags::FeatureFlags;
use super::gateway::{Gateway, GatewayType};
pub use super::node::NodeConnect;
pub use super::system_message::SystemMessage;
use super::tunnel::TunnelState;
pub use super::vpnd_status::{VersionCheck, VpndInfo, VpndStatus};
pub use crate::grpc::network::NetworkCompatVersions;

use crate::cli::Cli;
use crate::country::Country;
use crate::error::BackendError;
use crate::fs::config::AppConfig;
use crate::{events::AppHandleEventEmitter, state::SharedAppState};

#[cfg(target_os = "linux")]
const DEFAULT_SOCKET_PATH: &str = "/run/nym-vpn.sock";
#[cfg(target_os = "macos")]
const DEFAULT_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";
#[cfg(windows)]
const DEFAULT_SOCKET_PATH: &str = r"\\.\pipe\nym-vpn";
const DUMMY_HTTP_ENDPOINT: &str = "http://[::1]:53181";

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
    pub async fn vpnd(&self) -> Result<NymVpndClient<Channel>, VpndError> {
        let channel = get_channel(self.socket.clone()).await.map_err(|e| {
            warn!("failed to connect to the daemon: {}", e);
            VpndError::FailedToConnectIpc(e)
        })?;
        Ok(NymVpndClient::new(channel))
    }

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&mut self) -> Result<VpndInfo, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(());
        let response = vpnd
            .info(request)
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

        let request = Request::new(());
        let response = vpnd
            .get_log_path(request)
            .await
            .map_err(|e| {
                error!("grpc: {}", e);
                VpndError::GrpcError(e)
            })?
            .into_inner();

        debug!("vpnd log path: {:?}", response);
        Ok(response.path)
    }

    /// Get the current tunnel state and update the app state
    #[instrument(skip_all)]
    pub async fn tunnel_state(&self, app: &AppHandle) -> Result<TunnelState, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(());
        let res = vpnd.get_tunnel_state(request).await?;
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

    /// Watch tunnel state updates and mixnet events
    #[instrument(skip_all)]
    pub async fn watch_tunnel_events(&self, app: &AppHandle) -> Result<()> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(());
        let mut stream = vpnd
            .listen_to_events(request)
            .await
            .inspect_err(|e| {
                error!("listen_to_tunnel_state_changes failed: {}", e);
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
                        warn!("listen tunnel state stream closed by the server");
                        return;
                    }
                    Err(e) => {
                        warn!("listen tunnel state stream get a grpc error: {}", e);
                    }
                }
            }
        });

        while let Some(state) = rx.recv().await {
            let Some(event) = state.event else {
                warn!("no event data, ignoring…");
                continue;
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
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_tunnel_update(
        app: &AppHandle,
        tun_state: nym_vpn_proto::TunnelState,
    ) -> Result<()> {
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

    /// Connect to the VPN
    #[instrument(skip_all)]
    pub async fn vpn_connect(
        &self,
        entry_node: NodeConnect,
        exit_node: NodeConnect,
        two_hop_mod: bool,
        credentials_mode: bool,
        netstack: bool,
        dns: Option<Dns>,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ConnectRequest {
            entry: Some(entry_node.into()),
            exit: Some(exit_node.into()),
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

        let request = Request::new(());
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

        let request = Request::new(());
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

        let request = Request::new(());
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

    /// Get the account identity \
    /// public key derived from the mnemonic
    #[instrument(skip_all)]
    pub async fn account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(());
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

        let request = Request::new(());
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

    /// Get the list of available gateways
    #[instrument(skip(self))]
    pub async fn gateways(&self, gw_type: GatewayType) -> Result<Vec<Gateway>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = Request::new(ListGatewaysRequest {
            kind: nym_vpn_proto::GatewayType::from(gw_type) as i32,
            user_agent: Some(self.user_agent.clone()),
            min_mixnet_performance: None,
            min_vpn_performance: None,
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

        let request = Request::new(());
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

        let request = Request::new(());
        let response = vpnd.get_feature_flags(request).await.map_err(|e| {
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

        let request = Request::new(());
        let response = vpnd.get_network_compatibility(request).await.map_err(|e| {
            error!("grpc: {}", e);
            VpndError::GrpcError(e)
        })?;
        debug!("grpc response: {:?}", response);
        response
            .into_inner()
            .messages
            .map(NetworkCompatVersions::from)
            .ok_or_else(|| {
                error!("no network compatibility data");
                VpndError::internal("no network compatibility data")
            })
    }
}

async fn get_channel(socket_path: PathBuf) -> Result<Channel> {
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
