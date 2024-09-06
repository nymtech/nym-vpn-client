// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

#[cfg(target_os = "android")]
pub mod android;
pub(crate) mod error;
mod status_listener;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod swift;

use std::{
    env,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex as StdMutex,
    },
    time::SystemTime,
};

use lazy_static::lazy_static;
use log::*;
use talpid_core::mpsc::Sender;
use tokio::{
    runtime::Runtime,
    sync::{Mutex, Notify},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use url::Url;

use self::error::VpnError;
#[cfg(target_os = "ios")]
use crate::mobile::ios::tun_provider::OSTunProvider;
#[cfg(any(target_os = "ios", target_os = "android"))]
use crate::mobile::runner::WgTunnelRunner;
#[cfg(target_os = "android")]
use crate::platform::android::AndroidTunProvider;
use crate::{
    credentials::{check_credential_base58, import_credential_base58},
    gateway_directory::GatewayClient,
    platform::status_listener::VpnServiceStatusListener,
    uniffi_custom_impls::{
        BandwidthStatus, ConnectionStatus, EntryPoint, ExitPoint, ExitStatus, Location,
        NymVpnStatus, StatusEvent, TunStatus, UserAgent,
    },
    vpn::{
        spawn_nym_vpn, MixnetVpn, NymVpn, NymVpnCtrlMessage, NymVpnExitError,
        NymVpnExitStatusMessage, NymVpnHandle, SpecificVpn,
    },
};

lazy_static! {
    static ref VPN_SHUTDOWN_HANDLE: Mutex<Option<ShutdownHandle>> = Mutex::new(None);
    static ref RUNNING: AtomicBool = AtomicBool::new(false);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref LISTENER: StdMutex<Option<Arc<dyn TunnelStatusListener>>> = StdMutex::new(None);
}

enum ShutdownHandle {
    Notify(Arc<Notify>),

    #[allow(unused)]
    CancellationToken {
        join_handle: JoinHandle<()>,
        shutdown_token: CancellationToken,
    },
}

async fn set_shutdown_handle(shutdown_handle: ShutdownHandle) -> Result<(), VpnError> {
    let mut guard: tokio::sync::MutexGuard<'_, Option<ShutdownHandle>> =
        VPN_SHUTDOWN_HANDLE.lock().await;
    if guard.is_some() {
        return Err(VpnError::InvalidState {
            inner: "Vpn not stopped".to_string(),
        });
    }
    *guard = Some(shutdown_handle);

    Ok(())
}

pub(crate) fn uniffi_set_listener_status(status: StatusEvent) {
    let mut guard = LISTENER.lock().unwrap();
    if let Some(listener) = &mut *guard {
        match status {
            StatusEvent::Tun(status) => listener.on_tun_status_change(status),
            StatusEvent::Bandwidth(status) => listener.on_bandwidth_status_change(status),
            StatusEvent::NymVpn(status) => listener.on_nym_vpn_status_change(status),
            StatusEvent::Connection(status) => listener.on_connection_status_change(status),
            StatusEvent::Exit(status) => listener.on_exit_status_change(status),
        }
    }
}

async fn stop_and_reset_shutdown_handle() -> Result<(), VpnError> {
    tracing::debug!("Getting shutdown handle");
    let shutdown_handle =
        VPN_SHUTDOWN_HANDLE
            .lock()
            .await
            .take()
            .ok_or(VpnError::InvalidState {
                inner: "Vpn not started".to_string(),
            })?;

    match shutdown_handle {
        ShutdownHandle::Notify(sh) => {
            tracing::debug!("Notifying waiters");
            sh.notify_waiters();
            tracing::debug!("Waiting for waiters to be notified");
            sh.notified().await;
            tracing::debug!("Waiters notified");
        }
        ShutdownHandle::CancellationToken {
            join_handle,
            shutdown_token,
        } => {
            tracing::debug!("Cancel shutdown token.");
            shutdown_token.cancel();
            if let Err(e) = join_handle.await {
                tracing::error!("Failed to join on shutdown handle task: {}", e);
            }
        }
    }

    tracing::debug!("VPN shutdown handle reset");
    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
    Ok(())
}

async fn reset_shutdown_handle() {
    let _ = VPN_SHUTDOWN_HANDLE.lock().await.take();
    debug!("VPN shutdown handle reset");
}

async fn _async_run_vpn(vpn: SpecificVpn) -> Result<(Arc<Notify>, NymVpnHandle), VpnError> {
    debug!("creating new stop handle");
    let stop_handle = Arc::new(Notify::new());
    debug!("new stop handle created");
    set_shutdown_handle(ShutdownHandle::Notify(stop_handle.clone())).await?;
    debug!("shutdown handle set with new stop handle");
    let handle = spawn_nym_vpn(vpn)?;
    debug!("spawned vpn handle");
    Ok((stop_handle, handle))
}

async fn wait_for_shutdown(
    stop_handle: Arc<Notify>,
    handle: NymVpnHandle,
) -> crate::error::Result<()> {
    let NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    } = handle;

    RUNTIME.spawn(async move {
        stop_handle.notified().await;
        vpn_ctrl_tx.send(NymVpnCtrlMessage::Stop)
    });

    RUNTIME.spawn(async move {
        VpnServiceStatusListener::new().start(vpn_status_rx).await;
    });

    match vpn_exit_rx.await? {
        NymVpnExitStatusMessage::Failed(error) => {
            debug!("received exit status message for vpn");
            RUNNING.store(false, Ordering::Relaxed);
            let error = error
                .downcast_ref::<NymVpnExitError>()
                .ok_or(crate::Error::StopError)?;
            uniffi_set_listener_status(StatusEvent::Exit(error.into()));
            error!("Stopped Nym VPN with error: {:?}", error);
        }
        NymVpnExitStatusMessage::Stopped => {
            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Stopped));
            debug!("Stopped Nym VPN")
        }
    }
    Ok(())
}

#[derive(uniffi::Record)]
pub struct VPNConfig {
    pub api_url: Url,
    pub vpn_api_url: Option<Url>,
    pub entry_gateway: EntryPoint,
    pub exit_router: ExitPoint,
    pub enable_two_hop: bool,
    #[cfg(target_os = "android")]
    pub tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "ios")]
    pub tun_provider: Arc<dyn OSTunProvider>,
    pub credential_data_path: Option<PathBuf>,
    pub tun_status_listener: Option<Arc<dyn TunnelStatusListener>>,
}

fn sync_run_vpn(config: VPNConfig) -> Result<NymVpn<MixnetVpn>, VpnError> {
    let mut vpn = NymVpn::new_mixnet_vpn(
        config.entry_gateway.into(),
        config.exit_router.into(),
        #[cfg(target_os = "android")]
        config.tun_provider,
        #[cfg(target_os = "ios")]
        config.tun_provider,
    );
    debug!("Created new mixnet vpn");
    vpn.generic_config.gateway_config.api_url = config.api_url;
    vpn.generic_config.gateway_config.nym_vpn_api_url = config.vpn_api_url;
    vpn.generic_config
        .data_path
        .clone_from(&config.credential_data_path);
    Ok(vpn)
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn startVPN(config: VPNConfig) -> Result<(), VpnError> {
    if RUNNING.fetch_or(true, Ordering::Relaxed) {
        uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::InvalidStateError {
            message: "Vpn already running".to_string(),
        }));
        return Ok(());
    }

    LISTENER
        .lock()
        .unwrap()
        .clone_from(&config.tun_status_listener);

    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::InitializingClient));

    if config.enable_two_hop {
        RUNTIME.block_on(async move {
            tracing::debug!("Starting VPN tunnel...");

            let shutdown_token = CancellationToken::new();
            let clone_shutdown_token = shutdown_token.clone();
            let clone_shutdown_token2 = shutdown_token.clone();

            let join_handle = tokio::spawn(async move {
                #[cfg(any(target_os = "android", target_os = "ios"))]
                match WgTunnelRunner::new(config, clone_shutdown_token) {
                    Ok(tun_runner) => match tun_runner.start().await {
                        Ok(_) => {
                            tracing::debug!("Tunnel runner exited.");
                            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Stopped));
                        }
                        Err(e) => {
                            tracing::error!("Tunnel runner exited with error: {}", e);
                            uniffi_set_listener_status(StatusEvent::Exit(e.into()));
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to create the tunnel runner: {}", e);
                        uniffi_set_listener_status(StatusEvent::Exit(e.into()));
                    }
                }

                uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
                RUNNING.store(false, Ordering::Relaxed);
                reset_shutdown_handle().await;
            });

            let shutdown_handle = ShutdownHandle::CancellationToken {
                join_handle,
                shutdown_token,
            };
            if let Err(e) = set_shutdown_handle(shutdown_handle).await {
                tracing::error!("Failed to set shutdown handle: {}", e);
                clone_shutdown_token2.cancel();
                uniffi_set_listener_status(StatusEvent::Exit(e.into()));
            }
        });
        Ok(())
    } else {
        debug!("Trying to run VPN");
        let vpn = sync_run_vpn(config);
        debug!("Got VPN");
        match vpn {
            Ok(vpn) => {
                let ret = RUNTIME.block_on(run_vpn(vpn.into()));
                if let Some(error) = ret.err() {
                    error!("Error running VPN");
                    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
                    uniffi_set_listener_status(StatusEvent::Exit(error.into()));
                    RUNNING.store(false, Ordering::Relaxed);
                }
            }
            Err(e) => {
                error!("Err creating VPN");
                uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
                uniffi_set_listener_status(StatusEvent::Exit(e.into()));
                RUNNING.store(false, Ordering::Relaxed);
            }
        }
        Ok(())
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn initLogger() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    info!("Setting log level: {}", log_level);
    #[cfg(target_os = "ios")]
    swift::init_logs(log_level);
    #[cfg(target_os = "android")]
    android::init_logs(log_level);
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn importCredential(credential: String, path: String) -> Result<Option<SystemTime>, VpnError> {
    RUNTIME.block_on(import_credential_from_string(&credential, &path))
}

async fn import_credential_from_string(
    credential: &str,
    path: &str,
) -> Result<Option<SystemTime>, VpnError> {
    let path_result = PathBuf::from_str(path);
    let path_buf = match path_result {
        Ok(p) => p,
        Err(_) => {
            return Err(VpnError::Credential {
                inner: "Invalid path".to_string(),
            })
        }
    };
    match import_credential_base58(credential, path_buf).await {
        Ok(time) => match time {
            None => Ok(None),
            Some(t) => Ok(Some(SystemTime::from(t))),
        },
        Err(_) => Err(VpnError::Credential {
            inner: "Invalid credential".to_string(),
        }),
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn checkCredential(credential: String) -> Result<Option<SystemTime>, VpnError> {
    RUNTIME.block_on(check_credential_string(&credential))
}

async fn check_credential_string(credential: &str) -> Result<Option<SystemTime>, VpnError> {
    check_credential_base58(credential)
        .await
        .map_err(|e| VpnError::Credential {
            inner: e.to_string(),
        })
}

async fn run_vpn(vpn: SpecificVpn) -> Result<(), VpnError> {
    match _async_run_vpn(vpn).await {
        Err(err) => {
            debug!("Stopping and resetting shutdown handle");
            reset_shutdown_handle().await;
            RUNNING.store(false, Ordering::Relaxed);
            error!("Could not start the VPN: {:?}", err);
            uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
            uniffi_set_listener_status(StatusEvent::Exit(err.into()));
            Ok(())
        }
        Ok((stop_handle, handle)) => {
            debug!("Spawning wait for shutdown");
            RUNTIME.spawn(async move {
                wait_for_shutdown(stop_handle.clone(), handle)
                    .await
                    .map_err(|err| {
                        warn!("error during vpn run: {}", err);
                    })
                    .ok();
                stop_handle.notify_one();
            });
            Ok(())
        }
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn stopVPN() -> Result<(), VpnError> {
    if !RUNNING.fetch_and(false, Ordering::Relaxed) {
        return Err(VpnError::InvalidState {
            inner: "Vpn not started".to_string(),
        });
    }
    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Disconnecting));
    debug!("Stopping VPN");

    RUNTIME.block_on(stop_vpn())?;

    Ok(())
}

async fn stop_vpn() -> Result<(), VpnError> {
    debug!("Resetting shutdown handle");
    stop_and_reset_shutdown_handle().await
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getGatewayCountries(
    api_url: Url,
    nym_vpn_api_url: Option<Url>,
    exit_only: bool,
    user_agent: Option<UserAgent>,
) -> Result<Vec<Location>, VpnError> {
    RUNTIME.block_on(get_gateway_countries(
        api_url,
        nym_vpn_api_url,
        exit_only,
        user_agent,
    ))
}

async fn get_gateway_countries(
    api_url: Url,
    nym_vpn_api_url: Option<Url>,
    exit_only: bool,
    user_agent: Option<UserAgent>,
) -> Result<Vec<Location>, VpnError> {
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(|| nym_bin_common::bin_info_local_vergen!().into());
    let directory_config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url,
        min_gateway_performance: None,
    };
    let directory_client = GatewayClient::new(directory_config, user_agent)?;
    let locations = if !exit_only {
        directory_client.lookup_entry_countries().await
    } else {
        directory_client.lookup_exit_countries().await
    }?;
    Ok(locations.into_iter().map(Location::from).collect())
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getLowLatencyEntryCountry(
    api_url: Url,
    vpn_api_url: Option<Url>,
    harbour_master_url: Option<Url>,
) -> Result<Location, VpnError> {
    RUNTIME.block_on(get_low_latency_entry_country(
        api_url,
        vpn_api_url,
        harbour_master_url,
        None,
    ))
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getLowLatencyEntryCountryUserAgent(
    api_url: Url,
    vpn_api_url: Option<Url>,
    harbour_master_url: Option<Url>,
    user_agent: UserAgent,
) -> Result<Location, VpnError> {
    RUNTIME.block_on(get_low_latency_entry_country(
        api_url,
        vpn_api_url,
        harbour_master_url,
        Some(user_agent),
    ))
}

async fn get_low_latency_entry_country(
    api_url: Url,
    vpn_api_url: Option<Url>,
    _harbour_master_url: Option<Url>,
    user_agent: Option<UserAgent>,
) -> Result<Location, VpnError> {
    let config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url: vpn_api_url,
        min_gateway_performance: None,
    };
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(|| nym_bin_common::bin_info_local_vergen!().into());
    let gateway_client = GatewayClient::new(config, user_agent)?;
    let gateway = gateway_client.lookup_low_latency_entry_gateway().await?;
    let country = gateway
        .location
        .ok_or(crate::Error::CountryCodeNotFound)?
        .into();

    Ok(country)
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_tun_status_change(&self, status: TunStatus);
    fn on_bandwidth_status_change(&self, status: BandwidthStatus);
    fn on_connection_status_change(&self, status: ConnectionStatus);
    fn on_nym_vpn_status_change(&self, status: NymVpnStatus);
    fn on_exit_status_change(&self, status: ExitStatus);
}
