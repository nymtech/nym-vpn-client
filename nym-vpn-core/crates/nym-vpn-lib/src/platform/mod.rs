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
use nym_vpn_api_client::types::VpnApiAccount;
use nym_vpn_store::mnemonic::MnemonicStorage as _;
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
        BandwidthStatus, ConnectionStatus, EntryPoint, ExitPoint, ExitStatus,
        GatewayMinPerformance, GatewayType, Location, NymVpnStatus, StatusEvent, TunStatus,
        UserAgent,
    },
    vpn::{
        spawn_nym_vpn, MixnetVpn, NymVpn, NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnHandle,
        SpecificVpn,
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
        return Err(VpnError::InvalidStateError {
            details: "Vpn in an invalid state, trying to set the shutdown handle when the vpn is not stopped".to_string(),
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
            StatusEvent::Exit(status) => {
                listener.on_exit_status_change(status);
                //Exit errors will always mean tunnel is down
                listener.on_tun_status_change(TunStatus::Down);
            }
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
            .ok_or(VpnError::InternalError {
                details: "Vpn in an invalid state, trying to reset the shutdown handle when the vpn is not started".to_string(),
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

    match vpn_exit_rx
        .await
        .map_err(|_| crate::Error::NymVpnExitUnexpectedChannelClose)?
    {
        NymVpnExitStatusMessage::Failed(error) => {
            debug!("received exit status message for vpn");
            RUNNING.store(false, Ordering::Relaxed);
            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure {
                error: VpnError::InternalError {
                    details: error.to_string(),
                },
            }));
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
        tracing::warn!("VPN already running");
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
            let _clone_shutdown_token = shutdown_token.clone();
            let _clone_shutdown_token2 = shutdown_token.clone();

            let join_handle = tokio::spawn(async move {
                #[cfg(any(target_os = "android", target_os = "ios"))]
                match WgTunnelRunner::new(config, _clone_shutdown_token) {
                    Ok(tun_runner) => match tun_runner.start().await {
                        Ok(_) => {
                            tracing::debug!("Tunnel runner exited.");
                            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Stopped));
                        }
                        Err(e) => {
                            tracing::error!("Tunnel runner exited with error: {}", e);
                            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure {
                                error: e.into(),
                            }));
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to create the tunnel runner: {}", e);
                        uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure {
                            error: e.into(),
                        }));
                    }
                }

                RUNNING.store(false, Ordering::Relaxed);
                reset_shutdown_handle().await;
            });

            let shutdown_handle = ShutdownHandle::CancellationToken {
                join_handle,
                shutdown_token,
            };
            if let Err(e) = set_shutdown_handle(shutdown_handle).await {
                tracing::error!("Failed to set shutdown handle: {}", e);
                _clone_shutdown_token2.cancel();
                uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure { error: e }));
            }
        });
        Ok(())
    } else {
        let vpn = sync_run_vpn(config);
        match vpn {
            Ok(vpn) => {
                let ret = RUNTIME.block_on(run_vpn(vpn.into()));
                if let Some(error) = ret.err() {
                    error!("Error running VPN {error}");
                    uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure { error }));
                    RUNNING.store(false, Ordering::Relaxed);
                }
            }
            Err(e) => {
                error!("Err creating VPN {e}");
                uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure { error: e }));
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
        Err(e) => {
            return Err(VpnError::InternalError {
                details: e.to_string(),
            })
        }
    };
    match import_credential_base58(credential, path_buf).await {
        Ok(time) => match time {
            None => Ok(None),
            Some(t) => Ok(Some(SystemTime::from(t))),
        },
        Err(e) => Err(VpnError::InvalidCredential {
            details: e.to_string(),
        }),
    }
}

fn setup_account_storage(path: &str) -> Result<crate::storage::VpnClientOnDiskStorage, VpnError> {
    let path = PathBuf::from_str(path).map_err(|err| VpnError::InternalError {
        details: err.to_string(),
    })?;
    Ok(crate::storage::VpnClientOnDiskStorage::new(path))
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn storeAccountMnemonic(mnemonic: String, path: String) -> Result<(), VpnError> {
    RUNTIME.block_on(store_account_mnemonic(&mnemonic, &path))
}

async fn store_account_mnemonic(mnemonic: &str, path: &str) -> Result<(), VpnError> {
    let storage = setup_account_storage(path)?;

    let mnemonic = nym_vpn_store::mnemonic::Mnemonic::parse(mnemonic).map_err(|err| {
        VpnError::InternalError {
            details: err.to_string(),
        }
    })?;

    storage
        .store_mnemonic(mnemonic)
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

    Ok(())
}

#[allow(non_snake_case)]
pub fn isAccountMnemonicStored(path: String) -> Result<bool, VpnError> {
    RUNTIME.block_on(is_account_mnemonic_stored(&path))
}

async fn is_account_mnemonic_stored(path: &str) -> Result<bool, VpnError> {
    let storage = setup_account_storage(path)?;
    storage
        .is_mnemonic_stored()
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}

#[allow(non_snake_case)]
pub fn removeAccountMnemonic(path: String) -> Result<bool, VpnError> {
    RUNTIME.block_on(remove_account_mnemonic(&path))
}

async fn remove_account_mnemonic(path: &str) -> Result<bool, VpnError> {
    let storage = setup_account_storage(path)?;
    storage
        .remove_mnemonic()
        .await
        .map(|_| true)
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
}

#[allow(non_snake_case)]
pub fn getAccountSummary(
    path: String,
    nym_vpn_api_url: Url,
    user_agent: UserAgent,
) -> Result<String, VpnError> {
    RUNTIME.block_on(get_account_summary(path, nym_vpn_api_url, user_agent))
}

async fn get_account_summary(
    path: String,
    nym_vpn_api_url: Url,
    user_agent: UserAgent,
) -> Result<String, VpnError> {
    let storage = setup_account_storage(&path)?;
    let account = storage
        .load_mnemonic()
        .await
        .map(VpnApiAccount::from)
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })?;

    let api_client = nym_vpn_api_client::VpnApiClient::new(nym_vpn_api_url, user_agent.into())?;

    api_client
        .get_account_summary(&account)
        .await
        .map_err(|err| VpnError::InternalError {
            details: err.to_string(),
        })
        .and_then(|summary| {
            serde_json::to_string(&summary).map_err(|err| VpnError::InternalError {
                details: err.to_string(),
            })
        })
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn checkCredential(credential: String) -> Result<Option<SystemTime>, VpnError> {
    RUNTIME.block_on(check_credential_string(&credential))
}

async fn check_credential_string(credential: &str) -> Result<Option<SystemTime>, VpnError> {
    check_credential_base58(credential)
        .await
        .map_err(|e| VpnError::InvalidCredential {
            details: e.to_string(),
        })
}

async fn run_vpn(vpn: SpecificVpn) -> Result<(), VpnError> {
    match _async_run_vpn(vpn).await {
        Err(err) => {
            debug!("Stopping and resetting shutdown handle");
            reset_shutdown_handle().await;
            RUNNING.store(false, Ordering::Relaxed);
            error!("Could not start the VPN: {:?}", err);
            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failure { error: err }));
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
        return Err(VpnError::InvalidStateError {
            details: "Vpn not started".to_string(),
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
    gw_type: GatewayType,
    user_agent: Option<UserAgent>,
    min_gateway_performance: Option<GatewayMinPerformance>,
) -> Result<Vec<Location>, VpnError> {
    RUNTIME.block_on(get_gateway_countries(
        api_url,
        nym_vpn_api_url,
        gw_type,
        user_agent,
        min_gateway_performance,
    ))
}

async fn get_gateway_countries(
    api_url: Url,
    nym_vpn_api_url: Option<Url>,
    gw_type: GatewayType,
    user_agent: Option<UserAgent>,
    min_gateway_performance: Option<GatewayMinPerformance>,
) -> Result<Vec<Location>, VpnError> {
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(crate::util::construct_user_agent);
    let min_gateway_performance = min_gateway_performance.map(|p| p.try_into()).transpose()?;
    let directory_config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url,
        min_gateway_performance,
    };
    GatewayClient::new(directory_config, user_agent)?
        .lookup_countries(gw_type.into())
        .await
        .map(|countries| countries.into_iter().map(Location::from).collect())
        .map_err(VpnError::from)
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn getLowLatencyEntryCountry(
    api_url: Url,
    vpn_api_url: Option<Url>,
    user_agent: UserAgent,
) -> Result<Location, VpnError> {
    RUNTIME.block_on(get_low_latency_entry_country(
        api_url,
        vpn_api_url,
        Some(user_agent),
    ))
}

async fn get_low_latency_entry_country(
    api_url: Url,
    vpn_api_url: Option<Url>,
    user_agent: Option<UserAgent>,
) -> Result<Location, VpnError> {
    let config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url: vpn_api_url,
        min_gateway_performance: None,
    };
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(crate::util::construct_user_agent);

    GatewayClient::new(config, user_agent)?
        .lookup_low_latency_entry_gateway()
        .await
        .map_err(VpnError::from)
        .and_then(|gateway| {
            gateway.location.ok_or(VpnError::InternalError {
                details: "gateway does not contain a two character country ISO".to_string(),
            })
        })
        .map(Location::from)
}

#[uniffi::export(with_foreign)]
pub trait TunnelStatusListener: Send + Sync {
    fn on_tun_status_change(&self, status: TunStatus);
    fn on_bandwidth_status_change(&self, status: BandwidthStatus);
    fn on_connection_status_change(&self, status: ConnectionStatus);
    fn on_nym_vpn_status_change(&self, status: NymVpnStatus);
    fn on_exit_status_change(&self, status: ExitStatus);
}
