// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_lib::{
    paths::Paths,
    service::{ServiceConfigStorageType, VPN_DISCONNECT_TIMEOUT},
};
use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use nym_vpn_network_config::NetworkCache;
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::{
    NymEnvironment, TOKIO_RUNTIME, VPNConfig, VpnError,
    vpn_service_command_sender::NymVpnServiceCommandSender,
};

struct State {
    event_handler: JoinHandle<()>,
    vpn_service_handle: JoinHandle<()>,
    shutdown_drop_guard: DropGuard,
    tunnel_state: tokio::sync::watch::Receiver<Option<TunnelState>>,
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
        // Uniffi async adapter runs on single-threaded runtime
        // Create vpn service on multi-threaded runtime to ensure that it has enough resources
        TOKIO_RUNTIME
            .spawn(async move {
                // Export environment first!
                environment.export_to_env();

                let service_storage_type =
                    ServiceConfigStorageType::Ephemeral(config.as_vpn_service_config());

                #[cfg(target_os = "android")]
                let tun_provider = Arc::new(
                    crate::tunnel_provider::android::AndroidTunProviderImpl::new(
                        config.tun_provider,
                    ),
                );
                #[cfg(target_os = "ios")]
                let tun_provider = Arc::new(crate::tunnel_provider::ios::OSTunProviderImpl::new(
                    config.tun_provider,
                ));
                #[cfg(target_os = "android")]
                let connectivity_monitor = crate::offline_monitor::register_connectivity_observer(
                    config.connectivity_monitor,
                );

                let shutdown_token = CancellationToken::new();

                let paths = Paths {
                    data_dir: config.data_dir.clone(),
                    config_dir: config.config_dir.clone(),
                    log_dir: config.log_dir.clone(),
                    log_path: None,
                };

                paths
                    .create_directories()
                    .await
                    .map_err(|e| VpnError::InternalError {
                        details: e.to_string(),
                    })?;

                let network_cache = NetworkCache::new(
                    config.config_dir.to_path_buf(),
                    &environment.current().nym_network.network_name,
                    Some(config.user_agent.clone().into()),
                )
                .await
                .map_err(VpnError::internal)?;

                let vpn_service_params = nym_vpn_lib::service::NymVpnServiceParameters {
                    paths,
                    network_cache,
                    sentry_enabled: crate::logging::is_sentry_enabled(),
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

                let (tunnel_state_tx, tunnel_state_rx) = tokio::sync::watch::channel(None);
                let event_handler = tokio::spawn(async move {
                    while let Ok(event) = tunnel_event_rx.recv().await {
                        if let TunnelEvent::NewState(tunnel_state) = &event {
                            let _ = tunnel_state_tx.send_replace(Some(tunnel_state.clone()));
                        }
                        event_listener.on_event(event);
                    }
                });

                Ok(NymVpnService {
                    command_sender: Arc::new(NymVpnServiceCommandSender::new(vpn_command_tx)),
                    state: Arc::new(Mutex::new(Some(State {
                        event_handler,
                        vpn_service_handle,
                        shutdown_drop_guard: shutdown_token.drop_guard(),
                        tunnel_state: tunnel_state_rx,
                    }))),
                })
            })
            .await
            .map_err(|err| VpnError::InternalError {
                details: format!("failed to join on multi-threaded tokio runtime: {}", err),
            })?
    }

    pub fn get_command_sender(&self) -> Arc<NymVpnServiceCommandSender> {
        self.command_sender.clone()
    }

    pub async fn shutdown_and_wait(&self) {
        let Some(state) = self.state.lock().await.take() else {
            return;
        };

        let _ = self.command_sender.disconnect_tunnel().await;
        self.wait_for_disconnect(state.tunnel_state).await;

        drop(state.shutdown_drop_guard);

        state.event_handler.await.unwrap();
        state.vpn_service_handle.await.unwrap();
    }
}

impl NymVpnService {
    async fn wait_for_disconnect(
        &self,
        mut tunnel_state_rx: tokio::sync::watch::Receiver<Option<TunnelState>>,
    ) {
        let _ = tokio::time::timeout(
            VPN_DISCONNECT_TIMEOUT,
            tunnel_state_rx.wait_for(|tunnel_state| {
                matches!(
                    tunnel_state,
                    Some(TunnelState::Disconnected | TunnelState::Offline { .. })
                )
            }),
        )
        .await;
    }
}
