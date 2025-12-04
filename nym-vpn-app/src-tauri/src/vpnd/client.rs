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
use once_cell::sync::Lazy;
use std::{
    env::consts::{ARCH, OS},
    path::PathBuf,
    sync::Mutex,
    net::IpAddr,
};
use tauri::{AppHandle, Manager, PackageInfo};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::vpnd::network::NetworkCompatVersions;
use crate::{
    error::BackendError,
    events::AppHandleEventEmitter,
    state::SharedAppState,
    vpnd::account::{AccountState, log_account_state},
};

// simple flag to save that "failed to connect to daemon"
// warning has been logged once when vpnd is down
static VPND_DOWN_LOGGED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[derive(Debug, Clone)]
pub struct VpndClient {
    pkg_info: PackageInfo,
    user_agent: UserAgent,
}

impl VpndClient {
    #[instrument(skip_all)]
    pub fn new(pkg: &PackageInfo) -> Self {
        VpndClient {
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
        let client = RpcClient::new().await.map_err(|e| {
            let mut logged = VPND_DOWN_LOGGED.lock().unwrap();
            if !*logged {
                warn!("failed to connect to the daemon: {}", e);
                *logged = true;
            } else {
                trace!("failed to connect to the daemon: {}", e);
            }
            VpndError::FailedToConnectIpc(e.into())
        })?;
        Ok(client)
    }

    /// Get daemon info
    #[instrument(skip_all)]
    pub async fn vpnd_info(&mut self) -> Result<VpndInfo, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let info: VpndInfo = vpnd
            .get_info()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?
            .into();

        info!("vpnd UP");
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

        let log_path = vpnd
            .get_log_path()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("vpnd log path: {:?}", log_path);
        Ok(log_path.dir)
    }

    /// Get the current tunnel state and update the app state
    #[instrument(skip_all)]
    pub async fn tunnel_state(&self, app: &AppHandle) -> Result<TunnelState, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let tun_state = vpnd.get_tunnel_state().await?;
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
    pub async fn watch_events(&self, app: &AppHandle) -> Result<()> {
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

        let mut logged = VPND_DOWN_LOGGED.lock().unwrap();
        if !*logged {
            warn!("vpnd DOWN: stream closed");
            *logged = true;
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
        let config = vpnd
            .get_config()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(VpndConfig::from_lib(config)
            .inspect_err(|e| error!("failed to parse vpnd config: {e}"))?)
    }

    #[instrument(skip_all)]
    pub async fn set_entry_node(&self, node: Node) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_entry_point(node.try_into()?)
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn set_exit_node(&self, node: Node) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_exit_point(node.try_into()?)
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Enable or disable two-hop mode (aka wg)
    #[instrument(skip_all)]
    pub async fn set_two_hop(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;
        vpnd.set_enable_two_hop(enabled)
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Enable or disable QUIC mode (aka bridges)
    #[instrument(skip_all)]
    pub async fn set_quic(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;
        vpnd.set_enable_bridges(enabled)
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Enable or disable no-IPv6 mode
    #[instrument(skip_all)]
    pub async fn set_no_ipv6(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;
        vpnd.set_disable_ipv6(enabled)
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Allow or disallow LAN access while connected to the VPN
    #[instrument(skip_all)]
    pub async fn set_allow_lan(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;
        vpnd.set_allow_lan(enabled)
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Connect to the VPN
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn vpn_connect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.connect_tunnel()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        Ok(())
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disconnect_tunnel()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Store an account
    #[instrument(skip_all)]
    pub async fn store_account(&self, mnemonic: String) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd
            .store_account(lib::StoreAccountRequest::Vpn { mnemonic })
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("response: {:?}", response);
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

        let response = vpnd
            .forget_account()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("response: {:?}", response);
        if let Some(error) = response.error.map(BackendError::from) {
            return Err(VpndError::Response(error));
        }
        Ok(())
    }

    /// Check if an account is stored
    #[instrument(skip_all)]
    pub async fn is_account_stored(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let is_stored = vpnd
            .is_account_stored()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("account stored: {}", is_stored);
        Ok(is_stored)
    }

    /// Get the account identity \
    /// public key derived from the mnemonic
    #[instrument(skip_all)]
    pub async fn account_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let id = vpnd
            .get_account_identity()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("account id: {:?}", id);
        Ok(id)
    }

    /// Get the device identity
    #[instrument(skip_all)]
    pub async fn device_id(&self) -> Result<Option<String>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let id = vpnd
            .get_device_identity()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("device id: {:?}", id);
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
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("links: {:?}", links);
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
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
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
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        Ok(())
    }

    /// Get messages affecting the whole system, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn system_messages(&self) -> Result<Vec<SystemMessage>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let messages = vpnd
            .get_system_messages()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("system messages: {:?}", messages);
        Ok(messages.into_iter().map(Into::into).collect())
    }

    /// Get the feature flags, fetched from nym-vpn-api
    #[instrument(skip_all)]
    pub async fn feature_flags(&self) -> Result<FeatureFlags, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let flags = vpnd
            .get_feature_flags()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("feature flags: {:?}", flags);
        Ok(flags.into())
    }

    /// Get the network compatibility versions of supported vpn-core and tauri client
    #[instrument(skip_all)]
    pub async fn network_compat(&self) -> Result<Option<NetworkCompatVersions>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let net_compat = vpnd
            .get_network_compatibility()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;
        debug!("network compat: {:?}", net_compat);
        Ok(net_compat.map(NetworkCompatVersions::from))
    }

    /// Is sentry enabled at daemon level
    #[instrument(skip_all)]
    pub async fn sentry_enabled(&self) -> Result<bool, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let enabled = vpnd
            .is_sentry_enabled()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("sentry enabled: {}", enabled);
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
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("enabled vpnd sentry");
        info!("restart vpnd (service) required for the change to take effect");
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
            .await
            .map_err(|e| {
                error!("failed to enable SOCKS5 proxy: {}", e);
                VpndError::RpcClient(e)
            })?;

        info!("SOCKS5 proxy enabled");
        Ok(())
    }

    /// Disable SOCKS5 proxy
    #[instrument(skip_all)]
    pub async fn disable_socks5(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_socks5().await.map_err(|e| {
            error!("failed to disable SOCKS5 proxy: {}", e);
            VpndError::RpcClient(e)
        })?;

        info!("SOCKS5 proxy disabled");
        Ok(())
    }

    /// Get SOCKS5 proxy status
    #[instrument(skip_all)]
    pub async fn get_socks5_status(&self) -> Result<Socks5Status, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let response = vpnd.get_socks5_status().await.map_err(|e| {
            error!("failed to get SOCKS5 status: {}", e);
            VpndError::RpcClient(e)
        })?;

        debug!("SOCKS5 status: {:?}", response);

        Ok(response.into())
    }

    /// Disable sentry at daemon level
    #[instrument(skip_all)]
    pub async fn disable_sentry(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_sentry()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
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
            .is_collect_network_stats_enabled()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("network statistics collection enabled: {}", enabled);
        if enabled {
            info!("⚠ vpnd network statistics collection enabled ⚠");
        }
        Ok(enabled)
    }

    /// Enable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn enable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.enable_collect_network_stats()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("enabled vpnd network statistics collection");
        info!("restart vpnd (service) required for the change to take effect");
        Ok(())
    }

    /// Disable network statistics collection at daemon level
    #[instrument(skip_all)]
    pub async fn disable_netstats(&self) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.disable_collect_network_stats()
            .await
            .map_err(VpndError::RpcClient)
            .inspect_err(|e| {
                error!("rpc: {}", e);
            })?;

        debug!("disabled vpnd network statistics collection");
        info!("restart vpnd (service) required for the change to take effect");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn get_default_dns(&self) -> Result<Vec<IpAddr>, VpndError> {
        let mut vpnd = self.vpnd().await?;

        let dns = vpnd.get_default_dns().await.map_err(|e| {
            error!("failed to get default DNS: {}", e);
            VpndError::RpcClient(e)
        })?;
        Ok(dns)
    }

    #[instrument(skip_all)]
    pub async fn set_custom_dns_enabled(&self, enabled: bool) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        vpnd.set_enable_custom_dns(enabled).await.map_err(VpndError::RpcClient).inspect_err(|e| {
            error!("failed to set custom DNS enabled: {}", e);
        })?;


        debug!("custom DNS enabled: {}", enabled);
        if enabled {
            info!("⚠ vpnd custom DNS enabled ⚠");
        } else {
            info!("custom DNS disabled");
        }
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn set_custom_dns(&self, dns: Vec<IpAddr>) -> Result<(), VpndError> {
        let mut vpnd = self.vpnd().await?;

        let borrowed_dns = dns.clone();

        vpnd.set_custom_dns(dns).await.map_err(VpndError::RpcClient).inspect_err(|e| {
            error!("failed to set custom DNS: {}", e);
        })?;

        debug!("custom DNS set: {:?}", borrowed_dns);
        Ok(())
    }

    pub fn reset_log_flag() {
        let mut logged = VPND_DOWN_LOGGED.lock().unwrap();
        *logged = false;
    }
}
