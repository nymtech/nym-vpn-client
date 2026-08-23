// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use nym_credentials_interface::CredentialSpendingData;
use nym_gateway_directory::NodeIdentity;
use nym_wireguard_private_metadata_client::WireguardMetadataApiClient;
use nym_wireguard_private_metadata_shared::{AvailableBandwidth, Version, v1, v2};
use tokio::sync::{OnceCell, oneshot};
use url::Url;

use error::Result;

use crate::error::MetadataClientError;

pub mod error;

pub struct TunUpSendData {
    pub metadata_endpoint_reachable_tx: oneshot::Sender<bool>,
    pub data_type: TunUpSendDataType,
}

#[derive(Clone)]
pub enum TunUpSendDataType {
    InterfaceName(String),
    TcpProxy(SocketAddr),
}

pub type TunUpSender = tokio::sync::oneshot::Sender<TunUpSendData>;
pub type TunUpReceiver = tokio::sync::oneshot::Receiver<TunUpSendData>;

#[derive(Debug, Clone)]
struct LazyMetadataClient {
    inner: nym_http_api_client::Client,
    interface_name: Option<String>,
    version: Version,
}

#[cfg(target_os = "android")]
// Kernel after version 5.7 supports binding without root or `CAP_NET_RAW` capability
// Linux is already ran as root so the version only needs to be > 2.0.30 (released in 1997)
// so we don't check for that
fn kernel_supports_interface_binding() -> bool {
    let Ok(uts_name) = nix::sys::utsname::uname() else {
        return false;
    };
    let Some(release_str) = uts_name.release().to_str() else {
        return false;
    };
    let Ok(version) = semver::Version::parse(release_str) else {
        return false;
    };
    version >= semver::Version::new(5, 7, 0)
}

impl LazyMetadataClient {
    async fn new(
        mut base_url: Url,
        bind_ip: IpAddr,
        retries: usize,
        timeout: Duration,
        sent_data: TunUpSendData,
    ) -> Result<Self> {
        let mut interface_name = None;
        // Seed from the registry-configured builder (not `ReqwestClientBuilder::new()`)
        // so platform-specific TLS overrides (e.g. Android's webpki-roots backend, needed
        // because rustls-platform-verifier isn't initialized in this process) still apply
        // even though `with_reqwest_builder` below bypasses `nym_http_api_client`'s own
        // client construction.
        let reqwest_builder = nym_http_api_client::registry::default_builder();
        let reqwest_builder = match sent_data.data_type {
            TunUpSendDataType::InterfaceName(interface) => {
                #[cfg(any(target_os = "linux", target_os = "ios"))]
                let reqwest_builder = reqwest_builder.interface(&interface);
                #[cfg(target_os = "android")]
                let reqwest_builder = if kernel_supports_interface_binding() {
                    reqwest_builder.interface(&interface)
                } else {
                    reqwest_builder
                };

                interface_name = Some(interface.clone());
                reqwest_builder.local_address(bind_ip)
            }
            TunUpSendDataType::TcpProxy(tcp_proxy) => {
                base_url.set_ip_host(tcp_proxy.ip()).map_err(|_| {
                    MetadataClientError::Internal("failed to set tcp proxy ip".to_owned())
                })?;

                base_url.set_port(Some(tcp_proxy.port())).map_err(|_| {
                    MetadataClientError::Internal("failed to set tcp proxy port".to_owned())
                })?;

                reqwest_builder
            }
        };

        let inner = nym_http_api_client::Client::builder(base_url)
            .and_then(|builder| {
                builder
                    .with_reqwest_builder(reqwest_builder)
                    .with_retries(retries)
                    .with_timeout(timeout)
                    .build()
            })
            .map_err(Box::new)?;
        let response = inner.version().await.map_err(Box::new);

        let endpoint_reachable = response.is_ok();
        let _ = sent_data
            .metadata_endpoint_reachable_tx
            .send(endpoint_reachable);

        Ok(Self {
            inner,
            interface_name,
            version: response?,
        })
    }
}

pub struct MetadataClient {
    lazy_client: OnceCell<Result<LazyMetadataClient>>,
    lazy_client_retries: usize,
    lazy_client_timeout: Duration,
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
                LazyMetadataClient::new(
                    self.base_url.clone(),
                    self.bind_ip,
                    self.lazy_client_retries,
                    self.lazy_client_timeout,
                    data,
                )
                .await
            })
            .await
    }

    pub fn new(
        base_url: Url,
        gateway_id: NodeIdentity,
        bind_ip: IpAddr,
        signal_channel: TunUpReceiver,
        lazy_client_retries: usize,
        lazy_client_timeout: Duration,
    ) -> Self {
        Self {
            lazy_client: OnceCell::new(),
            lazy_client_retries,
            lazy_client_timeout,
            gateway_id,
            bind_ip,
            base_url,
            signal_channel: Some(signal_channel),
        }
    }

    pub fn gateway_id(&self) -> NodeIdentity {
        self.gateway_id
    }

    // Make sure the initialization is done, so that we can get the interface name without having to wait for the first query to complete.
    pub async fn lazy_init(&mut self) {
        self.lazy_client().await;
    }

    pub async fn interface_name(&mut self) -> Option<String> {
        self.lazy_client()
            .await
            .as_ref()
            .ok()
            .and_then(|client| client.interface_name.clone())
    }

    fn print_remaining_bandwidth(
        gateway_id: NodeIdentity,
        available_bandwidth: AvailableBandwidth,
    ) {
        let bytes = available_bandwidth.bandwidth_bytes;
        let upgrade_mode = available_bandwidth.upgrade_mode == Some(true);

        let remaining_pretty = if bytes > 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
        } else {
            format!("{} KB", bytes / 1024)
        };
        tracing::debug!(
            "Remaining wireguard bandwidth with gateway {} for today: {}",
            gateway_id,
            remaining_pretty
        );
        if upgrade_mode {
            tracing::debug!("Bandwidth is not metered as the system is undergoing an upgrade")
        }
    }

    pub async fn query_bandwidth(&mut self) -> Result<AvailableBandwidth> {
        let client = self
            .lazy_client()
            .await
            .as_ref()
            .map_err(|err| MetadataClientError::Internal(err.to_string()))?;
        let request = match client.version {
            Version::V1 => v1::AvailableBandwidthRequest {}.try_into()?,
            Version::V2 => v2::AvailableBandwidthRequest {}.try_into()?,
        };
        let response = client
            .inner
            .available_bandwidth(&request)
            .await
            .map_err(Box::new)?;
        let available_bandwidth = match client.version {
            Version::V1 => v1::AvailableBandwidthResponse::try_from(response)?.into(),
            Version::V2 => v2::AvailableBandwidthResponse::try_from(response)?.into(),
        };
        Self::print_remaining_bandwidth(self.gateway_id, available_bandwidth);
        Ok(available_bandwidth)
    }

    pub async fn topup_bandwidth(
        &mut self,
        credential: CredentialSpendingData,
    ) -> Result<AvailableBandwidth> {
        let client = self
            .lazy_client()
            .await
            .as_ref()
            .map_err(|err| MetadataClientError::Internal(err.to_string()))?;
        let request = match client.version {
            Version::V1 => v1::TopUpRequest { credential }.try_into()?,
            Version::V2 => v2::TopUpRequest {
                credential: credential.into(),
            }
            .try_into()?,
        };
        let response = client
            .inner
            .topup_bandwidth(&request)
            .await
            .map_err(Box::new)?;
        let available_bandwidth = match client.version {
            Version::V1 => v1::TopUpResponse::try_from(response)?.into(),
            Version::V2 => v2::TopUpResponse::try_from(response)?.into(),
        };
        Self::print_remaining_bandwidth(self.gateway_id, available_bandwidth);
        Ok(available_bandwidth)
    }

    pub async fn check_upgrade_mode(&mut self, upgrade_mode_jwt: String) -> Result<bool> {
        let client = self
            .lazy_client()
            .await
            .as_ref()
            .map_err(|err| MetadataClientError::Internal(err.to_string()))?;

        let request = match client.version {
            Version::V1 => return Err(MetadataClientError::UnsupportedMetadataEndpointVersion),
            Version::V2 => v2::UpgradeModeCheckRequest {
                request_type: v2::UpgradeModeCheckRequestType::UpgradeModeJwt {
                    token: upgrade_mode_jwt,
                },
            }
            .try_into()?,
        };
        let response = client
            .inner
            .request_upgrade_mode_check(&request)
            .await
            .map_err(Box::new)?;
        let upgrade_mode_enabled = match client.version {
            Version::V1 => return Err(MetadataClientError::UnsupportedMetadataEndpointVersion),
            Version::V2 => v2::UpgradeModeCheckResponse::try_from(response)?.upgrade_mode,
        };

        Ok(upgrade_mode_enabled)
    }
}
