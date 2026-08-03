// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for RPC client from `nym-vpn-proto`.

uniffi::setup_scaffolding!();

use std::{net::IpAddr, path::PathBuf, sync::Arc};

use futures::StreamExt;
use nym_vpn_proto::rpc_client::{Error as DaemonRpcError, RpcClient as DaemonRpcClient};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use nym_common::ErrorExt;
use nym_favorites::{FavoritesError, FavoritesManager};
use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerState, AutologinResponse, EntryPoint, ExitPoint,
    FavoriteSelector, FavoriteSelectors, FeatureFlags, FrontingMode, Gateway, GatewayType,
    GetDeeplinkParams, HttpRpcSettings, LogPath, MixnetTrafficConfig, NetworkCompatibility,
    NymVpnDevice, NymVpnUsage, ParsedAccountLinks, PrivyDerivationMessage, RecentGateways,
    Socks5Settings, Socks5Status, StoreAccountRequest, StoredAccountMode, SystemMessage,
    TunnelEvent, TunnelState, TunnelType, VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};
#[cfg(target_os = "macos")]
use nym_vpn_lib_types::{SplitApp, SplitTunnelExcludedProcessList};

uniffi::use_remote_type!(nym_vpn_lib_types::IpAddr);
uniffi::use_remote_type!(nym_vpn_lib_types::PathBuf);

#[derive(Clone, uniffi::Object)]
struct RpcClient {
    inner: DaemonRpcClient,
}

#[uniffi::export(async_runtime = "tokio")]
impl RpcClient {
    #[uniffi::constructor]
    pub async fn new() -> Result<Self> {
        Ok(Self {
            inner: DaemonRpcClient::new().await?,
        })
    }

    pub async fn get_info(&self) -> Result<VpnServiceInfo> {
        Ok(self.inner.clone().get_info().await?)
    }

    pub async fn get_config(&self) -> Result<VpnServiceConfig> {
        Ok(self.inner.clone().get_config().await?)
    }

    pub async fn set_entry_point(&self, entry_point: EntryPoint) -> Result<()> {
        self.inner.clone().set_entry_point(entry_point).await?;
        Ok(())
    }

    pub async fn set_exit_point(&self, exit_point: ExitPoint) -> Result<()> {
        self.inner.clone().set_exit_point(exit_point).await?;
        Ok(())
    }

    pub async fn set_disable_ipv6(&self, disable_ipv6: bool) -> Result<()> {
        self.inner.clone().set_disable_ipv6(disable_ipv6).await?;
        Ok(())
    }

    pub async fn set_enable_two_hop(&self, enable_two_hop: bool) -> Result<()> {
        self.inner
            .clone()
            .set_enable_two_hop(enable_two_hop)
            .await?;
        Ok(())
    }

    pub async fn set_enable_ad_blocking(&self, enable_ad_blocking: bool) -> Result<()> {
        self.inner
            .clone()
            .set_enable_ad_blocking(enable_ad_blocking)
            .await?;
        Ok(())
    }

    pub async fn set_netstack(&self, netstack: bool) -> Result<()> {
        self.inner.clone().set_netstack(netstack).await?;
        Ok(())
    }

    pub async fn set_allow_lan(&self, allow_lan: bool) -> Result<()> {
        self.inner.clone().set_allow_lan(allow_lan).await?;
        Ok(())
    }

    pub async fn set_enable_bridges(&self, enable_bridges: bool) -> Result<()> {
        self.inner
            .clone()
            .set_enable_bridges(enable_bridges)
            .await?;
        Ok(())
    }

    pub async fn set_enable_custom_dns(&self, enable: bool) -> Result<()> {
        self.inner.clone().set_enable_custom_dns(enable).await?;
        Ok(())
    }

    pub async fn set_custom_dns(&self, dns_servers: Vec<IpAddr>) -> Result<()> {
        self.inner.clone().set_custom_dns(dns_servers).await?;
        Ok(())
    }

    pub async fn set_mixnet_traffic_config(
        &self,
        mixnet_traffic_config: MixnetTrafficConfig,
    ) -> Result<()> {
        self.inner
            .clone()
            .set_mixnet_traffic_config(mixnet_traffic_config)
            .await?;
        Ok(())
    }

    pub async fn set_enable_geo_location(&self, enable_geo_location: bool) -> Result<()> {
        self.inner
            .clone()
            .set_enable_geo_location(enable_geo_location)
            .await?;
        Ok(())
    }

    pub async fn set_geo_exclusion_enabled(&self, enabled: bool) -> Result<()> {
        self.inner
            .clone()
            .set_geo_exclusion_enabled(enabled)
            .await?;
        Ok(())
    }

    pub async fn set_geo_exclusion_listen_port(&self, listen_port: u16) -> Result<()> {
        self.inner
            .clone()
            .set_geo_exclusion_listen_port(listen_port)
            .await?;
        Ok(())
    }

    pub async fn set_geo_exclusion_excluded_countries(
        &self,
        excluded_countries: Vec<String>,
    ) -> Result<()> {
        self.inner
            .clone()
            .set_geo_exclusion_excluded_countries(excluded_countries)
            .await?;
        Ok(())
    }

    pub async fn set_enable_gateway_independence(
        &self,
        enable_gateway_independence: bool,
    ) -> Result<()> {
        self.inner
            .clone()
            .set_enable_gateway_independence(enable_gateway_independence)
            .await?;
        Ok(())
    }

    pub async fn set_gateway_independence_notifications(
        &self,
        enable_notifications: bool,
    ) -> Result<()> {
        self.inner
            .clone()
            .set_gateway_independence_notifications(enable_notifications)
            .await?;
        Ok(())
    }

    pub async fn set_network(&self, network: String) -> Result<()> {
        self.inner.clone().set_network(network).await?;
        Ok(())
    }

    pub async fn set_fronting_mode(&self, fronting_mode: FrontingMode) -> Result<()> {
        self.inner.clone().set_fronting_mode(fronting_mode).await?;
        Ok(())
    }

    pub async fn get_system_messages(&self) -> Result<Vec<SystemMessage>> {
        let system_messages = self.inner.clone().get_system_messages().await?;
        Ok(system_messages.into_iter().collect())
    }

    pub async fn get_network_compatibility(&self) -> Result<Option<NetworkCompatibility>> {
        let network_compatibility = self.inner.clone().get_network_compatibility().await?;
        Ok(network_compatibility)
    }

    pub async fn get_feature_flags(&self) -> Result<FeatureFlags> {
        Ok(self.inner.clone().get_feature_flags().await?)
    }

    pub async fn get_default_dns(&self) -> Result<Vec<IpAddr>> {
        let ips = self.inner.clone().get_default_dns().await?;
        Ok(ips)
    }

    pub async fn connect_tunnel(&self) -> Result<()> {
        self.inner.clone().connect_tunnel().await?;
        Ok(())
    }

    pub async fn disconnect_tunnel(&self) -> Result<()> {
        self.inner.clone().disconnect_tunnel().await?;
        Ok(())
    }

    pub async fn get_tunnel_state(&self) -> Result<TunnelState> {
        Ok(self.inner.clone().get_tunnel_state().await?)
    }

    pub async fn listen_to_events(
        &self,
        observer: Arc<dyn TunnelEventObserver>,
    ) -> Result<StreamObserver> {
        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();
        let mut event_stream = self.inner.clone().listen_to_events().await?;

        tokio::spawn(async move {
            loop {
                match child_token
                    .run_until_cancelled(event_stream.next())
                    .await
                    .flatten()
                {
                    Some(Ok(evt)) => {
                        observer.on_tunnel_event(evt);
                    }
                    Some(Err(err)) => {
                        tracing::error!("Error receiving next event: {err}");
                        break;
                    }
                    None => break,
                }
            }
            observer.on_close();
        });

        Ok(StreamObserver::new(cancel_token))
    }

    pub async fn list_gateways(&self, gw_type: GatewayType) -> Result<Vec<Gateway>> {
        let options = nym_vpn_lib_types::ListGatewaysOptions {
            gw_type,
            user_agent: None,
        };
        let gateways = self
            .inner
            .clone()
            .list_gateways(options)
            .await?
            .into_iter()
            .collect();
        Ok(gateways)
    }

    pub async fn get_recent_gateways(&self, tunnel_type: TunnelType) -> Result<RecentGateways> {
        let params = nym_vpn_lib_types::GetRecentGatewaysParams { tunnel_type };
        Ok(self.inner.clone().get_recent_gateways(params).await?)
    }

    pub async fn store_account(&self, request: StoreAccountRequest) -> Result<()> {
        let response = self.inner.clone().store_account(request).await?;

        if let Some(err) = response.error {
            Err(RpcError::new(InnerRpcError::AccountCommand(Arc::new(err))))
        } else {
            Ok(())
        }
    }

    pub async fn is_account_stored(&self) -> Result<bool> {
        Ok(self.inner.clone().is_account_stored().await?)
    }

    pub async fn forget_account(&self) -> Result<()> {
        let response = self.inner.clone().forget_account().await?;
        if let Some(err) = response.error {
            Err(RpcError::from(Arc::new(err)))
        } else {
            Ok(())
        }
    }

    pub async fn get_account_identity(&self) -> Result<Option<String>> {
        Ok(self.inner.clone().get_account_identity().await?)
    }

    pub async fn get_canonical_account_identity(&self) -> Result<Option<String>> {
        Ok(self.inner.clone().get_canonical_account_identity().await?)
    }

    pub async fn get_account_mode(&self) -> Result<Option<StoredAccountMode>> {
        Ok(self.inner.clone().get_account_mode().await?)
    }

    pub async fn get_account_links(&self, locale: String) -> Result<ParsedAccountLinks> {
        Ok(self.inner.clone().get_account_links(locale).await?)
    }

    pub async fn get_account_state(&self) -> Result<AccountControllerState> {
        Ok(self.inner.clone().get_account_state().await?)
    }

    pub async fn refresh_account_state(&self, force: bool) -> Result<()> {
        self.inner.clone().refresh_account_state(force).await?;
        Ok(())
    }

    pub async fn get_account_usage(&self) -> Result<Vec<NymVpnUsage>> {
        let usage = self
            .inner
            .clone()
            .get_account_usage()
            .await?
            .into_iter()
            .collect::<Vec<_>>();
        Ok(usage)
    }

    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>> {
        let summary = self.inner.clone().get_account_summary().await?;
        Ok(summary)
    }

    pub async fn handle_subscription_payment(&self) -> Result<()> {
        self.inner.clone().handle_subscription_payment().await?;
        Ok(())
    }

    pub async fn get_deeplink(&self, params: GetDeeplinkParams) -> Result<String> {
        Ok(self.inner.clone().get_deeplink(params).await?)
    }

    pub async fn deeplink_store_account(&self, deeplink_callback_url: String) -> Result<()> {
        self.inner
            .clone()
            .deeplink_store_account(deeplink_callback_url)
            .await?;
        Ok(())
    }

    pub async fn get_autologin_deeplink(
        &self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse> {
        Ok(self.inner.clone().get_autologin_deeplink(params).await?)
    }

    pub async fn reset_device_identity(&self, seed: Option<Vec<u8>>) -> Result<()> {
        self.inner.clone().reset_device_identity(seed).await?;
        Ok(())
    }

    pub async fn get_device_identity(&self) -> Result<Option<String>> {
        Ok(self.inner.clone().get_device_identity().await?)
    }

    pub async fn get_devices(&self) -> Result<Vec<NymVpnDevice>> {
        Ok(self
            .inner
            .clone()
            .get_devices()
            .await?
            .into_iter()
            .collect())
    }

    pub async fn get_active_devices(&self) -> Result<Vec<NymVpnDevice>> {
        Ok(self
            .inner
            .clone()
            .get_active_devices()
            .await?
            .into_iter()
            .collect())
    }

    pub async fn get_log_path(&self) -> Result<Option<LogPath>> {
        let log_path = self.inner.clone().get_log_path().await?;
        Ok(log_path)
    }

    pub async fn delete_log_file(&self) -> Result<()> {
        self.inner.clone().delete_log_file().await?;
        Ok(())
    }

    pub async fn is_sentry_enabled(&self) -> Result<bool> {
        Ok(self.inner.clone().is_sentry_enabled().await?)
    }

    pub async fn enable_sentry(&self) -> Result<()> {
        self.inner.clone().enable_sentry().await?;
        Ok(())
    }

    pub async fn disable_sentry(&self) -> Result<()> {
        self.inner.clone().disable_sentry().await?;
        Ok(())
    }

    pub async fn network_stats_set_enabled(&self, enabled: bool) -> Result<()> {
        self.inner
            .clone()
            .network_stats_set_enabled(enabled)
            .await?;
        Ok(())
    }

    pub async fn network_stats_allow_disconnected(&self, allow_disconnected: bool) -> Result<()> {
        self.inner
            .clone()
            .network_stats_allow_disconnected(allow_disconnected)
            .await?;
        Ok(())
    }

    pub async fn enable_socks5(
        &self,
        socks5_settings: Socks5Settings,
        http_rpc_settings: HttpRpcSettings,
        exit_point: ExitPoint,
    ) -> Result<()> {
        self.inner
            .clone()
            .enable_socks5(socks5_settings, http_rpc_settings, exit_point)
            .await?;
        Ok(())
    }

    pub async fn disable_socks5(&self) -> Result<()> {
        self.inner.clone().disable_socks5().await?;
        Ok(())
    }

    pub async fn get_socks5_status(&self) -> Result<Socks5Status> {
        let status = self.inner.clone().get_socks5_status().await?;
        Ok(status)
    }

    pub async fn get_privy_derivation_message(&self) -> Result<PrivyDerivationMessage> {
        let message = self.inner.clone().get_privy_derivation_message().await?;
        Ok(message)
    }
}

#[cfg(target_os = "macos")]
#[uniffi::export(async_runtime = "tokio")]
impl RpcClient {
    pub async fn set_enable_split_tunnel(&self, enable: bool) -> Result<()> {
        self.inner.clone().set_enable_split_tunnel(enable).await?;
        Ok(())
    }

    pub async fn add_split_tunnel_app(&self, app: SplitApp) -> Result<()> {
        self.inner.clone().add_split_tunnel_app(app).await?;
        Ok(())
    }

    pub async fn remove_split_tunnel_app(&self, app: SplitApp) -> Result<()> {
        self.inner.clone().remove_split_tunnel_app(app).await?;
        Ok(())
    }

    pub async fn clear_split_tunnel_apps(&self) -> Result<()> {
        self.inner.clone().clear_split_tunnel_apps().await?;
        Ok(())
    }

    pub async fn get_split_tunnel_excluded_processes(
        &self,
    ) -> Result<SplitTunnelExcludedProcessList> {
        Ok(self
            .inner
            .clone()
            .get_split_tunnel_excluded_processes()
            .await?)
    }

    pub async fn need_full_disk_permissions(&self) -> Result<bool> {
        let value = self.inner.clone().need_full_disk_permissions().await?;
        Ok(value)
    }

    pub async fn run_diagnostic(&self) -> Result<String> {
        let params = nym_vpn_lib_types::DiagnosticRunParams {
            gateway: None,
            skip_dns: false,
            skip_http: false,
            skip_hybrid_transport: false,
        };
        Ok(self.inner.clone().run_diagnostic_raw(params).await?)
    }
}

/// Favorite entry/exit selectors, stored as `favorites.json` in `data_dir`.
///
/// Favorites are a client-side file, not daemon state — the daemon neither reads
/// nor writes them — so this wraps `FavoritesManager` directly, the same way
/// `nym-vpnc` does, rather than going through the RPC client.
#[derive(uniffi::Object)]
pub struct FavoritesController {
    manager: Arc<RwLock<FavoritesManager>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl FavoritesController {
    #[uniffi::constructor]
    pub async fn open(data_dir: PathBuf) -> Self {
        Self {
            manager: Arc::new(RwLock::new(FavoritesManager::new(data_dir).await)),
        }
    }

    pub async fn add_favorite_entry(&self, selector: FavoriteSelector) -> Result<()> {
        self.manager
            .write()
            .await
            .add_favorite_entry(selector)
            .await?;
        Ok(())
    }

    pub async fn add_favorite_exit(&self, selector: FavoriteSelector) -> Result<()> {
        self.manager
            .write()
            .await
            .add_favorite_exit(selector)
            .await?;
        Ok(())
    }

    pub async fn remove_favorite_entry(&self, selector: FavoriteSelector) -> Result<()> {
        self.manager
            .write()
            .await
            .remove_favorite_entry(selector)
            .await?;
        Ok(())
    }

    pub async fn remove_favorite_exit(&self, selector: FavoriteSelector) -> Result<()> {
        self.manager
            .write()
            .await
            .remove_favorite_exit(selector)
            .await?;
        Ok(())
    }

    pub async fn get_favorites(&self) -> FavoriteSelectors {
        self.manager.read().await.get_favorites()
    }
}

#[derive(Debug)]
pub enum InnerRpcError {
    RpcError(DaemonRpcError),
    AccountCommand(Arc<AccountCommandError>),
    Favorites(FavoritesError),
}

impl std::fmt::Display for InnerRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerRpcError::RpcError(err) => write!(f, "{err}"),
            InnerRpcError::AccountCommand(err) => write!(f, "{err}"),
            InnerRpcError::Favorites(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Debug, uniffi::Object)]
#[uniffi::export(Display)]
pub struct RpcError {
    inner: InnerRpcError,
}

impl RpcError {
    pub fn new(inner: InnerRpcError) -> Self {
        RpcError { inner }
    }
}

#[uniffi::export]
impl RpcError {
    /// Returns the account error if the underlying error is an account error.
    pub fn account_error(&self) -> Option<AccountCommandError> {
        match &self.inner {
            InnerRpcError::AccountCommand(err) => Some(err.as_ref().clone()),
            _ => None,
        }
    }

    /// Print the underlying error chain
    pub fn display_chain(&self) -> String {
        match &self.inner {
            InnerRpcError::AccountCommand(err) => err.display_chain(),
            InnerRpcError::RpcError(err) => err.display_chain(),
            InnerRpcError::Favorites(err) => err.to_string(),
        }
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<FavoritesError> for RpcError {
    fn from(err: FavoritesError) -> Self {
        RpcError {
            inner: InnerRpcError::Favorites(err),
        }
    }
}

impl From<DaemonRpcError> for RpcError {
    fn from(err: DaemonRpcError) -> Self {
        RpcError {
            inner: InnerRpcError::RpcError(err),
        }
    }
}

impl From<Arc<AccountCommandError>> for RpcError {
    fn from(err: Arc<AccountCommandError>) -> Self {
        RpcError {
            inner: InnerRpcError::AccountCommand(err),
        }
    }
}

pub type Result<T, E = RpcError> = std::result::Result<T, E>;

#[derive(Clone, uniffi::Object)]
pub struct StreamObserver {
    cancel_token: CancellationToken,
}

#[uniffi::export]
impl StreamObserver {
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

impl StreamObserver {
    fn new(cancel_token: CancellationToken) -> Self {
        StreamObserver { cancel_token }
    }
}

impl Drop for StreamObserver {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

#[uniffi::export(with_foreign)]
pub trait TunnelEventObserver: Send + Sync {
    fn on_tunnel_event(&self, event: TunnelEvent);
    fn on_close(&self);
}
