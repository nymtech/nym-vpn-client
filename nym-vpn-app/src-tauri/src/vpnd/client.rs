pub use super::{
    account::{AccountSummary, AutologinResponse, StoredAccountMode},
    account_links::AccountLinks,
    deeplink::DeeplinkKind,
    error::VpndError,
    feature_flags::FeatureFlags,
    node::Node,
    socks5::{HttpRpcSettings, Socks5Settings, Socks5Status},
    system_message::SystemMessage,
    vpnd_status::{VersionCheck, VpndInfo, VpndStatus},
};
use super::{
    config::{MixnetTrafficConfig, VpndConfig},
    events::{ConflictDetected, DiagnosticsSuggestedReason, MixnetEvent},
    gateway::{Gateway, GatewayType, RecentGateways, parse_gateways},
    tentative_gateways::TentativeGateways,
    tunnel::{FrontingMode, SplitApp, TunnelState},
};

use anyhow::Result;
use futures::future::TryFutureExt;
use lib::UserAgent;
use nym_vpn_lib_types::{self as lib, AccountCommandResponse};
use nym_vpn_proto::rpc_client::RpcClient;
use std::{
    env::consts::{ARCH, OS},
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
};
use tauri::{AppHandle, Manager, PackageInfo};
use tokio::sync::{Mutex, Notify};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::vpnd::network::NetworkCompatVersions;
use crate::{
    error::BackendError,
    events::AppHandleEventEmitter,
    state::{SharedAppState, app::VpnMode},
    vpnd::account::{AccountState, log_account_state},
};

#[derive(Debug, Clone)]
enum ConnectionState {
    Connected(RpcClient),
    Trying,
    AuthenticationDenied,
}

#[derive(Debug, Clone)]
pub struct VpndClient {
    rpc_client: Arc<Mutex<ConnectionState>>,
    connect_fail_logged: Arc<Mutex<bool>>,
    auth_retry_notify: Arc<Notify>,
    pkg_info: PackageInfo,
    user_agent: UserAgent,
}

impl VpndClient {
    #[instrument(skip_all)]
    pub fn new(pkg: &PackageInfo) -> Self {
        VpndClient {
            rpc_client: Arc::new(Mutex::new(ConnectionState::Trying)),
            connect_fail_logged: Arc::new(Mutex::new(false)),
            auth_retry_notify: Arc::new(Notify::new()),
            pkg_info: pkg.clone(),
            user_agent: VpndClient::user_agent(pkg, None),
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

    /// Get the rpc client
    #[instrument(skip_all)]
    pub async fn vpnd(&self) -> Result<RpcClient, VpndError> {
        let mut guard = self.rpc_client.lock().await;
        match &*guard {
            // fast path: already created
            ConnectionState::Connected(rpc_client) => return Ok(rpc_client.clone()),
            // tried before, but authentication didn't succeed
            ConnectionState::AuthenticationDenied => return Err(VpndError::AuthenticationRequired),
            // got signal to try again
            ConnectionState::Trying => {}
        }

        // slow path: create new client
        let client = match RpcClient::new().await {
            Ok(c) => {
                self.reset_log_connect_failed().await;
                c
            }
            Err(nym_vpn_proto::rpc_client::Error::AuthenticationRequired) => {
                self.log_connect_failed("need to authenticate").await;
                *guard = ConnectionState::AuthenticationDenied;
                return Err(VpndError::AuthenticationRequired);
            }
            Err(e) => {
                self.log_connect_failed("failed to connect to the daemon")
                    .await;
                return Err(VpndError::FailedToConnectIpc(e.into()));
            }
        };

        debug!("connected to the daemon");

        *guard = ConnectionState::Connected(client.clone());
        Ok(client)
    }

    pub async fn retry_daemon_authentication(&self) {
        let mut guard = self.rpc_client.lock().await;
        *guard = ConnectionState::Trying;
        drop(guard);
        self.auth_retry_notify.notify_one();
    }

    /// Wait until the frontend signals an authentication retry
    pub async fn wait_for_auth_retry(&self) {
        self.auth_retry_notify.notified().await;
    }

    async fn drop_rpc_client(&self) {
        let mut guard = self.rpc_client.lock().await;
        *guard = ConnectionState::Trying;
        debug!("dropped daemon connection");
    }

    async fn log_connect_failed(&self, message: &str) {
        let mut guard = self.connect_fail_logged.lock().await;
        if !*guard {
            warn!(message);
            *guard = true;
        }
    }

    async fn reset_log_connect_failed(&self) {
        let mut guard = self.connect_fail_logged.lock().await;
        if *guard {
            *guard = false;
        }
    }

    async fn handle_rpc_error<T>(
        &self,
        func: &str,
        e: nym_vpn_proto::rpc_client::Error,
    ) -> Result<T, VpndError> {
        if matches!(
            e,
            nym_vpn_proto::rpc_client::Error::Rpc(_)
                | nym_vpn_proto::rpc_client::Error::Transport(_)
        ) {
            self.drop_rpc_client().await;
        }
        error!("vpnd.{func}() failed: {e}");
        Err(VpndError::RpcClient(e))
    }

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&mut self) -> Result<VpndInfo, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let info: VpndInfo = vpnd
            .get_info()
            .or_else(async |e| self.handle_rpc_error("get_info", e).await)
            .await?
            .into();

        info!(
            "vpnd version: {}, network env: {}",
            info.version, info.network
        );
        self.user_agent = VpndClient::user_agent(&self.pkg_info, Some(&info));
        info!("user agent: {:?}", self.user_agent);
        Ok(info)
    }

    /// Get daemon log path
    #[instrument(skip_all)]
    pub async fn vpnd_log_path(&self) -> Result<Option<PathBuf>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let log_path = vpnd
            .get_log_path()
            .or_else(async |e| self.handle_rpc_error("get_log_path", e).await)
            .await?;

        debug!("vpnd log path: {:?}", log_path);
        Ok(log_path.map(|v| v.dir))
    }

    /// Delete logs
    #[instrument(skip_all)]
    pub async fn delete_logs(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.delete_log_file()
            .or_else(async |e| self.handle_rpc_error("delete_logs", e).await)
            .await?;

        Ok(())
    }

    /// Get the current tunnel state and update the app state
    #[instrument(skip_all)]
    pub async fn tunnel_state(&self, app: &AppHandle) -> Result<TunnelState, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let tun_state = vpnd
            .get_tunnel_state()
            .or_else(async |e| self.handle_rpc_error("get_tunnel_state", e).await)
            .await?;

        let tunnel = TunnelState::from_lib(tun_state);
        info!("tunnel state [{}]", tunnel);
        if let TunnelState::Error(e) = &tunnel {
            warn!("tunnel error: {:?}", e);
        }
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_tunnel(app, tunnel.clone()).await?;

        Ok(tunnel)
    }

    /// Get the current daemon configuration and update the app state
    #[instrument(skip_all)]
    pub async fn update_config(&self, app: &AppHandle) -> Result<(), VpndError> {
        let config = self.config().await?;
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_vpnd_config(app, config).await?;

        Ok(())
    }

    /// Watch tunnel state, account state and vpn config updates
    #[instrument(skip_all)]
    pub async fn watch_events(&mut self, app: &AppHandle) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let mut stream = vpnd
            .listen_to_events()
            .or_else(async |e| self.handle_rpc_error("listen_to_events", e).await)
            .await?;

        trace!("started listening to tunnel events");

        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    trace!("received tunnel event: {:?}", event);
                    self.handle_event(app, event).await.ok();
                }
                Err(e) => warn!("event stream error: {:?}", e),
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_event(&self, app: &AppHandle, event: lib::TunnelEvent) -> Result<()> {
        match event {
            lib::TunnelEvent::NewState(state) => {
                debug!("tunnel state event {:?}", state);
                VpndClient::handle_tunnel_update(app, state).await.ok();
            }
            lib::TunnelEvent::MixnetState(e) => {
                let event = MixnetEvent::from_lib(e);
                trace!("mixnet event [{}]", event.as_ref());
                app.emit_mixnet_event(event);
            }
            lib::TunnelEvent::AccountState(account_state) => {
                VpndClient::handle_account_update(app, account_state)
                    .await
                    .ok();
            }
            lib::TunnelEvent::ConfigChanged(e) => {
                debug!("config event {e}");
                VpndClient::handle_config_update(app, *e).await.ok();
            }
            lib::TunnelEvent::DiagnosticsSuggested(reason) => {
                debug!("diagnostics suggested: {reason}");
                app.emit_diagnostics_suggested(DiagnosticsSuggestedReason::from_lib(reason));
            }
            lib::TunnelEvent::ConflictDetected(conflict) => {
                debug!("conflict detected: {conflict}");
                app.emit_conflict_detected(ConflictDetected::from_lib(conflict));
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_tunnel_update(app: &AppHandle, tun_state: lib::TunnelState) -> Result<()> {
        let tunnel = TunnelState::from_lib(tun_state);
        info!("tunnel state [{}]", tunnel);
        if let TunnelState::Error(e) = &tunnel {
            warn!("tunnel error: {:?}", e);
        }
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_tunnel(app, tunnel).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_account_update(
        app: &AppHandle,
        update: lib::AccountControllerState,
    ) -> Result<()> {
        log_account_state(&update);
        let account_state = AccountState::from_lib(update);
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_account_state(app, account_state).await?;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_config_update(app: &AppHandle, update: lib::VpnServiceConfig) -> Result<()> {
        let config = VpndConfig::from_lib(update)
            .inspect_err(|e| error!("failed to parse vpnd config: {e}"))?;
        let s_state = app.state::<SharedAppState>();
        let mut app_state = s_state.lock().await;
        app_state.update_vpnd_config(app, config).await?;
        Ok(())
    }

    /// Get the current daemon configuration
    #[instrument(skip_all)]
    pub async fn config(&self) -> Result<VpndConfig, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let config = vpnd
            .get_config()
            .or_else(async |e| self.handle_rpc_error("get_config", e).await)
            .await?;

        Ok(VpndConfig::from_lib(config)
            .inspect_err(|e| error!("failed to parse vpnd config: {e}"))?)
    }

    #[instrument(skip_all)]
    pub async fn set_entry_node(&self, node: Node) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let node = node.try_into()?;

        vpnd.set_entry_point(node)
            .or_else(async |e| self.handle_rpc_error("set_entry_point", e).await)
            .await
    }

    #[instrument(skip_all)]
    pub async fn set_exit_node(&self, node: Node) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let node = node.try_into()?;

        vpnd.set_exit_point(node)
            .or_else(async |e| self.handle_rpc_error("set_exit_point", e).await)
            .await
    }

    /// Enable or disable two-hop mode (aka wg)
    #[instrument(skip_all)]
    pub async fn set_two_hop(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_two_hop(enabled)
            .or_else(async |e| self.handle_rpc_error("set_enable_two_hop", e).await)
            .await
    }

    /// Enable or disable QUIC mode (aka bridges)
    #[instrument(skip_all)]
    pub async fn set_quic(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_bridges(enabled)
            .or_else(async |e| self.handle_rpc_error("set_enable_bridges", e).await)
            .await
    }

    /// Enable or disable domain fronting
    #[instrument(skip_all)]
    pub async fn set_fronting_mode(&self, mode: FrontingMode) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_fronting_mode(mode.into())
            .or_else(async |e| self.handle_rpc_error("set_fronting_mode", e).await)
            .await
    }

    /// Enable or disable no-IPv6 mode
    #[instrument(skip_all)]
    pub async fn set_no_ipv6(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_disable_ipv6(enabled)
            .or_else(async |e| self.handle_rpc_error("set_disable_ipv6", e).await)
            .await
    }

    /// Allow or disallow LAN access while connected to the VPN
    #[instrument(skip_all)]
    pub async fn set_allow_lan(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_allow_lan(enabled)
            .or_else(async |e| self.handle_rpc_error("set_allow_lan", e).await)
            .await
    }

    /// Enable or disable ad blocking
    #[instrument(skip_all)]
    pub async fn set_ad_block(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_ad_blocking(enabled)
            .or_else(async |e| self.handle_rpc_error("set_enable_ad_blocking", e).await)
            .await
    }

    /// Enable or disable detection of conflicting software (e.g. AdGuard's DNS protection)
    #[instrument(skip_all)]
    pub async fn set_conflict_detection(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_conflict_detection(enabled)
            .or_else(async |e| self.handle_rpc_error("set_enable_conflict_detection", e).await)
            .await
    }

    /// Connect to the VPN
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn vpn_connect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let ok = vpnd
            .connect_tunnel()
            .or_else(async |e| self.handle_rpc_error("connect_tunnel", e).await)
            .await?;

        // TODO: return the bool and handle this in the caller
        if !ok {
            warn!("vpn_connect: connect_tunnel returned false");
        }

        Ok(())
    }

    /// Reconnect the VPN
    #[instrument(skip_all)]
    pub async fn vpn_reconnect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.reconnect_tunnel()
            .or_else(async |e| self.handle_rpc_error("reconnect_tunnel", e).await)
            .await?;

        Ok(())
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let ok = vpnd
            .disconnect_tunnel()
            .or_else(async |e| self.handle_rpc_error("disconnect_tunnel", e).await)
            .await?;

        // TODO: return the bool and handle this in the caller
        if !ok {
            warn!("vpn_disconnect: disconnect_tunnel returned false");
        }

        Ok(())
    }

    /// Store an account
    #[instrument(skip_all)]
    pub async fn store_account(
        &self,
        mnemonic: Option<String>,
        signature: Option<String>,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let request = match (mnemonic, signature) {
            (Some(mnemonic), None) => lib::StoreAccountRequest::Vpn { mnemonic },
            (None, Some(signature)) => lib::StoreAccountRequest::Privy {
                mnemonic: signature,
            },
            _ => {
                return Err(VpndError::Response(BackendError::internal(
                    "either passphrase or signature must be provided",
                    None,
                )));
            }
        };

        let response = vpnd
            .store_account(request)
            .or_else(async |e| self.handle_rpc_error("store_account", e).await)
            .await?;

        debug!("store account response: {response:?}");
        if let Some(error) = response.error.map(BackendError::from) {
            Err(VpndError::Response(error))
        } else {
            Ok(())
        }
    }

    /// Removes everything related to the account, including the device identity,
    /// credential storage, mixnet keys, gateway registrations
    #[instrument(skip_all)]
    pub async fn forget_account(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd
            .forget_account()
            .or_else(async |e| self.handle_rpc_error("forget_account", e).await)
            .await?;

        debug!("forget account response: {response:?}");
        if let Some(error) = response.error.map(BackendError::from) {
            Err(VpndError::Response(error))
        } else {
            Ok(())
        }
    }

    /// Check if an account is stored
    #[instrument(skip_all)]
    pub async fn is_account_stored(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let is_stored = vpnd
            .is_account_stored()
            .or_else(async |e| self.handle_rpc_error("is_account_stored", e).await)
            .await?;

        debug!("account stored: {is_stored}");
        Ok(is_stored)
    }

    /// Get the account identity \
    /// public key derived from the mnemonic
    #[instrument(skip_all)]
    pub async fn account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let id = vpnd
            .get_account_identity()
            .or_else(async |e| self.handle_rpc_error("get_account_identity", e).await)
            .await?;

        debug!("account id: {id:?}");
        Ok(id)
    }

    // Get canonical account id
    #[instrument(skip_all)]
    pub async fn canonical_account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let id = vpnd
            .get_canonical_account_identity()
            .or_else(async |e| {
                self.handle_rpc_error("get_canonical_account_identity", e)
                    .await
            })
            .await?;

        debug!("canonical account id: {id:?}");
        Ok(id)
    }

    /// Get account mode
    #[instrument(skip_all)]
    pub async fn account_mode(&self) -> Result<Option<StoredAccountMode>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let mode = vpnd
            .get_account_mode()
            .or_else(async |e| self.handle_rpc_error("get_account_mode", e).await)
            .await?;

        Ok(mode.map(Into::into))
    }

    /// Get the device identity
    #[instrument(skip_all)]
    pub async fn device_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let id = vpnd
            .get_device_identity()
            .or_else(async |e| self.handle_rpc_error("get_device_identity", e).await)
            .await?;

        debug!("device id: {id:?}");
        Ok(id)
    }

    /// Get the account links
    #[instrument(skip_all)]
    pub async fn account_links(&self, _locale: &str) -> Result<AccountLinks, VpndError> {
        let mut vpnd = self.vpnd().await?;

        // TODO use the user local once website is i18n ready
        let locale = "en".to_string();

        let links = vpnd
            .get_account_links(locale)
            .or_else(async |e| self.handle_rpc_error("get_account_links", e).await)
            .await?;

        debug!("links: {links:?}");
        Ok(links.into())
    }

    /// Get the list of available gateways
    #[instrument(skip(self))]
    pub async fn gateways(&self, gw_type: GatewayType) -> Result<Vec<Gateway>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let options = lib::ListGatewaysOptions {
            gw_type: gw_type.into(),
            user_agent: Some(self.user_agent.clone()),
        };

        let gateways = vpnd
            .list_gateways(options)
            .or_else(async |e| self.handle_rpc_error("list_gateways", e).await)
            .await?;

        debug!("vpnd gateways count: {}", gateways.len());

        let gateways = parse_gateways(gateways, gw_type);

        debug!("parsed gateway #{}", gateways.len());
        Ok(gateways)
    }

    /// Get the gateways of the most recent successful connections for the given
    /// mode, most-recent-first.
    #[instrument(skip(self))]
    pub async fn recent_gateways(&self, mode: &VpnMode) -> Result<RecentGateways, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let params = lib::GetRecentGatewaysParams {
            tunnel_type: match mode {
                VpnMode::Mixnet => lib::TunnelType::Mixnet,
                VpnMode::Wg => lib::TunnelType::Wireguard,
            },
        };

        let recents = vpnd
            .get_recent_gateways(params)
            .or_else(async |e| self.handle_rpc_error("get_recent_gateways", e).await)
            .await?;

        let (entry_type, exit_type) = match mode {
            VpnMode::Mixnet => (GatewayType::MxEntry, GatewayType::MxExit),
            VpnMode::Wg => (GatewayType::Wg, GatewayType::Wg),
        };

        let recents = RecentGateways {
            entry: parse_gateways(recents.entry, entry_type),
            exit: parse_gateways(recents.exit, exit_type),
        };

        debug!(
            "parsed recent gateways: entry #{}, exit #{}",
            recents.entry.len(),
            recents.exit.len()
        );
        Ok(recents)
    }

    #[instrument(skip(self, app))]
    pub async fn update_vpnd_state(
        &mut self,
        info: VpndInfo,
        app: &AppHandle,
    ) -> Result<(), VpndError> {
        let net_compat = self.network_compat().await.ok().flatten();

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

        vpnd.set_network(network.to_owned())
            .or_else(async |e| self.handle_rpc_error("set_network", e).await)
            .await?;

        info!("vpnd network set to {network} ⚠ restart vpnd!");
        Ok(())
    }

    /// Get messages affecting the whole system, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn system_messages(&self) -> Result<Vec<SystemMessage>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let messages = vpnd
            .get_system_messages()
            .or_else(async |e| self.handle_rpc_error("get_system_messages", e).await)
            .await?;

        debug!("system messages: {messages:?}");
        Ok(messages.into_iter().map(Into::into).collect())
    }

    /// Get the feature flags, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn feature_flags(&self) -> Result<FeatureFlags, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let flags = vpnd
            .get_feature_flags()
            .or_else(async |e| self.handle_rpc_error("get_feature_flags", e).await)
            .await?;

        debug!("feature flags: {flags:?}");
        Ok(flags.into())
    }

    /// Get the network compatibility versions of supported vpn-core and tauri client
    #[instrument(skip_all)]
    pub async fn network_compat(&self) -> Result<Option<NetworkCompatVersions>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let net_compat = vpnd
            .get_network_compatibility()
            .or_else(async |e| self.handle_rpc_error("get_network_compatibility", e).await)
            .await?;

        debug!("network compat: {net_compat:?}");
        Ok(net_compat.map(NetworkCompatVersions::from))
    }

    /// Is sentry enabled at daemon level
    #[instrument(skip_all)]
    pub async fn sentry_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let enabled = vpnd
            .is_sentry_enabled()
            .or_else(async |e| self.handle_rpc_error("is_entry_enabled", e).await)
            .await?;

        debug!("sentry enabled: {enabled}");
        if enabled {
            info!("⚠ vpnd sentry monitoring is enabled ⚠");
        }
        Ok(enabled)
    }

    /// Enable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn enable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.enable_sentry()
            .or_else(async |e| self.handle_rpc_error("enable_sentry", e).await)
            .await?;

        info!("sentry enabled ⚠ restart vpnd!");
        Ok(())
    }

    /// Disable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn disable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_sentry()
            .or_else(async |e| self.handle_rpc_error("disable_sentry", e).await)
            .await?;

        info!("sentry disabled ⚠ restart vpnd!");
        Ok(())
    }

    /// Enable SOCKS5 proxy
    #[instrument(skip_all)]
    pub async fn enable_socks5(
        &self,
        socks5_settings: Socks5Settings,
        http_rpc_settings: HttpRpcSettings,
        exit_node: Node,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let exit_point: lib::ExitPoint = exit_node.try_into()?;

        let lib_socks5_settings = lib::Socks5Settings {
            listen_address: socks5_settings.listen_address,
        };

        let lib_http_rpc_settings = lib::HttpRpcSettings {
            listen_address: http_rpc_settings.listen_address,
        };

        vpnd.enable_socks5(lib_socks5_settings, lib_http_rpc_settings, exit_point)
            .or_else(async |e| self.handle_rpc_error("enable_socks5", e).await)
            .await?;

        info!("SOCKS5 proxy enabled");
        Ok(())
    }

    /// Disable SOCKS5 proxy
    #[instrument(skip_all)]
    pub async fn disable_socks5(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_socks5()
            .or_else(async |e| self.handle_rpc_error("disable_socks5", e).await)
            .await?;

        info!("SOCKS5 proxy disabled");
        Ok(())
    }

    /// Get SOCKS5 proxy status
    #[instrument(skip_all)]
    pub async fn get_socks5_status(&self) -> Result<Socks5Status, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let status = vpnd
            .get_socks5_status()
            .or_else(async |e| self.handle_rpc_error("get_socks5_status", e).await)
            .await?;

        debug!("SOCKS5 status: {status:?}");
        Ok(status.into())
    }

    /// Is network statistics collection enabled at daemon level
    #[instrument(skip_all)]
    pub async fn netstats_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let config = vpnd
            .get_config()
            .or_else(async |e| self.handle_rpc_error("get_config", e).await)
            .await?;

        let enabled = config.network_stats.enabled;
        debug!("network statistics collection enabled: {enabled}");
        if enabled {
            info!("⚠ vpnd network statistics collection enabled ⚠");
        }
        Ok(enabled)
    }

    /// Enable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn enable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.network_stats_set_enabled(true)
            .or_else(async |e| self.handle_rpc_error("network_stats_set_enabled", e).await)
            .await?;

        debug!("enabled vpnd network statistics collection");
        Ok(())
    }

    /// Disable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn disable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.network_stats_set_enabled(false)
            .or_else(async |e| self.handle_rpc_error("network_stats_set_enabled", e).await)
            .await?;

        debug!("disabled vpnd network statistics collection");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn get_default_dns(&self) -> Result<Vec<IpAddr>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.get_default_dns()
            .or_else(async |e| self.handle_rpc_error("get_default_dns", e).await)
            .await
    }

    #[instrument(skip_all)]
    pub async fn set_custom_dns_enabled(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_custom_dns(enabled)
            .or_else(async |e| self.handle_rpc_error("set_enable_custom_dns", e).await)
            .await
    }

    #[instrument(skip_all)]
    pub async fn set_custom_dns(&self, dns: Vec<IpAddr>) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_custom_dns(dns)
            .or_else(async |e| self.handle_rpc_error("set_custom_dns", e).await)
            .await
    }

    #[instrument(skip_all)]
    pub async fn get_privy_derivation_message(&self) -> Result<String, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let message = vpnd
            .get_privy_derivation_message()
            .or_else(async |e| {
                self.handle_rpc_error("get_privy_derivation_message", e)
                    .await
            })
            .await?;

        Ok(message.message)
    }

    #[instrument(skip_all)]
    pub async fn get_deep_link(
        &self,
        locale: String,
        kind: lib::DeeplinkKind,
    ) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let deeplink = vpnd
            .get_deeplink(lib::GetDeeplinkParams {
                client: lib::DeeplinkClient::Desktop,
                locale,
                kind,
                name: "default".to_string(),
            })
            .or_else(async |e| self.handle_rpc_error("get_deeplink", e).await)
            .await?;

        Ok(Some(deeplink))
    }

    #[instrument(skip_all)]
    pub async fn get_autologin_deeplink(
        &self,
        locale: String,
        kind: DeeplinkKind,
    ) -> Result<Option<AutologinResponse>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let result = vpnd
            .get_autologin_deeplink(lib::GetDeeplinkParams {
                client: lib::DeeplinkClient::Desktop,
                locale,
                kind: kind.into(),
                name: "default".to_string(),
            })
            .or_else(async |e| self.handle_rpc_error("get_autologin_deeplink", e).await)
            .await?;

        Ok(Some(result.into()))
    }

    #[instrument(skip_all)]
    pub async fn store_deeplink_account(&self, callback_url: String) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response: AccountCommandResponse = vpnd
            .deeplink_store_account(callback_url)
            .or_else(async |e| self.handle_rpc_error("deeplink_store_account", e).await)
            .await?;

        if let Some(error) = response.error.map(BackendError::from) {
            Err(VpndError::Response(error))
        } else {
            Ok(())
        }
    }

    pub async fn set_mixnet_traffic_config(
        &self,
        config: MixnetTrafficConfig,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_mixnet_traffic_config(config.into())
            .or_else(async |e| self.handle_rpc_error("set_mixnet_traffic_config", e).await)
            .await
    }

    pub async fn get_account_summary(&self) -> Result<Option<AccountSummary>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let summary = vpnd
            .get_account_summary()
            .or_else(async |e| self.handle_rpc_error("get_account_summary", e).await)
            .await?;

        Ok(summary.map(Into::into))
    }

    pub async fn refresh_account_state(&self, force: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.refresh_account_state(force)
            .or_else(async |e| self.handle_rpc_error("refresh_account_state", e).await)
            .await
    }

    pub async fn handle_subscription_payment(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.handle_subscription_payment()
            .or_else(async |e| {
                self.handle_rpc_error("handle_subscription_payment", e)
                    .await
            })
            .await
    }

    /// Run network diagnostics
    #[instrument(skip_all)]
    pub async fn run_diagnostic(
        &self,
        params: lib::DiagnosticRunParams,
    ) -> Result<lib::DiagnosticReport, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let report = vpnd
            .run_diagnostic(params)
            .or_else(async |e| self.handle_rpc_error("run_diagnostic", e).await)
            .await?;

        debug!("diagnostic report: {report:?}");
        Ok(report)
    }

    /// Enable split tunneling
    #[instrument(skip_all)]
    #[allow(unused_variables, unused_mut)]
    pub async fn enable_split_tunnel(&self, _enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        #[cfg(target_os = "windows")]
        {
            vpnd.set_enable_split_tunnel(_enabled)
                .or_else(async |e| {
                    info!(error = %e, "vpnd.enable_split_tunnel() RPC failed");
                    self.handle_rpc_error("enable_split_tunnel", e).await
                })
                .await
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            warn!("Split tunnel enabling can only be enabled on Windows");
            Ok(())
        }
    }

    /// Add app to split tunneling
    #[instrument(skip_all)]
    #[allow(unused_variables, unused_mut)]
    pub async fn add_app_to_split_tunnel(&self, app: SplitApp) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        #[cfg(target_os = "windows")]
        {
            vpnd.add_split_tunnel_app(app.into())
                .or_else(async |e| self.handle_rpc_error("add_split_tunnel_app", e).await)
                .await
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            warn!("Split tunnel addition can only be used on Windows");
            Ok(())
        }
    }

    /// Remove app from split tunneling
    #[instrument(skip_all)]
    #[allow(unused_variables, unused_mut)]
    pub async fn remove_app_from_split_tunnel(&self, app: SplitApp) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        #[cfg(target_os = "windows")]
        {
            vpnd.remove_split_tunnel_app(app.into())
                .or_else(async |e| self.handle_rpc_error("remove_split_tunnel_app", e).await)
                .await
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            warn!("Split tunnel removal can only be used on Windows");
            Ok(())
        }
    }

    #[instrument(skip_all)]
    pub async fn is_split_tunnel_supported(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let is_supported = vpnd
            .is_split_tunnel_supported()
            .or_else(async |e| self.handle_rpc_error("is_split_tunnel_supported", e).await)
            .await?;

        Ok(is_supported)
    }

    #[instrument(skip_all)]
    pub async fn set_enable_geo_location(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_geo_location(enabled)
            .or_else(async |e| self.handle_rpc_error("set_enable_geo_location", e).await)
            .await
    }

    #[instrument(skip_all)]
    /// Get the tentative entry/exit gateway pair for the current settings.
    #[instrument(skip_all)]
    pub async fn get_tentative_gateways(&self) -> Result<TentativeGateways, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let tentative = vpnd
            .get_tentative_gateways()
            .or_else(async |e| self.handle_rpc_error("get_tentative_gateways", e).await)
            .await?;

        Ok(tentative.into())
    }

    /// Enable or disable gateway independence. For this iteration all
    /// `different_*` constraints move together (all `true` or all `false`).
    #[instrument(skip_all)]
    pub async fn set_gateway_independence(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_gateway_independence(enabled)
            .or_else(async |e| {
                self.handle_rpc_error("set_enable_gateway_independence", e)
                    .await
            })
            .await
    }

    /// Enable or disable the gateway-independence reminder notifications.
    #[instrument(skip_all)]
    pub async fn set_gateway_independence_notifications(
        &self,
        enabled: bool,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_gateway_independence_notifications(enabled)
            .or_else(async |e| {
                self.handle_rpc_error("set_gateway_independence_notifications", e)
                    .await
            })
            .await
    }

    #[instrument(skip_all)]
    pub async fn set_geo_exclusion_enabled(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_geo_exclusion_enabled(enabled)
            .or_else(async |e| self.handle_rpc_error("set_geo_exclusion_enabled", e).await)
            .await
    }

    #[instrument(skip_all)]
    pub async fn set_geo_exclusion_listen_port(&self, port: u16) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_geo_exclusion_listen_port(port)
            .or_else(async |e| {
                self.handle_rpc_error("set_geo_exclusion_listen_port", e)
                    .await
            })
            .await
    }

    #[instrument(skip_all)]
    pub async fn set_geo_exclusion_excluded_countries(
        &self,
        countries: Vec<String>,
    ) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_geo_exclusion_excluded_countries(countries)
            .or_else(async |e| {
                self.handle_rpc_error("set_geo_exclusion_excluded_countries", e)
                    .await
            })
            .await
    }
}
