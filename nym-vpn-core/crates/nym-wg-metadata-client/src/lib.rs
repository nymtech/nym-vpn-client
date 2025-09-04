// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_credentials_interface::CredentialSpendingData;
use nym_gateway_directory::NodeIdentity;
use nym_http_api_client::ReqwestClientBuilder;
use nym_wireguard_private_metadata_client::WireguardMetadataApiClient;
use nym_wireguard_private_metadata_shared::{Version, v1};
use tokio::sync::OnceCell;
use url::Url;

use error::Result;

use crate::error::MetadataClientError;

pub mod error;

pub struct TunUpEvent {
    pub entry: TunUpSender,
    pub exit: TunUpSender,
}

#[derive(Clone)]
pub enum TunUpSendData {
    InterfaceName(String),
    Signal,
}

pub type TunUpSender = tokio::sync::oneshot::Sender<TunUpSendData>;
pub type TunUpReceiver = tokio::sync::oneshot::Receiver<TunUpSendData>;

#[derive(Debug, Clone)]
struct LazyMetadataClient {
    inner: nym_http_api_client::Client,
    version: Version,
}

impl LazyMetadataClient {
    async fn new(base_url: Url, bind_ip: IpAddr, sent_data: TunUpSendData) -> Result<Self> {
        let reqwest_builder = ReqwestClientBuilder::new().local_address(bind_ip);
        let reqwest_builder = match sent_data {
            #[cfg(not(target_os = "windows"))]
            TunUpSendData::InterfaceName(interface) => reqwest_builder.interface(&interface),
            _ => reqwest_builder,
        };
        let inner = nym_http_api_client::Client::builder(base_url)
            .and_then(|builder| builder.with_reqwest_builder(reqwest_builder).build())?;
        let version = inner.version().await?;

        Ok(Self { inner, version })
    }
}

pub struct MetadataClient {
    lazy_client: OnceCell<Result<LazyMetadataClient>>,
    gateway_id: NodeIdentity,
    base_url: Url,
    bind_ip: IpAddr,
    signal_channel: Option<TunUpReceiver>,
}

impl MetadataClient {
    async fn lazy_client(&mut self) -> &Result<LazyMetadataClient> {
        self.lazy_client
            .get_or_init(|| async {
                let data = self
                    .signal_channel
                    .take()
                    .ok_or(MetadataClientError::Internal(
                        "signal channel already consumed".to_string(),
                    ))?
                    .await
                    .map_err(|_| {
                        MetadataClientError::Internal("interface up signal never sent".to_string())
                    })?;
                LazyMetadataClient::new(self.base_url.clone(), self.bind_ip, data).await
            })
            .await
    }

    pub fn new(
        base_url: Url,
        gateway_id: NodeIdentity,
        bind_ip: IpAddr,
        signal_channel: TunUpReceiver,
    ) -> Self {
        Self {
            lazy_client: OnceCell::new(),
            gateway_id,
            bind_ip,
            base_url,
            signal_channel: Some(signal_channel),
        }
    }

    pub fn gateway_id(&self) -> NodeIdentity {
        self.gateway_id
    }

    fn print_remaining_bandwidth(gateway_id: NodeIdentity, available_bandwidth: i64) {
        let remaining_pretty = if available_bandwidth > 1024 * 1024 {
            format!("{:.2} MB", available_bandwidth as f64 / 1024.0 / 1024.0)
        } else {
            format!("{} KB", available_bandwidth / 1024)
        };
        tracing::debug!(
            "Remaining wireguard bandwidth with gateway {} for today: {}",
            gateway_id,
            remaining_pretty
        );
    }

    pub async fn query_bandwidth(&mut self) -> Result<i64> {
        let client = self
            .lazy_client()
            .await
            .as_ref()
            .map_err(|err| MetadataClientError::Internal(err.to_string()))?;
        let request = match client.version {
            Version::V1 => v1::AvailableBandwidthRequest {}.try_into()?,
        };
        let response = client.inner.available_bandwidth(&request).await?;
        let available_bandwidth = match client.version {
            Version::V1 => v1::AvailableBandwidthResponse::try_from(response)?.available_bandwidth,
        };
        Self::print_remaining_bandwidth(self.gateway_id, available_bandwidth);
        Ok(available_bandwidth)
    }

    pub async fn topup_bandwidth(&mut self, credential: CredentialSpendingData) -> Result<i64> {
        let client = self
            .lazy_client()
            .await
            .as_ref()
            .map_err(|err| MetadataClientError::Internal(err.to_string()))?;
        let request = match client.version {
            Version::V1 => v1::TopUpRequest { credential }.try_into()?,
        };
        let response = client.inner.topup_bandwidth(&request).await?;
        let available_bandwidth = match client.version {
            Version::V1 => v1::TopUpResponse::try_from(response)?.available_bandwidth,
        };
        Self::print_remaining_bandwidth(self.gateway_id, available_bandwidth);
        Ok(available_bandwidth)
    }
}
