// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for RPC client from `nym-vpn-proto`.

uniffi::setup_scaffolding!();

use std::sync::Arc;

use futures::StreamExt;
use nym_vpn_proto::rpc_client::{Error as DaemonRpcError, RpcClient as DaemonRpcClient};
use tokio_util::sync::CancellationToken;

use nym_vpn_lib_types_uniffi::{
    AccountCommandError, AccountControllerState, AccountLinks, EntryPoint, ExitPoint, FeatureFlags,
    GatewayType, NetworkCompatibility, SystemMessage, TunnelEvent, TunnelState,
};
use nym_vpnd_types_uniffi::{
    gateway::{Gateway, GatewayCountry},
    log_path::LogPath,
    nym_vpn_api::{NymVpnDevice, NymVpnUsage},
    service::{VpnServiceConfig, VpnServiceInfo},
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
        Ok(self
            .inner
            .clone()
            .get_info()
            .await
            .map(VpnServiceInfo::from)?)
    }

    pub async fn get_config(&self) -> Result<VpnServiceConfig> {
        Ok(self
            .inner
            .clone()
            .get_config()
            .await
            .map(VpnServiceConfig::from)?)
    }

    pub async fn set_entry_point(&self, entry_point: EntryPoint) -> Result<()> {
        let entry_point = nym_gateway_directory::EntryPoint::from(entry_point);

        self.inner.clone().set_entry_point(entry_point).await?;
        Ok(())
    }

    pub async fn set_exit_point(&self, exit_point: ExitPoint) -> Result<()> {
        let exit_point = nym_gateway_directory::ExitPoint::from(exit_point);

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

    pub async fn set_network(&self, network: String) -> Result<()> {
        self.inner.clone().set_network(network).await?;
        Ok(())
    }

    pub async fn get_system_messages(&self) -> Result<Vec<SystemMessage>> {
        let system_messages = self.inner.clone().get_system_messages().await?;
        Ok(system_messages
            .into_iter()
            .map(SystemMessage::from)
            .collect())
    }

    pub async fn get_network_compatibility(&self) -> Result<Option<NetworkCompatibility>> {
        let network_compatibility = self
            .inner
            .clone()
            .get_network_compatibility()
            .await?
            .map(NetworkCompatibility::from);
        Ok(network_compatibility)
    }

    pub async fn get_feature_flags(&self) -> Result<FeatureFlags> {
        Ok(self
            .inner
            .clone()
            .get_feature_flags()
            .await
            .map(FeatureFlags::from)?)
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
        Ok(self
            .inner
            .clone()
            .get_tunnel_state()
            .await
            .map(TunnelState::from)?)
    }

    pub async fn listen_to_tunnel_state(
        &self,
        observer: Arc<dyn TunnelStateObserver>,
    ) -> Result<StreamObserver> {
        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();
        let mut event_stream = self.inner.clone().listen_to_tunnel_state().await?;

        tokio::spawn(async move {
            loop {
                match child_token
                    .run_until_cancelled(event_stream.next())
                    .await
                    .flatten()
                {
                    Some(Ok(evt)) => {
                        observer.on_tunnel_state_change(TunnelState::from(evt));
                    }
                    Some(Err(err)) => {
                        tracing::error!("Error receiving next tunnel state: {err}");
                        break;
                    }
                    None => break,
                }
            }
            observer.on_close();
        });

        Ok(StreamObserver::new(cancel_token))
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
                        observer.on_tunnel_event(TunnelEvent::from(evt));
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
        let options = nym_vpnd_types::ListGatewaysOptions {
            gw_type: nym_gateway_directory::GatewayType::from(gw_type),
            user_agent: None,
        };
        let gateways = self
            .inner
            .clone()
            .list_gateways(options)
            .await?
            .into_iter()
            .map(Gateway::from)
            .collect();
        Ok(gateways)
    }

    pub async fn list_countries(&self, gw_type: GatewayType) -> Result<Vec<GatewayCountry>> {
        let options = nym_vpnd_types::ListCountriesOptions {
            gw_type: nym_gateway_directory::GatewayType::from(gw_type),
            user_agent: None,
        };
        let countries = self
            .inner
            .clone()
            .list_countries(options)
            .await?
            .into_iter()
            .map(GatewayCountry::from)
            .collect();
        Ok(countries)
    }

    pub async fn store_account(&self, mnemonic: String) -> Result<()> {
        let response = self
            .inner
            .clone()
            .store_account(nym_vpnd_types::StoreAccountRequest { mnemonic })
            .await?;

        if let Some(err) = response.error {
            Err(RpcError::new(InnerRpcError::AccountCommand(Arc::new(
                AccountCommandError::from(err),
            ))))
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
            Err(RpcError::from(Arc::new(AccountCommandError::from(err))))
        } else {
            Ok(())
        }
    }

    pub async fn get_account_identity(&self) -> Result<Option<String>> {
        Ok(self.inner.clone().get_account_identity().await?)
    }

    pub async fn get_account_links(&self, locale: String) -> Result<AccountLinks> {
        Ok(self
            .inner
            .clone()
            .get_account_links(locale)
            .await
            .map(AccountLinks::from)?)
    }

    pub async fn get_account_state(&self) -> Result<AccountControllerState> {
        Ok(self
            .inner
            .clone()
            .get_account_state()
            .await
            .map(AccountControllerState::from)?)
    }

    pub async fn listen_to_account_state(
        &self,
        observer: Arc<dyn AccountEventObserver>,
    ) -> Result<StreamObserver> {
        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();
        let mut event_stream = self
            .inner
            .clone()
            .listen_to_account_controller_state()
            .await?;

        tokio::spawn(async move {
            loop {
                match child_token
                    .run_until_cancelled(event_stream.next())
                    .await
                    .flatten()
                {
                    Some(Ok(evt)) => {
                        observer.on_account_state_change(AccountControllerState::from(evt));
                    }
                    Some(Err(err)) => {
                        tracing::error!("Error receiving next account state: {err}");
                        break;
                    }
                    None => break,
                }
            }
            observer.on_close();
        });

        Ok(StreamObserver::new(cancel_token))
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
            .map(NymVpnUsage::from)
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
            .map(NymVpnDevice::from)
            .collect())
    }

    pub async fn get_active_devices(&self) -> Result<Vec<NymVpnDevice>> {
        Ok(self
            .inner
            .clone()
            .get_active_devices()
            .await?
            .into_iter()
            .map(NymVpnDevice::from)
            .collect())
    }

    pub async fn get_log_path(&self) -> Result<LogPath> {
        let log_path = self.inner.clone().get_log_path().await.map(LogPath::from)?;
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

    pub async fn is_collect_network_stats_enabled(&self) -> Result<bool> {
        Ok(self
            .inner
            .clone()
            .is_collect_network_stats_enabled()
            .await?)
    }

    pub async fn enable_collect_network_stats(&self) -> Result<()> {
        self.inner.clone().enable_collect_network_stats().await?;
        Ok(())
    }

    pub async fn disable_collect_network_stats(&self) -> Result<()> {
        self.inner.clone().disable_collect_network_stats().await?;
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
            InnerRpcError::AccountCommand(err) => write!(f, "{}", err.message()),
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
    pub fn account_error(&self) -> Option<Arc<AccountCommandError>> {
        match &self.inner {
            InnerRpcError::AccountCommand(err) => Some(err.clone()),
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

#[uniffi::export(with_foreign)]
pub trait TunnelStateObserver: Send + Sync {
    fn on_tunnel_state_change(&self, new_state: TunnelState);
    fn on_close(&self);
}

#[uniffi::export(with_foreign)]
pub trait AccountEventObserver: Send + Sync {
    fn on_account_state_change(&self, new_state: AccountControllerState);
    fn on_close(&self);
}
