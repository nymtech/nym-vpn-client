// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_api_client::{
    NetworkCompatibility,
    response::{NymVpnDevice, NymVpnUsage},
};
use nym_vpn_lib_types::{AccountControllerState, AvailableTickets, TunnelEvent, TunnelState};
use nym_vpn_network_config::{FeatureFlags, ParsedAccountLinks, SystemMessages};
use nym_vpnd_types::{
    AccountCommandResponse, ListCountriesOptions, ListGatewaysOptions, StoreAccountRequest,
    gateway::{Country, Gateway},
    log_path::LogPath,
    service::VpnServiceInfo,
};
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

    pub async fn set_network(&mut self, network: String) -> Result<()> {
        self.0
            .set_network(network)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(())
    }

    pub async fn get_system_messages(&mut self) -> Result<SystemMessages> {
        let response = self
            .0
            .get_system_messages(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(SystemMessages::from(response))
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

    pub async fn connect_tunnel(&mut self) -> Result<()> {
        self.0
            .connect_tunnel(())
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)
    }

    pub async fn disconnect_tunnel(&mut self) -> Result<()> {
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

    pub async fn listen_to_tunnel_state(
        &mut self,
    ) -> Result<impl Stream<Item = Result<TunnelState>> + 'static> {
        let listener = self
            .0
            .listen_to_tunnel_state(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(listener.map(|item| {
            item.map_err(Error::Rpc).and_then(|tunnel_state| {
                TunnelState::try_from(tunnel_state).map_err(Error::InvalidResponse)
            })
        }))
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

    pub async fn list_countries(&mut self, options: ListCountriesOptions) -> Result<Vec<Country>> {
        let request =
            proto::ListCountriesRequest::try_from(options).map_err(Error::InvalidRequest)?;

        let countries = self
            .0
            .list_countries(request)
            .await
            .map(|v| v.into_inner().countries)
            .map_err(Error::Rpc)?;

        Ok(countries.into_iter().map(Country::from).collect())
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

        ParsedAccountLinks::try_from(response).map_err(Error::InvalidResponse)
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

    pub async fn listen_to_account_controller_state(
        &mut self,
    ) -> Result<impl Stream<Item = Result<AccountControllerState>> + 'static> {
        let listener = self
            .0
            .listen_to_account_state(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(listener.map(|item| {
            item.map_err(Error::Rpc).and_then(|account_state| {
                AccountControllerState::try_from(account_state).map_err(Error::InvalidResponse)
            })
        }))
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
}

pub fn get_rpc_socket_path() -> PathBuf {
    #[cfg(unix)]
    return PathBuf::from("/var/run/nym-vpn.sock");

    #[cfg(windows)]
    return PathBuf::from(r"\\.\pipe\nym-vpn");
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
