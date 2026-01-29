// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_lib::service::ServiceConfigStorageType;
use nym_vpn_lib_types::TunnelEvent;
use nym_vpn_network_config::NetworkCache;
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::{
    NymEnvironment, VPNConfig, VpnError, vpn_service_command_sender::NymVpnServiceCommandSender,
};

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
    pub async fn new_service(
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
        let network_cache = NetworkCache::new(
            config.config_dir.clone(),
            &environment.current().nym_network.network_name,
            Some(config.user_agent.clone().into()),
            None,
        )
        .await
        .map_err(VpnError::internal)?;

        let vpn_service_params = nym_vpn_lib::service::NymVpnServiceParameters {
            // This is only needed for log removal helper
            log_path: None,
            config_dir: config.config_dir.clone(),
            data_dir: config.data_dir.clone(),
            network_cache,
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
