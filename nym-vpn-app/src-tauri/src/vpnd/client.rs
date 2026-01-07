pub use super::{
    account_links::AccountLinks,
    error::VpndError,
    feature_flags::FeatureFlags,
    node::Node,
    socks5::{HttpRpcSettings, Socks5Settings, Socks5Status},
    system_message::SystemMessage,
    vpnd_status::{VersionCheck, VpndInfo, VpndStatus},
};
use super::{
    config::VpndConfig,
    events::MixnetEvent,
    gateway::{Gateway, GatewayType},
    tunnel::TunnelState,
};

use anyhow::Result;
use lib::UserAgent;
use nym_vpn_lib_types as lib;
use nym_vpn_proto::rpc_client::RpcClient;
use std::sync::Arc;
use std::{
    env::consts::{ARCH, OS},
    net::IpAddr,
    path::PathBuf,
};
use tauri::{AppHandle, Manager, PackageInfo};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::vpnd::network::NetworkCompatVersions;
use crate::{
    error::BackendError,
    events::AppHandleEventEmitter,
    state::SharedAppState,
    vpnd::account::{AccountState, log_account_state},
};

#[derive(Debug, Clone)]
pub struct VpndClient {
    rpc_client: Arc<Mutex<Option<RpcClient>>>,
    connect_fail_logged: Arc<Mutex<bool>>,
    pkg_info: PackageInfo,
    user_agent: UserAgent,
}

impl VpndClient {
    #[instrument(skip_all)]
    pub fn new(pkg: &PackageInfo) -> Self {
        VpndClient {
            rpc_client: Arc::new(Mutex::new(None)),
            connect_fail_logged: Arc::new(Mutex::new(false)),
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
        {
            // fast path: already created
            let guard = self.rpc_client.lock().await;
            if let Some(client) = &*guard {
                return Ok(client.clone());
            }
        }

        // slow path: create new client
        let client = match RpcClient::new().await {
            Ok(c) => c,
            Err(e) => {
                self.log_connect_failed().await;
                return Err(VpndError::FailedToConnectIpc(e.into()));
            }
        };

        debug!("connected to the daemon");

        let mut guard = self.rpc_client.lock().await;
        *guard = Some(client.clone());
        Ok(client)
    }

    async fn drop_rpc_client(&self) {
        let mut guard = self.rpc_client.lock().await;
        if guard.is_some() {
            *guard = None;
            debug!("dropped daemon connection");
        } else {
            debug!("daemon connection already dropped");
        }

        self.reset_log_connect_failed().await;
    }

    async fn log_connect_failed(&self) {
        let mut guard = self.connect_fail_logged.lock().await;
        if !*guard {
            warn!("failed to connect to the daemon");
            *guard = true;
        }
    }

    async fn reset_log_connect_failed(&self) {
        let mut guard = self.connect_fail_logged.lock().await;
        if *guard {
            *guard = false;
        }
    }

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&mut self) -> Result<VpndInfo, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let info: VpndInfo = match vpnd.get_info().await {
            Ok(res) => res.into(),
            Err(e) => {
                error!("rpc: {e}");
                self.drop_rpc_client().await;
                return Err(VpndError::RpcClient(e));
            }
        };

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
    pub async fn vpnd_log_path(&self) -> Result<PathBuf, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let log_path = match vpnd.get_log_path().await {
            Ok(res) => res,
            Err(e) => {
                error!("rpc: {e}");
                self.drop_rpc_client().await;
                return Err(VpndError::RpcClient(e));
            }
        };

        debug!("vpnd log path: {:?}", log_path);
        Ok(log_path.dir)
    }

    /// Get the current tunnel state and update the app state
    #[instrument(skip_all)]
    pub async fn tunnel_state(&self, app: &AppHandle) -> Result<TunnelState, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let tun_state = match vpnd.get_tunnel_state().await {
            Ok(state) => state,
            Err(e) => {
                error!("rpc: {e}");
                self.drop_rpc_client().await;
                return Err(VpndError::RpcClient(e));
            }
        };

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
    pub async fn watch_events(&mut self, app: &AppHandle) -> Result<()> {
        let mut vpnd = self.vpnd().await?;

        let mut stream = vpnd.listen_to_events().await.inspect_err(|e| {
            error!("listen to events failed: {}", e);
        })?;

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

        match vpnd.get_config().await {
            Ok(config) => Ok(VpndConfig::from_lib(config)
                .inspect_err(|e| error!("failed to parse vpnd config: {e}"))?),
            Err(e) => {
                error!("vpnd.get_config() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn set_entry_node(&self, node: Node) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_entry_point(node.try_into()?).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_entry_point() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn set_exit_node(&self, node: Node) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_exit_point(node.try_into()?).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_exit_point() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Enable or disable two-hop mode (aka wg)
    #[instrument(skip_all)]
    pub async fn set_two_hop(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_enable_two_hop(enabled).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_enable_two_hop() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Enable or disable QUIC mode (aka bridges)
    #[instrument(skip_all)]
    pub async fn set_quic(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_enable_bridges(enabled).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_enable_bridges() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Enable or disable no-IPv6 mode
    #[instrument(skip_all)]
    pub async fn set_no_ipv6(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_disable_ipv6(enabled).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_disable_ipv6() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Allow or disallow LAN access while connected to the VPN
    #[instrument(skip_all)]
    pub async fn set_allow_lan(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_allow_lan(enabled).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_allow_lan() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Connect to the VPN
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn vpn_connect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.connect_tunnel().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.connect_tunnel() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.disconnect_tunnel().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.disconnect_tunnel() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
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
                hex_signature: signature,
            },
            _ => {
                return Err(VpndError::Response(BackendError::internal(
                    "either mnemonic or signature must be provided",
                    None,
                )));
            }
        };

        match vpnd.store_account(request).await {
            Ok(res) => {
                debug!("store account response: {res:?}");
                if let Some(error) = res.error.map(BackendError::from) {
                    Err(VpndError::Response(error))
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                error!("vpnd.store_account() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Removes everything related to the account, including the device identity,
    /// credential storage, mixnet keys, gateway registrations
    #[instrument(skip_all)]
    pub async fn forget_account(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.forget_account().await {
            Ok(res) => {
                debug!("forget account response: {res:?}");
                if let Some(error) = res.error.map(BackendError::from) {
                    Err(VpndError::Response(error))
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                error!("vpnd.forget_account() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Check if an account is stored
    #[instrument(skip_all)]
    pub async fn is_account_stored(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.is_account_stored().await {
            Ok(is_stored) => {
                debug!("account stored: {is_stored}");
                Ok(is_stored)
            }
            Err(e) => {
                error!("vpnd.is_account_stored() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get the account identity \
    /// public key derived from the mnemonic
    #[instrument(skip_all)]
    pub async fn account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_account_identity().await {
            Ok(id) => {
                debug!("account id: {id:?}");
                Ok(id)
            }
            Err(e) => {
                error!("vpnd.get_account_identity() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get the device identity
    #[instrument(skip_all)]
    pub async fn device_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_device_identity().await {
            Ok(id) => {
                debug!("device id: {id:?}");
                Ok(id)
            }
            Err(e) => {
                error!("vpnd.get_device_identity() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get the account links
    #[instrument(skip_all)]
    pub async fn account_links(&self, _locale: &str) -> Result<AccountLinks, VpndError> {
        let mut vpnd = self.vpnd().await?;

        // TODO use the user local once website is i18n ready
        let locale = "en".to_string();

        match vpnd.get_account_links(locale).await {
            Ok(links) => {
                debug!("links: {links:?}");
                Ok(links.into())
            }
            Err(e) => {
                error!("vpnd.get_account_links() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get the list of available gateways
    #[instrument(skip(self))]
    pub async fn gateways(&self, gw_type: GatewayType) -> Result<Vec<Gateway>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let options = lib::ListGatewaysOptions {
            gw_type: gw_type.into(),
            user_agent: Some(self.user_agent.clone()),
        };

        match vpnd.list_gateways(options).await {
            Ok(gateways) => {
                debug!("vpnd gateways count: {}", gateways.len());

                let gateways: Vec<Gateway> = gateways
                    .into_iter()
                    .filter_map(|gateway| {
                        Gateway::from_lib(gateway, gw_type)
                            .inspect_err(|e| warn!("failed to parse gateway from lib: {e}"))
                            .ok()
                    })
                    .collect();

                debug!("parsed gateway #{}", gateways.len());
                Ok(gateways)
            }
            Err(e) => {
                error!("vpnd.list_gateways_failed(): {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
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

        match vpnd.set_network(network.to_owned()).await {
            Ok(_) => {
                info!("vpnd network set to {network} ⚠ restart vpnd!");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.set_network() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get messages affecting the whole system, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn system_messages(&self) -> Result<Vec<SystemMessage>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_system_messages().await {
            Ok(messages) => {
                debug!("system messages: {messages:?}");
                Ok(messages.into_iter().map(Into::into).collect())
            }
            Err(e) => {
                error!("vpnd.get_system_messages() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get the feature flags, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn feature_flags(&self) -> Result<FeatureFlags, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_feature_flags().await {
            Ok(flags) => {
                debug!("feature flags: {flags:?}");
                Ok(flags.into())
            }
            Err(e) => {
                error!("vpnd.get_feature_flags() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get the network compatibility versions of supported vpn-core and tauri client
    #[instrument(skip_all)]
    pub async fn network_compat(&self) -> Result<Option<NetworkCompatVersions>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_network_compatibility().await {
            Ok(net_compat) => {
                debug!("network compat: {net_compat:?}");
                Ok(net_compat.map(NetworkCompatVersions::from))
            }
            Err(e) => {
                error!("vpnd.get_network_compatibility() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Is sentry enabled at daemon level
    #[instrument(skip_all)]
    pub async fn sentry_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.is_sentry_enabled().await {
            Ok(enabled) => {
                debug!("sentry enabled: {enabled}");
                if enabled {
                    info!("⚠ vpnd sentry monitoring is enabled ⚠");
                }
                Ok(enabled)
            }
            Err(e) => {
                error!("vpnd.is_sentry_enabled() failed: {e}");
                self.drop_rpc_client().await;
                return Err(VpndError::RpcClient(e));
            }
        }
    }

    /// Enable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn enable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.enable_sentry().await {
            Ok(_) => {
                info!("sentry enabled ⚠ restart vpnd!");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.enable_sentry() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Disable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn disable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.disable_sentry().await {
            Ok(_) => {
                info!("sentry disabled ⚠ restart vpnd!");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.disable_sentry() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
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

        match vpnd
            .enable_socks5(lib_socks5_settings, lib_http_rpc_settings, exit_point)
            .await
        {
            Ok(_) => {
                info!("SOCKS5 proxy enabled");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.enable_socks5() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Disable SOCKS5 proxy
    #[instrument(skip_all)]
    pub async fn disable_socks5(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.disable_socks5().await {
            Ok(_) => {
                info!("SOCKS5 proxy disabled");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.disable_socks5() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Get SOCKS5 proxy status
    #[instrument(skip_all)]
    pub async fn get_socks5_status(&self) -> Result<Socks5Status, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_socks5_status().await {
            Ok(res) => {
                debug!("SOCKS5 status: {res:?}");
                Ok(res.into())
            }
            Err(e) => {
                error!("vpnd.get_socks5_status() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Is network statistics collection enabled at daemon level
    #[instrument(skip_all)]
    pub async fn netstats_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_config().await {
            Ok(config) => {
                let enabled = config.network_stats.enabled;
                debug!("network statistics collection enabled: {enabled}");
                if enabled {
                    info!("⚠ vpnd network statistics collection enabled ⚠");
                }
                Ok(enabled)
            }
            Err(e) => {
                error!("vpnd.get_config() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Enable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn enable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.network_stats_set_enabled(true).await {
            Ok(_) => {
                debug!("enabled vpnd network statistics collection");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.network_stats.set_enabled() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    /// Disable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn disable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.network_stats_set_enabled(false).await {
            Ok(_) => {
                debug!("disabled vpnd network statistics collection");
                Ok(())
            }
            Err(e) => {
                error!("vpnd.network_stats_set_enabled() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn get_default_dns(&self) -> Result<Vec<IpAddr>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_default_dns().await {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("vpnd.get_default_dns() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn set_custom_dns_enabled(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_enable_bridges(enabled).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_enable_bridges() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn set_custom_dns(&self, dns: Vec<IpAddr>) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.set_custom_dns(dns).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("vpnd.set_custom_dns() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }

    #[instrument(skip_all)]
    pub async fn get_privy_derivation_message(&self) -> Result<String, VpndError> {
        let mut vpnd = self.vpnd().await?;

        match vpnd.get_privy_derivation_message().await {
            Ok(message) => Ok(message.message),
            Err(e) => {
                error!("vpnd.get_privy_derivation_message() failed: {e}");
                self.drop_rpc_client().await;
                Err(VpndError::RpcClient(e))
            }
        }
    }
}
