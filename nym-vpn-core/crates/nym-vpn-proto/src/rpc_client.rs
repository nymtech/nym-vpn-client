// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AccountBalanceResponse, AccountCommandResponse, AccountControllerState, AvailableTickets,
    ConnectArgs, EntryPoint, ExitPoint, FeatureFlags, Gateway, GatewayFilters, HttpRpcSettings,
    ListGatewaysOptions, LogPath, NetworkCompatibility, NymVpnDevice, NymVpnUsage,
    ParsedAccountLinks, Socks5Settings, Socks5Status, StoreAccountRequest, SystemMessage,
    TunnelEvent, TunnelState, VpnServiceConfig, VpnServiceInfo,
};
use std::{net::IpAddr, path::PathBuf};
use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

use crate::proto::{self, nym_vpn_service_client::NymVpnServiceClient};

type ServiceClient = NymVpnServiceClient<tonic::transport::Channel>;

#[derive(Debug, Clone)]
pub struct RpcClient(ServiceClient);

impl RpcClient {
    pub async fn new() -> Result<RpcClient> {
        let socket_path = get_rpc_socket_path();
        let channel = Endpoint::from_static("unix://placeholder")
            .connect_with_connector(service_fn(move |_: Uri| {
                nym_ipc::client::connect(socket_path.clone())
            }))
            .await?;
        Ok(RpcClient(ServiceClient::new(channel)))
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

    pub async fn set_custom_dns(&mut self, ips: Option<Vec<IpAddr>>) -> Result<()> {
        let request: proto::IpAddrList = ips.into();

        self.0
            .set_custom_dns(request)
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

    pub async fn connect_tunnel(&mut self, request: ConnectArgs) -> Result<()> {
        let request = proto::ConnectRequest::try_from(request).map_err(Error::InvalidRequest)?;

        self.0
            .connect_tunnel(request)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn connect_tunnel_v2(&mut self) -> Result<bool> {
        self.0
            .connect_tunnel_v2(())
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
        filters: GatewayFilters,
    ) -> Result<Vec<Gateway>> {
        let request = proto::GatewayFilters::from(filters);

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

    pub async fn decentralised_obtain_ticketbooks(
        &mut self,
        amount: u64,
    ) -> Result<AccountCommandResponse> {
        let request = proto::DecentralisedObtainTicketbooksRequest { amount };
        let response = self
            .0
            .decentralised_obtain_ticketbooks(request)
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

    pub async fn refresh_account_state(&mut self) -> Result<()> {
        self.0
            .refresh_account_state(())
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

    pub async fn get_log_path(&mut self) -> Result<LogPath> {
        let response = self
            .0
            .get_log_path(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;

        Ok(LogPath::from(response))
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

    pub async fn is_collect_network_stats_enabled(&mut self) -> Result<bool> {
        self.0
            .is_collect_network_stats_enabled(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn enable_collect_network_stats(&mut self) -> Result<()> {
        self.0
            .enable_collect_network_stats(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn disable_collect_network_stats(&mut self) -> Result<()> {
        self.0
            .disable_collect_network_stats(())
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
                listen_address: socks5_settings.listen_address,
            }),
            http_rpc_settings: Some(proto::HttpRpcSettings {
                listen_address: http_rpc_settings.listen_address,
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

    #[error("Failed to parse rpc response")]
    InvalidResponse(#[source] crate::conversions::ConversionError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
