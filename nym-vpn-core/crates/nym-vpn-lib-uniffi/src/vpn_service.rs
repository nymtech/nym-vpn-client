// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, sync::Arc};

use nym_common::ErrorExt;
use nym_vpn_lib::service::{
    AccountLinksError, ListGatewaysError, ServiceConfigStorageType, VpnServiceCommand,
};
use nym_vpn_lib_types::TunnelEvent;
use tokio::{
    sync::{Mutex, broadcast, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, DropGuard};

use nym_vpn_lib_types::{
    AccountCommandError, AccountControllerState, EntryPoint, ExitPoint, FeatureFlags, Gateway,
    ListGatewaysOptions, NetworkCompatibility, ParsedAccountLinks, StoreAccountRequest,
    SystemMessage, TargetState, TunnelState, VpnAccountSummary, VpnServiceConfig, VpnServiceInfo,
};

use crate::{NymEnvironment, VPNConfig, VpnError};

struct State {
    event_handler: JoinHandle<()>,
    vpn_service_handle: JoinHandle<()>,
    shutdown_drop_guard: DropGuard,
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_event(&self, event: TunnelEvent);
}

#[derive(uniffi::Object)]
pub struct NymVpnService {
    command_sender: Arc<NymVpnServiceCommandSender>,
    state: Arc<Mutex<Option<State>>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl NymVpnService {
    #[uniffi::constructor]
    pub async fn new(
        config: VPNConfig,
        environment: Arc<NymEnvironment>,
        event_listener: Arc<dyn TunnelStatusListener>,
    ) -> Result<Self, VpnError> {
        // Export environment first!
        environment.export_to_env();

        let service_storage_type =
            ServiceConfigStorageType::Ephemeral(config.as_vpn_service_config());

        #[cfg(target_os = "android")]
        let tun_provider = Arc::new(
            crate::tunnel_provider::android::AndroidTunProviderImpl::new(config.tun_provider),
        );
        #[cfg(target_os = "ios")]
        let tun_provider = Arc::new(crate::tunnel_provider::ios::OSTunProviderImpl::new(
            config.tun_provider,
        ));
        #[cfg(target_os = "android")]
        let connectivity_monitor =
            crate::offline_monitor::register_connectivity_observer(config.connectivity_monitor);

        let network_env = environment.inner().clone();
        let shutdown_token = CancellationToken::new();
        let vpn_service_params = nym_vpn_lib::service::NymVpnServiceParameters {
            // This is only needed for log removal helper
            log_path: None,
            config_dir: config.config_dir.clone(),
            data_dir: config.data_dir.clone(),
            network_env: Box::new(network_env),
            sentry_enabled: crate::logging::is_sentry_enabled().await,
            user_agent: config.user_agent.clone().into(),
            service_storage_type,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            tun_provider,
            #[cfg(target_os = "android")]
            connectivity_monitor: Box::new(connectivity_monitor),
        };

        let (vpn_command_tx, vpn_command_rx) = mpsc::unbounded_channel();
        let (tunnel_event_tx, mut tunnel_event_rx) = broadcast::channel(10);

        let vpn_service_handle = nym_vpn_lib::service::NymVpnService::spawn(
            vpn_command_rx,
            tunnel_event_tx,
            None,
            vpn_service_params,
            shutdown_token.child_token(),
        );

        let event_handler = tokio::spawn(async move {
            while let Ok(event) = tunnel_event_rx.recv().await {
                event_listener.on_event(event);
            }
        });

        Ok(NymVpnService {
            command_sender: Arc::new(NymVpnServiceCommandSender::new(vpn_command_tx)),
            state: Arc::new(Mutex::new(Some(State {
                event_handler,
                vpn_service_handle,
                shutdown_drop_guard: shutdown_token.drop_guard(),
            }))),
        })
    }

    pub fn get_command_sender(&self) -> Arc<NymVpnServiceCommandSender> {
        self.command_sender.clone()
    }

    pub async fn shutdown_and_wait(&self) {
        let Some(state) = self.state.lock().await.take() else {
            return;
        };

        drop(state.shutdown_drop_guard);

        state.event_handler.await.unwrap();
        state.vpn_service_handle.await.unwrap();
    }
}

#[derive(Debug, thiserror::Error)]
enum NymVpnServiceCommandInnerError {
    Internal(&'static str),
    ListGateway(#[source] ListGatewaysError),
    Account(#[source] AccountCommandError),
    AccountLinks(#[source] AccountLinksError),
}

impl std::fmt::Display for NymVpnServiceCommandInnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(msg) => f.write_str(msg),
            Self::ListGateway(err) => write!(f, "{}", err.display_chain()),
            Self::Account(err) => write!(f, "{}", err.display_chain()),
            Self::AccountLinks(err) => write!(f, "{}", err.display_chain()),
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

impl std::fmt::Display for NymVpnServiceCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl From<NymVpnServiceCommandInnerError> for NymVpnServiceCommandError {
    fn from(value: NymVpnServiceCommandInnerError) -> Self {
        Self::new(value)
    }
}

#[derive(uniffi::Object)]
pub struct NymVpnServiceCommandSender {
    vpn_command_tx: mpsc::UnboundedSender<VpnServiceCommand>,
}

impl NymVpnServiceCommandSender {
    fn new(vpn_command_tx: mpsc::UnboundedSender<VpnServiceCommand>) -> Self {
        Self { vpn_command_tx }
    }

    async fn send_and_wait<R, F, O>(
        &self,
        command: F,
        opts: O,
    ) -> Result<R, NymVpnServiceCommandError>
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
    pub async fn get_info(&self) -> Result<VpnServiceInfo, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::Info, ()).await
    }

    pub async fn get_config(&self) -> Result<VpnServiceConfig, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetConfig, ()).await
    }

    pub async fn set_enable_two_hop(
        &self,
        enable_two_hop: bool,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetEnableTwoHop, enable_two_hop)
            .await
    }

    pub async fn set_entry_point(
        &self,
        entry_point: EntryPoint,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetEntryPoint, entry_point)
            .await
    }

    pub async fn set_exit_point(
        &self,
        exit_point: ExitPoint,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetExitPoint, exit_point)
            .await
    }

    pub async fn set_enable_bridges(
        &self,
        enable_bridges: bool,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetEnableBridges, enable_bridges)
            .await
    }

    pub async fn set_residential_exit(
        &self,
        residential_exit: bool,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetResidentialExit, residential_exit)
            .await
    }

    pub async fn set_enable_custom_dns(
        &self,
        enable_custom_dns: bool,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetEnableCustomDns, enable_custom_dns)
            .await
    }

    pub async fn set_custom_dns(
        &self,
        addrs: Vec<IpAddr>,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetCustomDns, addrs)
            .await
    }

    pub async fn get_system_messages(
        &self,
    ) -> Result<Vec<SystemMessage>, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetSystemMessages, ())
            .await
    }

    pub async fn get_network_compatibility(
        &self,
    ) -> Result<Option<NetworkCompatibility>, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetNetworkCompatibility, ())
            .await
    }

    pub async fn get_feature_flags(
        &self,
    ) -> Result<Option<Arc<FeatureFlags>>, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetFeatureFlags, ())
            .await
            .map(|v| v.map(|v| Arc::new(v)))
    }

    pub async fn get_default_dns(&self) -> Result<Vec<IpAddr>, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetDefaultDns, ())
            .await
    }

    pub async fn list_gateways(
        &self,
        options: ListGatewaysOptions,
    ) -> Result<Vec<Gateway>, NymVpnServiceCommandError> {
        Ok(self
            .send_and_wait(VpnServiceCommand::ListGateways, options)
            .await?
            .map_err(NymVpnServiceCommandInnerError::ListGateway)?)
    }

    pub async fn connect_tunnel(&self) -> Result<bool, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Secured)
            .await
    }

    pub async fn disconnect_tunnel(&self) -> Result<bool, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::SetTargetState, TargetState::Unsecured)
            .await
    }

    pub async fn reconnect_tunnel(&self) -> Result<bool, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::Reconnect, ()).await
    }

    pub async fn get_tunnel_state(&self) -> Result<TunnelState, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetTunnelState, ())
            .await
    }

    pub async fn store_account(
        &self,
        request: StoreAccountRequest,
    ) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::StoreAccount, request)
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn is_account_stored(&self) -> Result<bool, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::IsAccountStored, ())
            .await
    }

    pub async fn forget_account(&self) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::ForgetAccount, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn rotate_keys(&self) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::RotateKeys, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }

    pub async fn get_account_identity(&self) -> Result<Option<String>, NymVpnServiceCommandError> {
        let value = self
            .send_and_wait(VpnServiceCommand::GetAccountIdentity, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(value)
    }

    pub async fn get_account_links(
        &self,
        locale: String,
    ) -> Result<ParsedAccountLinks, NymVpnServiceCommandError> {
        let value = self
            .send_and_wait(VpnServiceCommand::GetAccountLinks, locale)
            .await?
            .map_err(NymVpnServiceCommandInnerError::AccountLinks)?;
        Ok(value)
    }

    pub async fn get_account_state(
        &self,
    ) -> Result<AccountControllerState, NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::GetAccountState, ())
            .await
    }

    pub async fn refresh_account(&self) -> Result<(), NymVpnServiceCommandError> {
        self.send_and_wait(VpnServiceCommand::RefreshAccountState, ())
            .await
    }

    pub async fn get_account_summary(
        &self,
    ) -> Result<Option<VpnAccountSummary>, NymVpnServiceCommandError> {
        Ok(self
            .send_and_wait(VpnServiceCommand::GetAccountSummary, ())
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?)
    }
}
