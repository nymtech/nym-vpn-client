// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, error::Error as _, net::IpAddr, path::PathBuf};

use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::SplitApp;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use nym_vpn_lib_types::SplitTunnelExcludedProcessList;
use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandResponse, AccountControllerState, AppCompatibility,
    AutologinResponse, AvailableTickets, BridgeMode, DiagnosticRegisterParams, DiagnosticReport,
    DiagnosticRunParams, EnableSocks5Request, EntryPoint, ExitPoint, FeatureFlags, FrontingMode,
    Gateway, GetDeeplinkParams, GetRecentGatewaysParams, HttpRpcSettings, ListGatewaysOptions,
    LogPath, LookupGatewayFilters, MixnetTrafficConfig, NetworkStatisticsIdentity, NymVpnDevice,
    NymVpnUsage, ParsedAccountLinks, PrivyDerivationMessage, ProfileOptions, RecentGateways,
    RegistrationReport, Socks5Settings, Socks5Status, StoreAccountRequest, StoredAccountMode,
    SystemMessage, TentativeGateways, TunnelEvent, TunnelState, VpnAccountSummary,
    VpnServiceConfig, VpnServiceInfo,
};

use crate::NymVpnServiceClient;

type ServiceClient = NymVpnServiceClient<Channel>;

#[derive(Debug, Clone)]
pub struct RpcClient(ServiceClient);

fn into_inner<T>(response: std::result::Result<tonic::Response<T>, tonic::Status>) -> Result<T> {
    response
        .map(tonic::Response::into_inner)
        .map_err(Error::Rpc)
}

impl RpcClient {
    pub async fn new() -> Result<RpcClient> {
        Endpoint::from_static("unix://placeholder")
            .connect_with_connector(service_fn(move |_: Uri| {
                nym_ipc::client::connect(get_rpc_socket_path())
            }))
            .await
            .map(ServiceClient::new)
            .map(RpcClient)
            .map_err(|err| {
                if let Some(std::io::ErrorKind::PermissionDenied) = err
                    .source()
                    .and_then(|err| err.source())
                    .and_then(|err| err.downcast_ref::<std::io::Error>())
                    .map(|err| err.kind())
                {
                    Error::AuthenticationRequired
                } else {
                    err.into()
                }
            })
    }

    pub async fn new_over_serial<C>(
        connector: C,
        timeout: Option<std::time::Duration>,
    ) -> Result<RpcClient>
    where
        C: tower::Service<Uri> + Send + 'static,
        C::Response: hyper::rt::Read + hyper::rt::Write + Send + Unpin,
        C::Future: Send,
        C::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut endpoint = Endpoint::from_static("serial://placeholder");
        if let Some(timeout) = timeout {
            endpoint = endpoint.timeout(timeout);
        }

        endpoint
            .connect_with_connector(connector)
            .await
            .map(ServiceClient::new)
            .map(RpcClient)
            .map_err(Error::Transport)
    }

    pub async fn get_info(&mut self) -> Result<VpnServiceInfo> {
        into_inner(self.0.info(()).await)
    }

    pub async fn get_config(&mut self) -> Result<VpnServiceConfig> {
        into_inner(self.0.get_config(()).await)
    }

    pub async fn set_entry_point(&mut self, entry_point: EntryPoint) -> Result<()> {
        into_inner(self.0.set_entry_point(entry_point).await)
    }

    pub async fn set_exit_point(&mut self, exit_point: ExitPoint) -> Result<()> {
        into_inner(self.0.set_exit_point(exit_point).await)
    }

    pub async fn set_disable_ipv6(&mut self, disable_ipv6: bool) -> Result<()> {
        into_inner(self.0.set_disable_ipv6(disable_ipv6).await)
    }

    pub async fn set_enable_two_hop(&mut self, enable_two_hop: bool) -> Result<()> {
        into_inner(self.0.set_enable_two_hop(enable_two_hop).await)
    }

    pub async fn set_enable_ad_blocking(&mut self, enable_ad_blocking: bool) -> Result<()> {
        into_inner(self.0.set_enable_ad_blocking(enable_ad_blocking).await)
    }

    pub async fn set_enable_conflict_detection(
        &mut self,
        enable_conflict_detection: bool,
    ) -> Result<()> {
        into_inner(
            self.0
                .set_enable_conflict_detection(enable_conflict_detection)
                .await,
        )
    }

    pub async fn set_netstack(&mut self, netstack: bool) -> Result<()> {
        into_inner(self.0.set_netstack(netstack).await)
    }

    pub async fn set_allow_lan(&mut self, allow_lan: bool) -> Result<()> {
        into_inner(self.0.set_allow_lan(allow_lan).await)
    }

    pub async fn set_bridge_mode(&mut self, bridge_mode: BridgeMode) -> Result<()> {
        into_inner(self.0.set_bridge_mode(bridge_mode).await)
    }

    pub async fn set_residential_exit(&mut self, residential_exit: bool) -> Result<()> {
        into_inner(self.0.set_residential_exit(residential_exit).await)
    }

    pub async fn set_enable_custom_dns(&mut self, enable: bool) -> Result<()> {
        into_inner(self.0.set_enable_custom_dns(enable).await)
    }

    pub async fn set_custom_dns(&mut self, ips: Vec<IpAddr>) -> Result<()> {
        into_inner(self.0.set_custom_dns(ips).await)
    }

    pub async fn set_mixnet_traffic_config(
        &mut self,
        mixnet_traffic: MixnetTrafficConfig,
    ) -> Result<()> {
        into_inner(self.0.set_mixnet_traffic_config(mixnet_traffic).await)
    }

    pub async fn set_enable_geo_location(&mut self, enable_geo_location: bool) -> Result<()> {
        into_inner(self.0.set_enable_geo_location(enable_geo_location).await)
    }

    pub async fn set_enable_gateway_independence(
        &mut self,
        enable_gateway_independence: bool,
    ) -> Result<()> {
        into_inner(
            self.0
                .set_enable_gateway_independence(enable_gateway_independence)
                .await,
        )
    }

    pub async fn set_gateway_independence_notifications(
        &mut self,
        enable_notifications: bool,
    ) -> Result<()> {
        into_inner(
            self.0
                .set_gateway_independence_notifications(enable_notifications)
                .await,
        )
    }

    pub async fn set_network(&mut self, network: String) -> Result<()> {
        into_inner(self.0.set_network(network).await)
    }

    pub async fn set_fronting_mode(&mut self, fronting_mode: FrontingMode) -> Result<()> {
        into_inner(self.0.set_fronting_mode(fronting_mode).await)
    }

    pub async fn get_system_messages(&mut self) -> Result<Vec<SystemMessage>> {
        into_inner(self.0.get_system_messages(()).await)
    }

    pub async fn get_app_compatibility(&mut self) -> Result<AppCompatibility> {
        into_inner(self.0.get_app_compatibility(()).await).map(|compatibility| {
            compatibility.unwrap_or_else(|| AppCompatibility {
                network: None,
                app_update_policy: String::new(),
                app_update_policies: HashMap::new(),
            })
        })
    }

    pub async fn get_feature_flags(&mut self) -> Result<FeatureFlags> {
        into_inner(self.0.get_feature_flags(()).await)
    }

    pub async fn get_default_dns(&mut self) -> Result<Vec<IpAddr>> {
        into_inner(self.0.get_default_dns(()).await)
    }

    pub async fn connect_tunnel(&mut self) -> Result<bool> {
        into_inner(self.0.connect_tunnel(()).await)
    }

    pub async fn reconnect_tunnel(&mut self) -> Result<bool> {
        into_inner(self.0.reconnect_tunnel(()).await)
    }

    pub async fn disconnect_tunnel(&mut self) -> Result<bool> {
        into_inner(self.0.disconnect_tunnel(()).await)
    }

    pub async fn get_tunnel_state(&mut self) -> Result<TunnelState> {
        into_inner(self.0.get_tunnel_state(()).await)
    }

    pub async fn listen_to_events(
        &mut self,
    ) -> Result<impl Stream<Item = Result<TunnelEvent>> + 'static> {
        into_inner(self.0.listen_to_events(()).await)
            .map(|events| events.map(|event| event.map_err(Error::Rpc)))
    }

    pub async fn list_gateways(&mut self, options: ListGatewaysOptions) -> Result<Vec<Gateway>> {
        into_inner(self.0.list_gateways(options).await)
    }

    pub async fn list_filtered_gateways(
        &mut self,
        filters: LookupGatewayFilters,
    ) -> Result<Vec<Gateway>> {
        into_inner(self.0.list_filtered_gateways(filters).await)
    }

    pub async fn store_account(
        &mut self,
        store_request: StoreAccountRequest,
    ) -> Result<AccountCommandResponse> {
        into_inner(self.0.store_account(store_request).await)
    }

    pub async fn is_account_stored(&mut self) -> Result<bool> {
        into_inner(self.0.is_account_stored(()).await)
    }

    pub async fn forget_account(&mut self) -> Result<AccountCommandResponse> {
        into_inner(self.0.forget_account(()).await)
    }

    pub async fn rotate_keys(&mut self) -> Result<AccountCommandResponse> {
        into_inner(self.0.rotate_keys(()).await)
    }

    pub async fn get_account_identity(&mut self) -> Result<Option<String>> {
        into_inner(self.0.get_account_identity(()).await)
    }

    pub async fn get_canonical_account_identity(&mut self) -> Result<Option<String>> {
        into_inner(self.0.get_canonical_account_identity(()).await)
    }

    pub async fn get_account_mode(&mut self) -> Result<Option<StoredAccountMode>> {
        into_inner(self.0.get_account_mode(()).await)
    }

    pub async fn get_account_links(&mut self, locale: String) -> Result<ParsedAccountLinks> {
        into_inner(self.0.get_account_links(locale).await)
    }

    pub async fn account_balance(&mut self) -> Result<AccountBalanceResponse> {
        into_inner(self.0.account_balance(()).await)
    }

    pub async fn decentralised_obtain_ticketbooks(&mut self) -> Result<AccountCommandResponse> {
        into_inner(self.0.decentralised_obtain_ticketbooks(()).await)
    }

    pub async fn get_account_state(&mut self) -> Result<AccountControllerState> {
        into_inner(self.0.get_account_state(()).await)
    }

    pub async fn refresh_account_state(&mut self, force: bool) -> Result<()> {
        into_inner(self.0.refresh_account_state(force).await)
    }

    pub async fn get_account_usage(&mut self) -> Result<Vec<NymVpnUsage>> {
        into_inner(self.0.get_account_usage(()).await)
    }

    pub async fn reset_device_identity(&mut self, seed: Option<Vec<u8>>) -> Result<()> {
        into_inner(self.0.reset_device_identity(seed).await)
    }

    pub async fn get_device_identity(&mut self) -> Result<Option<String>> {
        into_inner(self.0.get_device_identity(()).await)
    }

    pub async fn get_devices(&mut self) -> Result<Vec<NymVpnDevice>> {
        into_inner(self.0.get_devices(()).await)
    }

    pub async fn get_active_devices(&mut self) -> Result<Vec<NymVpnDevice>> {
        into_inner(self.0.get_active_devices(()).await)
    }

    pub async fn get_available_tickets(&mut self) -> Result<AvailableTickets> {
        into_inner(self.0.get_available_tickets(()).await)
    }

    pub async fn restock_ticketbooks(&mut self) -> Result<()> {
        into_inner(self.0.restock_ticketbooks(()).await)
    }

    pub async fn get_account_summary(&mut self) -> Result<Option<VpnAccountSummary>> {
        into_inner(self.0.get_account_summary(()).await)
    }

    pub async fn handle_subscription_payment(&mut self) -> Result<()> {
        into_inner(self.0.handle_subscription_payment(()).await)
    }

    pub async fn get_autologin_deeplink(
        &mut self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse> {
        into_inner(self.0.get_autologin_deeplink(params).await)
    }

    pub async fn get_deeplink(&mut self, params: GetDeeplinkParams) -> Result<String> {
        into_inner(self.0.get_deeplink(params).await)
    }

    pub async fn deeplink_store_account(
        &mut self,
        deeplink_callback_url: String,
    ) -> Result<AccountCommandResponse> {
        into_inner(self.0.deeplink_store_account(deeplink_callback_url).await)
    }

    pub async fn get_log_path(&mut self) -> Result<Option<LogPath>> {
        into_inner(self.0.get_log_path(()).await)
    }

    pub async fn delete_log_file(&mut self) -> Result<()> {
        into_inner(self.0.delete_log_file(()).await)
    }

    pub async fn is_sentry_enabled(&mut self) -> Result<bool> {
        into_inner(self.0.is_sentry_enabled(()).await)
    }

    pub async fn enable_sentry(&mut self) -> Result<()> {
        into_inner(self.0.enable_sentry(()).await)
    }

    pub async fn disable_sentry(&mut self) -> Result<()> {
        into_inner(self.0.disable_sentry(()).await)
    }

    pub async fn network_stats_set_enabled(&mut self, enabled: bool) -> Result<()> {
        into_inner(self.0.network_stats_set_enabled(enabled).await)
    }

    pub async fn network_stats_allow_disconnected(
        &mut self,
        allow_disconnected: bool,
    ) -> Result<()> {
        into_inner(
            self.0
                .network_stats_allow_disconnected(allow_disconnected)
                .await,
        )
    }

    pub async fn network_stats_reset_seed(&mut self, seed: Option<String>) -> Result<()> {
        into_inner(self.0.network_stats_reset_seed(seed).await)
    }

    pub async fn enable_socks5(
        &mut self,
        socks5_settings: Socks5Settings,
        http_rpc_settings: HttpRpcSettings,
        exit_point: ExitPoint,
    ) -> Result<()> {
        let request = EnableSocks5Request {
            socks5_settings,
            http_rpc_settings,
            exit_point,
        };
        into_inner(self.0.enable_socks5(request).await)
    }

    pub async fn disable_socks5(&mut self) -> Result<()> {
        into_inner(self.0.disable_socks5(()).await)
    }

    pub async fn get_socks5_status(&mut self) -> Result<Socks5Status> {
        into_inner(self.0.get_socks5_status(()).await)
    }

    pub async fn set_geo_exclusion_enabled(&mut self, enabled: bool) -> Result<()> {
        into_inner(self.0.set_geo_exclusion_enabled(enabled).await)
    }

    pub async fn set_geo_exclusion_listen_port(&mut self, listen_port: u16) -> Result<()> {
        into_inner(self.0.set_geo_exclusion_listen_port(listen_port).await)
    }

    pub async fn set_geo_exclusion_excluded_countries(
        &mut self,
        excluded_countries: Vec<String>,
    ) -> Result<()> {
        into_inner(
            self.0
                .set_geo_exclusion_excluded_countries(excluded_countries)
                .await,
        )
    }

    pub async fn network_stats_get_seed(&mut self) -> Result<NetworkStatisticsIdentity> {
        into_inner(self.0.network_stats_get_seed(()).await)
    }

    pub async fn get_privy_derivation_message(&mut self) -> Result<PrivyDerivationMessage> {
        into_inner(self.0.get_privy_derivation_message(()).await)
    }

    pub async fn run_diagnostic(
        &mut self,
        params: DiagnosticRunParams,
    ) -> Result<DiagnosticReport> {
        into_inner(self.0.run_diagnostic(params).await)
    }

    /// Returns the diagnostic report serialized as JSON.
    pub async fn run_diagnostic_raw(&mut self, params: DiagnosticRunParams) -> Result<String> {
        let report = self.run_diagnostic(params).await?;
        serde_json::to_string(&report).map_err(Error::Json)
    }

    pub async fn register_diagnostic(
        &mut self,
        params: DiagnosticRegisterParams,
    ) -> Result<RegistrationReport> {
        into_inner(self.0.register_diagnostic(params).await)
    }

    pub async fn get_tentative_gateways(&mut self) -> Result<TentativeGateways> {
        into_inner(self.0.get_tentative_gateways(()).await)
    }

    pub async fn get_recent_gateways(
        &mut self,
        params: GetRecentGatewaysParams,
    ) -> Result<RecentGateways> {
        into_inner(self.0.get_recent_gateways(params).await)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub async fn is_split_tunnel_supported(&mut self) -> Result<bool> {
        into_inner(self.0.is_split_tunnel_supported(()).await)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn set_enable_split_tunnel(&mut self, enable: bool) -> Result<()> {
        into_inner(self.0.set_enable_split_tunnel(enable).await)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn add_split_tunnel_app(&mut self, app: SplitApp) -> Result<()> {
        into_inner(self.0.add_split_tunnel_app(app).await)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn remove_split_tunnel_app(&mut self, app: SplitApp) -> Result<()> {
        into_inner(self.0.remove_split_tunnel_app(app).await)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn clear_split_tunnel_apps(&mut self) -> Result<()> {
        into_inner(self.0.clear_split_tunnel_apps(()).await)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub async fn get_split_tunnel_excluded_processes(
        &mut self,
    ) -> Result<SplitTunnelExcludedProcessList> {
        into_inner(self.0.get_split_tunnel_excluded_processes(()).await)
    }

    #[cfg(target_os = "linux")]
    pub async fn add_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        into_inner(self.0.add_split_tunnel_process(pid).await)
    }

    #[cfg(target_os = "linux")]
    pub async fn remove_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        into_inner(self.0.remove_split_tunnel_process(pid).await)
    }

    #[cfg(target_os = "linux")]
    pub async fn clear_split_tunnel_processes(&mut self) -> Result<()> {
        into_inner(self.0.clear_split_tunnel_processes(()).await)
    }

    #[cfg(target_os = "macos")]
    pub async fn need_full_disk_permissions(&mut self) -> Result<bool> {
        into_inner(self.0.need_full_disk_permissions(()).await)
    }

    pub async fn set_profile(&mut self, profile_options: ProfileOptions) -> Result<()> {
        into_inner(self.0.set_profile(profile_options).await)
    }
}

pub fn get_rpc_socket_path() -> PathBuf {
    #[cfg(unix)]
    let path = "/var/run/nym-vpn.sock";

    #[cfg(windows)]
    let path = r"\\.\pipe\nym-vpn";

    PathBuf::from(path)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),

    #[error("Rpc call returned error")]
    Rpc(#[source] tonic::Status),

    #[error("Failed to serialize response as JSON")]
    Json(#[source] serde_json::Error),

    #[cfg(target_os = "linux")]
    #[error(
        "Authentication is required to access the daemon. Consider adding your user to the nym-vpn group: usermod -aG nym-vpn \"$USER\" (needs root permissions)"
    )]
    AuthenticationRequired,

    #[cfg(not(target_os = "linux"))]
    #[error("Authentication is required to access the daemon.")]
    AuthenticationRequired,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
