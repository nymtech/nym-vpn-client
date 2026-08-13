// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error as _, net::IpAddr, path::PathBuf};

use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::SplitApp;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use nym_vpn_lib_types::SplitTunnelExcludedProcessList;
use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandResponse, AccountControllerState, AutologinResponse,
    AvailableTickets, DiagnosticReport, EntryPoint, ExitPoint, FeatureFlags, FrontingMode, Gateway,
    GetDeeplinkParams, HttpRpcSettings, ListGatewaysOptions, LogPath, LookupGatewayFilters,
    NetworkCompatibility, NetworkStatisticsIdentity, NymVpnDevice, NymVpnUsage, ParsedAccountLinks,
    PrivyDerivationMessage, ProfileOptions, RecentGateways, RegistrationReport, Socks5Settings,
    Socks5Status, StoreAccountRequest, StoredAccountMode, SystemMessage, TentativeGateways,
    TunnelEvent, TunnelState, VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};

use crate::proto::{self, nym_vpn_service_client::NymVpnServiceClient};

type ServiceClient = NymVpnServiceClient<tonic::transport::Channel>;

#[derive(Debug, Clone)]
pub struct RpcClient(ServiceClient);

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
        let response = self.0.info(()).await.map_err(Error::Rpc)?.into_inner();

        VpnServiceInfo::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn get_config(&mut self) -> Result<VpnServiceConfig> {
        let response = self
            .0
            .get_config(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        let config = response
            .config
            .ok_or_else(|| Error::Rpc(tonic::Status::internal("Missing config in response")))?;

        VpnServiceConfig::try_from(config).map_err(Error::InvalidResponse)
    }

    pub async fn set_entry_point(&mut self, entry_point: EntryPoint) -> Result<()> {
        let entry_node = proto::EntryNode::from(entry_point);

        self.0
            .set_entry_point(entry_node)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(())
    }

    pub async fn set_exit_point(&mut self, exit_point: ExitPoint) -> Result<()> {
        let exit_node = proto::ExitNode::from(exit_point);

        self.0
            .set_exit_point(exit_node)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(())
    }

    pub async fn set_disable_ipv6(&mut self, disable_ipv6: bool) -> Result<()> {
        self.0
            .set_disable_ipv6(disable_ipv6)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_enable_two_hop(&mut self, enable_two_hop: bool) -> Result<()> {
        self.0
            .set_enable_two_hop(enable_two_hop)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_enable_ad_blocking(&mut self, enable_ad_blocking: bool) -> Result<()> {
        self.0
            .set_enable_ad_blocking(enable_ad_blocking)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_netstack(&mut self, netstack: bool) -> Result<()> {
        self.0
            .set_netstack(netstack)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_allow_lan(&mut self, allow_lan: bool) -> Result<()> {
        self.0
            .set_allow_lan(allow_lan)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_enable_bridges(&mut self, enable_bridges: bool) -> Result<()> {
        self.0
            .set_enable_bridges(enable_bridges)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_residential_exit(&mut self, residential_exit: bool) -> Result<()> {
        self.0
            .set_residential_exit(residential_exit)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_enable_custom_dns(&mut self, enable: bool) -> Result<()> {
        self.0
            .set_enable_custom_dns(enable)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_custom_dns(&mut self, ips: Vec<IpAddr>) -> Result<()> {
        let request = proto::IpAddrList {
            ips: ips.into_iter().map(proto::IpAddr::from).collect(),
        };

        self.0
            .set_custom_dns(request)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_mixnet_traffic_config(
        &mut self,
        mixnet_traffic: nym_vpn_lib_types::MixnetTrafficConfig,
    ) -> Result<()> {
        let request = proto::MixnetTrafficConfig::from(mixnet_traffic);

        self.0
            .set_mixnet_traffic_config(request)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_enable_geo_location(&mut self, enable_geo_location: bool) -> Result<()> {
        self.0
            .set_enable_geo_location(enable_geo_location)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_enable_gateway_independence(
        &mut self,
        enable_gateway_independence: bool,
    ) -> Result<()> {
        self.0
            .set_enable_gateway_independence(enable_gateway_independence)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_gateway_independence_notifications(
        &mut self,
        enable_notifications: bool,
    ) -> Result<()> {
        self.0
            .set_gateway_independence_notifications(enable_notifications)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_network(&mut self, network: String) -> Result<()> {
        self.0
            .set_network(network)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn set_fronting_mode(&mut self, fronting_mode: FrontingMode) -> Result<()> {
        self.0
            .set_fronting_mode(proto::FrontingModeRequest {
                mode: proto::FrontingModes::from(fronting_mode).into(),
            })
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn get_system_messages(&mut self) -> Result<Vec<SystemMessage>> {
        let response = self
            .0
            .get_system_messages(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        let messages = response
            .messages
            .into_iter()
            .map(SystemMessage::from)
            .collect::<Vec<_>>();

        Ok(messages)
    }

    pub async fn get_network_compatibility(&mut self) -> Result<Option<NetworkCompatibility>> {
        let response = self
            .0
            .get_network_compatibility(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(response
            .network_compatibility
            .map(NetworkCompatibility::from))
    }

    pub async fn get_feature_flags(&mut self) -> Result<FeatureFlags> {
        let response = self
            .0
            .get_feature_flags(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(FeatureFlags::from(response))
    }

    pub async fn get_default_dns(&mut self) -> Result<Vec<IpAddr>> {
        let response = self
            .0
            .get_default_dns(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        let ip_vec = response.try_into().map_err(Error::InvalidResponse)?;
        Ok(ip_vec)
    }

    pub async fn connect_tunnel(&mut self) -> Result<bool> {
        self.0
            .connect_tunnel(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn reconnect_tunnel(&mut self) -> Result<bool> {
        self.0
            .reconnect_tunnel(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn disconnect_tunnel(&mut self) -> Result<bool> {
        self.0
            .disconnect_tunnel(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn get_tunnel_state(&mut self) -> Result<TunnelState> {
        let state = self
            .0
            .get_tunnel_state(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        TunnelState::try_from(state).map_err(Error::InvalidResponse)
    }

    pub async fn listen_to_events(
        &mut self,
    ) -> Result<impl Stream<Item = Result<TunnelEvent>> + 'static> {
        let listener = self
            .0
            .listen_to_events(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(listener.map(|item| {
            item.map_err(Error::Rpc).and_then(|daemon_event| {
                TunnelEvent::try_from(daemon_event).map_err(Error::InvalidResponse)
            })
        }))
    }

    pub async fn list_gateways(&mut self, options: ListGatewaysOptions) -> Result<Vec<Gateway>> {
        let request =
            proto::ListGatewaysRequest::try_from(options).map_err(Error::InvalidRequest)?;

        let gateways = self
            .0
            .list_gateways(request)
            .await
            .map(|v| v.into_inner().gateways)
            .map_err(Error::Rpc)?;

        gateways
            .into_iter()
            .map(|gateway| Gateway::try_from(gateway).map_err(Error::InvalidResponse))
            .collect::<Result<Vec<_>>>()
    }

    pub async fn list_filtered_gateways(
        &mut self,
        filters: LookupGatewayFilters,
    ) -> Result<Vec<Gateway>> {
        let request = proto::LookupGatewayFilters::from(filters);

        let gateways = self
            .0
            .list_filtered_gateways(request)
            .await
            .map(|v| v.into_inner().gateways)
            .map_err(Error::Rpc)?;

        gateways
            .into_iter()
            .map(|gateway| Gateway::try_from(gateway).map_err(Error::InvalidResponse))
            .collect::<Result<Vec<_>>>()
    }

    pub async fn store_account(
        &mut self,
        store_request: StoreAccountRequest,
    ) -> Result<AccountCommandResponse> {
        let request = proto::StoreAccountRequest::from(store_request);
        let response = self
            .0
            .store_account(request)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn is_account_stored(&mut self) -> Result<bool> {
        self.0
            .is_account_stored(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn forget_account(&mut self) -> Result<AccountCommandResponse> {
        let response = self
            .0
            .forget_account(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn rotate_keys(&mut self) -> Result<AccountCommandResponse> {
        let response = self
            .0
            .rotate_keys(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn get_account_identity(&mut self) -> Result<Option<String>> {
        self.0
            .get_account_identity(())
            .await
            .map(|v| v.into_inner().account_identity)
            .map_err(Error::Rpc)
    }

    pub async fn get_canonical_account_identity(&mut self) -> Result<Option<String>> {
        self.0
            .get_canonical_account_identity(())
            .await
            .map(|v| v.into_inner().account_identity)
            .map_err(Error::Rpc)
    }

    pub async fn get_account_mode(&mut self) -> Result<Option<StoredAccountMode>> {
        let response = self
            .0
            .get_account_mode(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;

        let opt_mode: Option<StoredAccountMode> =
            response.try_into().map_err(Error::InvalidResponse)?;

        Ok(opt_mode)
    }

    pub async fn get_account_links(&mut self, locale: String) -> Result<ParsedAccountLinks> {
        let request = proto::GetAccountLinksRequest { locale };
        let response = self
            .0
            .get_account_links(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;

        Ok(ParsedAccountLinks::from(response))
    }

    pub async fn account_balance(&mut self) -> Result<AccountBalanceResponse> {
        let response = self
            .0
            .account_balance(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountBalanceResponse::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn decentralised_obtain_ticketbooks(&mut self) -> Result<AccountCommandResponse> {
        let response = self
            .0
            .decentralised_obtain_ticketbooks(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn get_account_state(&mut self) -> Result<AccountControllerState> {
        let state = self
            .0
            .get_account_state(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountControllerState::try_from(state).map_err(Error::InvalidResponse)
    }

    pub async fn refresh_account_state(&mut self, force: bool) -> Result<()> {
        self.0
            .refresh_account_state(proto::RefreshAccountStateRequest { force })
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn get_account_usage(&mut self) -> Result<Vec<NymVpnUsage>> {
        let response = self
            .0
            .get_account_usage(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(response.account_usages.map(Vec::from).unwrap_or_default())
    }

    pub async fn reset_device_identity(&mut self, seed: Option<Vec<u8>>) -> Result<()> {
        let request = proto::ResetDeviceIdentityRequest { seed };
        self.0
            .reset_device_identity(request)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn get_device_identity(&mut self) -> Result<Option<String>> {
        let response = self
            .0
            .get_device_identity(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(response.device_identity)
    }

    pub async fn get_devices(&mut self) -> Result<Vec<NymVpnDevice>> {
        let response = self
            .0
            .get_devices(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        let devices = response
            .devices
            .unwrap_or_default()
            .devices
            .into_iter()
            .map(NymVpnDevice::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::InvalidResponse)?;

        Ok(devices)
    }

    pub async fn get_active_devices(&mut self) -> Result<Vec<NymVpnDevice>> {
        let response = self
            .0
            .get_active_devices(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        let devices = response
            .devices
            .unwrap_or_default()
            .devices
            .into_iter()
            .map(NymVpnDevice::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::InvalidResponse)?;

        Ok(devices)
    }

    pub async fn get_available_tickets(&mut self) -> Result<AvailableTickets> {
        let response = self
            .0
            .get_available_tickets(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(AvailableTickets::from(response))
    }

    pub async fn restock_ticketbooks(&mut self) -> Result<()> {
        self.0.restock_ticketbooks(()).await.map_err(Error::Rpc)?;

        Ok(())
    }

    pub async fn get_account_summary(&mut self) -> Result<Option<VpnAccountSummary>> {
        let response = self
            .0
            .get_account_summary(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        let account_summary = response
            .account_summary
            .map(VpnAccountSummary::try_from)
            .transpose()
            .map_err(Error::InvalidResponse)?;

        Ok(account_summary)
    }

    pub async fn handle_subscription_payment(&mut self) -> Result<()> {
        self.0
            .handle_subscription_payment(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn get_autologin_deeplink(
        &mut self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse> {
        let request: proto::GetDeeplinkParams = params.into();
        let response = self
            .0
            .get_autologin_deeplink(request)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        let deeplink = response.try_into().map_err(Error::InvalidResponse)?;

        Ok(deeplink)
    }

    pub async fn get_deeplink(&mut self, params: GetDeeplinkParams) -> Result<String> {
        let request: proto::GetDeeplinkParams = params.into();
        let url = self
            .0
            .get_deeplink(request)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(url)
    }

    pub async fn deeplink_store_account(
        &mut self,
        deeplink_callback_url: String,
    ) -> Result<AccountCommandResponse> {
        let response = self
            .0
            .deeplink_store_account(deeplink_callback_url)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        AccountCommandResponse::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn get_log_path(&mut self) -> Result<Option<LogPath>> {
        let response = self
            .0
            .get_log_path(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;

        Ok(response.log_path.map(LogPath::from))
    }

    pub async fn delete_log_file(&mut self) -> Result<()> {
        self.0
            .delete_log_file(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn is_sentry_enabled(&mut self) -> Result<bool> {
        self.0
            .is_sentry_enabled(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn enable_sentry(&mut self) -> Result<()> {
        self.0
            .enable_sentry(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn disable_sentry(&mut self) -> Result<()> {
        self.0
            .disable_sentry(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn network_stats_set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.0
            .network_stats_set_enabled(enabled)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn network_stats_allow_disconnected(
        &mut self,
        allow_disconnected: bool,
    ) -> Result<()> {
        self.0
            .network_stats_allow_disconnected(allow_disconnected)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn network_stats_reset_seed(&mut self, seed: Option<String>) -> Result<()> {
        let request = proto::NetworkStatsResetSeedRequest { seed };
        self.0
            .network_stats_reset_seed(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn enable_socks5(
        &mut self,
        socks5_settings: Socks5Settings,
        http_rpc_settings: HttpRpcSettings,
        exit_point: ExitPoint,
    ) -> Result<()> {
        let request = proto::EnableSocks5Request {
            socks5_settings: Some(proto::Socks5Settings {
                listen_address: match socks5_settings.listen_address {
                    Some(addr) => addr.to_string(),
                    None => String::new(),
                },
            }),
            http_rpc_settings: Some(proto::HttpRpcSettings {
                listen_address: match http_rpc_settings.listen_address {
                    Some(addr) => addr.to_string(),
                    None => String::new(),
                },
            }),
            exit: Some(proto::ExitNode::from(exit_point)),
        };

        self.0
            .enable_socks5(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn disable_socks5(&mut self) -> Result<()> {
        self.0
            .disable_socks5(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn get_socks5_status(&mut self) -> Result<Socks5Status> {
        let response = self
            .0
            .get_socks5_status(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Socks5Status::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn set_geo_exclusion_enabled(&mut self, enabled: bool) -> Result<()> {
        self.0
            .set_geo_exclusion_enabled(enabled)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn set_geo_exclusion_listen_port(&mut self, listen_port: u16) -> Result<()> {
        self.0
            .set_geo_exclusion_listen_port(proto::GeoExclusionListenPortRequest {
                listen_port: listen_port as u32,
            })
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn set_geo_exclusion_excluded_countries(
        &mut self,
        excluded_countries: Vec<String>,
    ) -> Result<()> {
        self.0
            .set_geo_exclusion_excluded_countries(proto::StringList {
                values: excluded_countries,
            })
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn network_stats_get_seed(&mut self) -> Result<NetworkStatisticsIdentity> {
        let response = self
            .0
            .network_stats_get_seed(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;
        Ok(NetworkStatisticsIdentity::from(response))
    }

    pub async fn get_privy_derivation_message(&mut self) -> Result<PrivyDerivationMessage> {
        let response = self
            .0
            .get_privy_derivation_message(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;

        Ok(PrivyDerivationMessage::from(response))
    }

    pub async fn run_diagnostic(
        &mut self,
        params: nym_vpn_lib_types::DiagnosticRunParams,
    ) -> Result<DiagnosticReport> {
        let request = proto::DiagnosticRunParams::from(params);
        let response = self
            .0
            .run_diagnostic(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;
        DiagnosticReport::try_from(response).map_err(Error::InvalidResponse)
    }

    /// Returns the diagnostic report as a raw JSON string, bypassing
    /// deserialization into the `DiagnosticReport` type.
    pub async fn run_diagnostic_raw(
        &mut self,
        params: nym_vpn_lib_types::DiagnosticRunParams,
    ) -> Result<String> {
        let request = proto::DiagnosticRunParams::from(params);
        let response = self
            .0
            .run_diagnostic(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;
        Ok(response.json)
    }

    pub async fn register_diagnostic(
        &mut self,
        params: nym_vpn_lib_types::DiagnosticRegisterParams,
    ) -> Result<RegistrationReport> {
        let request = proto::DiagnosticRegisterParams::from(params);
        let response = self
            .0
            .register_diagnostic(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;
        RegistrationReport::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn get_tentative_gateways(&mut self) -> Result<TentativeGateways> {
        let response = self
            .0
            .get_tentative_gateways(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;
        TentativeGateways::try_from(response).map_err(Error::InvalidResponse)
    }

    pub async fn get_recent_gateways(
        &mut self,
        params: nym_vpn_lib_types::GetRecentGatewaysParams,
    ) -> Result<RecentGateways> {
        let request = proto::GetRecentGatewaysParams::from(params);
        let response = self
            .0
            .get_recent_gateways(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;
        RecentGateways::try_from(response).map_err(Error::InvalidResponse)
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    pub async fn is_split_tunnel_supported(&mut self) -> Result<bool> {
        self.0
            .is_split_tunnel_supported(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn set_enable_split_tunnel(&mut self, enable: bool) -> Result<()> {
        self.0
            .set_enable_split_tunnel(enable)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn add_split_tunnel_app(&mut self, app: SplitApp) -> Result<()> {
        self.0
            .add_split_tunnel_app(proto::SplitApp::from(app))
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn remove_split_tunnel_app(&mut self, app: SplitApp) -> Result<()> {
        self.0
            .remove_split_tunnel_app(proto::SplitApp::from(app))
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn clear_split_tunnel_apps(&mut self) -> Result<()> {
        self.0
            .clear_split_tunnel_apps(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub async fn get_split_tunnel_excluded_processes(
        &mut self,
    ) -> Result<SplitTunnelExcludedProcessList> {
        self.0
            .get_split_tunnel_excluded_processes(())
            .await
            .map(|v| SplitTunnelExcludedProcessList::from(v.into_inner()))
            .map_err(Error::Rpc)
    }

    #[cfg(target_os = "linux")]
    pub async fn add_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.0
            .add_split_tunnel_process(pid)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(target_os = "linux")]
    pub async fn remove_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.0
            .remove_split_tunnel_process(pid)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(target_os = "linux")]
    pub async fn clear_split_tunnel_processes(&mut self) -> Result<()> {
        self.0
            .clear_split_tunnel_processes(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    #[cfg(target_os = "macos")]
    pub async fn need_full_disk_permissions(&mut self) -> Result<bool> {
        self.0
            .need_full_disk_permissions(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn set_profile(&mut self, profile_options: ProfileOptions) -> Result<()> {
        let profile_options_proto =
            proto::ProfileOptions::try_from(profile_options).map_err(Error::InvalidRequest)?;

        self.0
            .set_profile(profile_options_proto)
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(())
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

    #[error("Failed to serialize rpc request")]
    InvalidRequest(#[source] crate::conversions::ConversionError),

    #[error("Failed to parse rpc response: {0}")]
    InvalidResponse(#[source] crate::conversions::ConversionError),

    #[error(
        "Authentication is required to access the daemon. Consider adding your user to the nym-vpn group: usermod -aG nym-vpn \"$USER\" (needs root permissions)"
    )]
    AuthenticationRequired,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
