// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, sync::Arc};

use nym_common::ErrorExt;
use nym_vpn_lib::service::{
    AccountLinksError, GeoExclusionConfigError, ListGatewaysError, VpnServiceCommand,
};
use tokio::sync::{mpsc, oneshot};

#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AppBypassConfig;

use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerState, AutologinResponse, DiagnosticRunParams,
    EntryPoint, ExitPoint, FeatureFlags, FrontingMode, Gateway, GetDeeplinkParams,
    ListGatewaysOptions, MixnetTrafficConfig, NetworkCompatibility, ParsedAccountLinks,
    RecentGateways, RegisterAccountRequest, RegisterAccountResponse, StoreAccountRequest,
    StoredAccountMode, SystemMessage, TargetState, TentativeGateways, TunnelState, TunnelType,
    VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};

#[derive(Debug, thiserror::Error)]
enum NymVpnServiceCommandInnerError {
    Internal(&'static str),
    ListGateway(#[source] ListGatewaysError),
    Account(#[source] AccountCommandError),
    AccountLinks(#[source] AccountLinksError),
    GeoExclusionConfig(#[source] GeoExclusionConfigError),
}

impl NymVpnServiceCommandInnerError {
    pub fn error_chain(&self) -> String {
        match self {
            Self::Internal(msg) => msg.to_string(),
            Self::ListGateway(err) => err.display_chain(),
            Self::Account(err) => err.display_chain(),
            Self::AccountLinks(err) => err.display_chain(),
            Self::GeoExclusionConfig(err) => err.display_chain(),
        }
    }
}

impl std::fmt::Display for NymVpnServiceCommandInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(msg) => f.write_str(msg),
            Self::ListGateway(err) => write!(f, "{}", err),
            Self::Account(err) => write!(f, "{}", err),
            Self::AccountLinks(err) => write!(f, "{}", err),
            Self::GeoExclusionConfig(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Debug, uniffi::Object)]
#[uniffi::export(Display)]
pub struct NymVpnServiceCommandError {
    inner: NymVpnServiceCommandInnerError,
}

impl NymVpnServiceCommandError {
    fn new(inner: NymVpnServiceCommandInnerError) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl NymVpnServiceCommandError {
    /// Returns formatted error chain
    pub fn error_chain(&self) -> String {
        self.inner.error_chain()
    }
}

impl std::fmt::Display for NymVpnServiceCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner.to_string())
    }
}

impl From<NymVpnServiceCommandInnerError> for NymVpnServiceCommandError {
    fn from(value: NymVpnServiceCommandInnerError) -> Self {
        Self::new(value)
    }
}

pub type Result<T, E = NymVpnServiceCommandError> = std::result::Result<T, E>;

#[derive(uniffi::Object)]
pub struct NymVpnServiceCommandSender {
    vpn_command_tx: mpsc::UnboundedSender<VpnServiceCommand>,
}

impl NymVpnServiceCommandSender {
    pub fn new(vpn_command_tx: mpsc::UnboundedSender<VpnServiceCommand>) -> Self {
        Self { vpn_command_tx }
    }

    async fn send_and_wait<R, F, O>(&self, command: F, opts: O) -> Result<R>
    where
        F: FnOnce(oneshot::Sender<R>, O) -> VpnServiceCommand,
    {
        let (tx, rx) = oneshot::channel();

        self.vpn_command_tx.send(command(tx, opts)).map_err(|_| {
            NymVpnServiceCommandError::new(NymVpnServiceCommandInnerError::Internal(
                "Command channel is closed",
            ))
        })?;

        rx.await.map_err(|_| {
            NymVpnServiceCommandError::new(NymVpnServiceCommandInnerError::Internal(
                "Response channel is closed",
            ))
        })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NymVpnServiceCommandSender {
    pub async fn get_info(&self) -> Result<VpnServiceInfo> {
        self.send_and_wait(VpnServiceCommand::Info, ()).await
    }

    pub async fn get_config(&self) -> Result<VpnServiceConfig> {
        self.send_and_wait(VpnServiceCommand::GetConfig, ()).await
    }

    pub async fn set_enable_two_hop(&self, enable_two_hop: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetEnableTwoHop, enable_two_hop)
            .await
    }

    pub async fn set_entry_point(&self, entry_point: EntryPoint) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetEntryPoint, entry_point)
            .await
    }

    pub async fn set_exit_point(&self, exit_point: ExitPoint) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetExitPoint, exit_point)
            .await
    }

    pub async fn set_enable_bridges(&self, enable_bridges: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetEnableBridges, enable_bridges)
            .await
    }

    pub async fn set_enable_ad_blocking(&self, enable_ad_blocking: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetEnableAdBlocking, enable_ad_blocking)
            .await
    }

    pub async fn set_residential_exit(&self, residential_exit: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetResidentialExit, residential_exit)
            .await
    }

    pub async fn set_enable_custom_dns(&self, enable_custom_dns: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetEnableCustomDns, enable_custom_dns)
            .await
    }

    pub async fn set_custom_dns(&self, addrs: Vec<IpAddr>) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetCustomDns, addrs)
            .await
    }

    pub async fn set_enable_gateway_independence(
        &self,
        enable_gateway_independence: bool,
    ) -> Result<()> {
        self.send_and_wait(
            VpnServiceCommand::SetEnableGatewayIndependence,
            enable_gateway_independence,
        )
        .await
    }

    pub async fn set_gateway_independence_notifications(
        &self,
        enable_notifications: bool,
    ) -> Result<()> {
        self.send_and_wait(
            VpnServiceCommand::SetGatewayIndependenceNotifications,
            enable_notifications,
        )
        .await
    }

    pub async fn set_fronting_mode(&self, fronting_mode: FrontingMode) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetFrontingMode, fronting_mode)
            .await
    }

    pub async fn set_disable_ipv6(&self, disable_ipv6: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetDisableIPv6, disable_ipv6)
            .await
    }

    pub async fn set_mixnet_traffic_config(
        &self,
        mixnet_traffic_config: MixnetTrafficConfig,
    ) -> Result<()> {
        self.send_and_wait(
            VpnServiceCommand::SetMixnetTrafficConfig,
            mixnet_traffic_config,
        )
        .await?
        .map_err(|_| {
            NymVpnServiceCommandInnerError::Internal("Failed to set mixnet traffic config")
        })?;
        Ok(())
    }

    pub async fn get_system_messages(&self) -> Result<Vec<SystemMessage>> {
        self.send_and_wait(VpnServiceCommand::GetSystemMessages, ())
            .await
    }

    pub async fn get_network_compatibility(&self) -> Result<Option<NetworkCompatibility>> {
        self.send_and_wait(VpnServiceCommand::GetNetworkCompatibility, ())
            .await
    }

    pub async fn get_feature_flags(&self) -> Result<Option<Arc<FeatureFlags>>> {
        self.send_and_wait(VpnServiceCommand::GetFeatureFlags, ())
            .await
            .map(|v| v.map(Arc::new))
    }

    pub async fn get_default_dns(&self) -> Result<Vec<IpAddr>> {
        self.send_and_wait(VpnServiceCommand::GetDefaultDns, ())
            .await
    }

    pub async fn list_gateways(&self, options: ListGatewaysOptions) -> Result<Vec<Gateway>> {
        Ok(self
            .send_and_wait(VpnServiceCommand::ListGateways, options)
            .await?
            .map_err(NymVpnServiceCommandInnerError::ListGateway)?)
    }

    pub async fn set_geo_exclusion_enabled(&self, enabled: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetGeoExclusionEnabled, enabled)
            .await
    }

    pub async fn set_geo_exclusion_listen_port(&self, listen_port: u16) -> Result<()> {
        Ok(self
            .send_and_wait(VpnServiceCommand::SetGeoExclusionListenPort, listen_port)
            .await?
            .map_err(NymVpnServiceCommandInnerError::GeoExclusionConfig)?)
    }

    pub async fn set_geo_exclusion_excluded_countries(
        &self,
        excluded_countries: Vec<String>,
    ) -> Result<()> {
        Ok(self
            .send_and_wait(
                VpnServiceCommand::SetGeoExclusionExcludedCountries,
                excluded_countries,
            )
            .await?
            .map_err(NymVpnServiceCommandInnerError::GeoExclusionConfig)?)
    }

    pub async fn connect_tunnel(&self) -> Result<bool> {
        self.send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Secured)
            .await
    }

    pub async fn disconnect_tunnel(&self) -> Result<bool> {
        self.send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Unsecured)
            .await
    }

    pub async fn reconnect_tunnel(&self) -> Result<bool> {
        self.send_and_wait(VpnServiceCommand::Reconnect, ()).await
    }

    pub async fn get_tunnel_state(&self) -> Result<TunnelState> {
        self.send_and_wait(VpnServiceCommand::GetTunnelState, ())
            .await
    }

    pub async fn register_account(
        &self,
        request: RegisterAccountRequest,
    ) -> Result<RegisterAccountResponse> {
        Ok(self
            .send_and_wait(VpnServiceCommand::RegisterAccount, request)
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn create_account(&self) -> Result<()> {
        Ok(self
            .send_and_wait(VpnServiceCommand::CreateAccount, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn store_account(&self, request: StoreAccountRequest) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::StoreAccount, request)
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn is_account_stored(&self) -> Result<bool> {
        self.send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await
    }

    pub async fn get_stored_mnemonic(&self) -> Result<String> {
        Ok(self
            .send_and_wait(VpnServiceCommand::GetStoredMnemonic, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn forget_account(&self) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::ForgetAccount, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn rotate_keys(&self) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::RotateKeys, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn get_device_identity(&self) -> Result<Option<String>> {
        let value = self
            .send_and_wait(VpnServiceCommand::GetDeviceIdentity, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(value)
    }

    pub async fn get_account_identity(&self) -> Result<Option<String>> {
        let value = self
            .send_and_wait(VpnServiceCommand::GetAccountIdentity, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(value)
    }

    pub async fn get_canonical_account_identity(&self) -> Result<Option<String>> {
        let value = self
            .send_and_wait(VpnServiceCommand::GetCanonicalAccountIdentity, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(value)
    }

    pub async fn get_account_mode(&self) -> Option<StoredAccountMode> {
        self.send_and_wait(VpnServiceCommand::GetAccountMode, ())
            .await
            .ok()
            .flatten()
    }

    pub async fn get_account_links(&self, locale: String) -> Result<ParsedAccountLinks> {
        let value = self
            .send_and_wait(VpnServiceCommand::GetAccountLinks, locale)
            .await?
            .map_err(NymVpnServiceCommandInnerError::AccountLinks)?;
        Ok(value)
    }

    pub async fn get_account_state(&self) -> Result<AccountControllerState> {
        self.send_and_wait(VpnServiceCommand::GetAccountState, ())
            .await
    }

    pub async fn refresh_account(&self, force: bool) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::RefreshAccountState, force)
            .await
    }

    pub async fn get_account_summary(&self) -> Result<Option<VpnAccountSummary>> {
        Ok(self
            .send_and_wait(VpnServiceCommand::GetAccountSummary, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn handle_subscription_payment(&self) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::HandleSubscriptionPayment, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn get_deeplink(&self, params: GetDeeplinkParams) -> Result<String> {
        Ok(self
            .send_and_wait(VpnServiceCommand::GetDeeplink, params)
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn deeplink_store_account(&self, deeplink_callback_url: String) -> Result<()> {
        Ok(self
            .send_and_wait(
                VpnServiceCommand::DeeplinkStoreAccount,
                deeplink_callback_url,
            )
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn get_autologin_deeplink(
        &self,
        params: GetDeeplinkParams,
    ) -> Result<AutologinResponse> {
        Ok(self
            .send_and_wait(VpnServiceCommand::GetAutologinDeeplink, params)
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }

    pub async fn run_diagnostic(&self, params: DiagnosticRunParams) -> Result<String> {
        Ok(serde_json::to_string_pretty(
            &self
                .send_and_wait(VpnServiceCommand::RunDiagnostic, params)
                .await?,
        )
        .map_err(|_| {
            NymVpnServiceCommandInnerError::Internal("Failed to serialize DiagnosticReport")
        })?)
    }

    pub async fn get_tentative_gateways(&self) -> Result<TentativeGateways> {
        self.send_and_wait(VpnServiceCommand::GetTentativeGateways, ())
            .await
    }

    pub async fn get_recent_gateways(&self, tunnel_type: TunnelType) -> Result<RecentGateways> {
        Ok(self
            .send_and_wait(VpnServiceCommand::GetRecentGateways, tunnel_type)
            .await?
            .map_err(NymVpnServiceCommandInnerError::ListGateway)?)
    }
}

#[cfg(target_os = "android")]
#[uniffi::export(async_runtime = "tokio")]
impl NymVpnServiceCommandSender {
    /// Set the app bypass ("steering") configuration used on the next connect.
    ///
    /// Pass `None` to turn app bypass off. The value is not persisted, so it must be sent on
    /// every connect.
    pub async fn set_app_bypass(&self, config: Option<AppBypassConfig>) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetAppBypass, config.map(Into::into))
            .await
    }
}
