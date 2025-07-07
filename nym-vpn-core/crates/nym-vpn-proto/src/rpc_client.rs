// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_api_client::NetworkCompatibility;
use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use nym_vpn_network_config::{FeatureFlags, SystemMessages};
use nym_vpnd_types::{
    ConnectArgs, ListCountriesOptions, ListGatewaysOptions,
    gateway::{Country, Gateway},
    log_path::LogPath,
    service::VpnServiceInfo,
};
use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

use crate::{
    ConnectRequest, ListCountriesRequest, ListGatewaysRequest, nym_vpnd_client::NymVpndClient,
};

type ServiceClient = NymVpndClient<tonic::transport::Channel>;

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
        Ok(self
            .0
            .set_network(network)
            .await
            .map_err(Error::Rpc)?
            .into_inner())
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

    pub async fn connect_tunnel(&mut self, request: ConnectArgs) -> Result<bool> {
        let connect_req = ConnectRequest::try_from(request).map_err(Error::InvalidRequest)?;

        let is_accepted = self
            .0
            .vpn_connect(connect_req)
            .await
            .map(|v| v.into_inner())
            .map_err(Error::Rpc)?;

        Ok(is_accepted)
    }

    pub async fn disconnect_tunnel(&mut self) -> Result<bool> {
        self.0
            .vpn_disconnect(())
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
    ) -> Result<impl Stream<Item = Result<TunnelState>>> {
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

    pub async fn listen_to_events(&mut self) -> Result<impl Stream<Item = Result<TunnelEvent>>> {
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
        let request = ListGatewaysRequest::try_from(options).map_err(Error::InvalidRequest)?;

        let gateways = self
            .0
            .list_gateways(request)
            .await
            .map(|v| v.into_inner().gateways)
            .map_err(Error::Rpc)?;

        Ok(gateways
            .into_iter()
            .map(|gateway| Gateway::try_from(gateway).map_err(Error::InvalidResponse))
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn list_countries(&mut self, options: ListCountriesOptions) -> Result<Vec<Country>> {
        let request = ListCountriesRequest::try_from(options).map_err(Error::InvalidRequest)?;

        let countries = self
            .0
            .list_countries(request)
            .await
            .map(|v| v.into_inner().countries)
            .map_err(Error::Rpc)?;

        Ok(countries.into_iter().map(Country::from).collect())
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

    pub async fn delete_log_file(&mut self) -> Result<bool> {
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

    #[error("GRPC call returned error")]
    Rpc(#[source] tonic::Status),

    #[error("Failed to serialize gRPC request")]
    InvalidRequest(#[source] crate::conversions::ConversionError),

    #[error("Failed to parse gRPC response")]
    InvalidResponse(#[source] crate::conversions::ConversionError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
