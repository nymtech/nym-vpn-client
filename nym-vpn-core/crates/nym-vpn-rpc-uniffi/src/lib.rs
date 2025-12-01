// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for RPC client from `nym-vpn-proto`.

uniffi::setup_scaffolding!();

use std::sync::Arc;

use futures::StreamExt;
use nym_vpn_proto::rpc_client::{Error as DaemonRpcError, RpcClient as DaemonRpcClient};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerState, EntryPoint, ExitPoint, FeatureFlags, Gateway,
    GatewayType, LogPath, NetworkCompatibility, NymVpnDevice, NymVpnUsage, ParsedAccountLinks,
    SystemMessage, TunnelEvent, TunnelState, VpnServiceConfig, VpnServiceInfo,
};

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

    pub async fn set_network(&self, network: String) -> Result<()> {
        self.inner.clone().set_network(network).await?;
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

    pub async fn get_default_dns(&self) -> Result<Vec<String>> {
        Ok(self.inner.clone().get_default_dns().await?)
    }

    pub async fn connect_tunnel(&self) -> Result<()> {
        self.inner.clone().connect_tunnel_v2().await?;
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

    pub async fn store_account(&self, mnemonic: String) -> Result<()> {
        let response = self
            .inner
            .clone()
            .store_account(nym_vpn_lib_types::StoreAccountRequest::Vpn { mnemonic })
            .await?;

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

    pub async fn get_account_links(&self, locale: String) -> Result<ParsedAccountLinks> {
        Ok(self.inner.clone().get_account_links(locale).await?)
    }

    pub async fn get_account_state(&self) -> Result<AccountControllerState> {
        Ok(self.inner.clone().get_account_state().await?)
    }

    pub async fn refresh_account_state(&self) -> Result<()> {
        self.inner.clone().refresh_account_state().await?;
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

    pub async fn get_log_path(&self) -> Result<LogPath> {
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

    pub async fn network_stats_enabled(&self, enabled: bool) -> Result<()> {
        self.inner.clone().network_stats_enabled(enabled).await?;
        Ok(())
    }

    pub async fn network_stats_allow_disconnected(&self, allow_disconnected: bool) -> Result<()> {
        self.inner
            .clone()
            .network_stats_allow_disconnected(allow_disconnected)
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum InnerRpcError {
    RpcError(DaemonRpcError),
    AccountCommand(Arc<AccountCommandError>),
}

impl std::fmt::Display for InnerRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerRpcError::RpcError(err) => write!(f, "{err}"),
            InnerRpcError::AccountCommand(err) => write!(f, "{err}"),
        }
    }
}

#[derive(Debug, uniffi::Object)]
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
    /// Returns the error message.
    pub fn message(&self) -> String {
        self.inner.to_string()
    }

    /// Returns the account error if the underlying error is an account error.
    pub fn account_error(&self) -> Option<AccountCommandError> {
        match &self.inner {
            InnerRpcError::AccountCommand(err) => Some(err.as_ref().clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
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
