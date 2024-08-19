// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use self::error::FFIError;
use crate::credentials::{check_credential_base58, import_credential_base58};
use crate::gateway_directory::GatewayClient;
use crate::platform::status_listener::VpnServiceStatusListener;
use crate::uniffi_custom_impls::{
    BandwidthStatus, ConnectionStatus, EntryPoint, ExitPoint, ExitStatus, Location, NymVpnStatus,
    StatusEvent, TunStatus, UserAgent,
};
use crate::{
    spawn_nym_vpn, NymVpn, NymVpnCtrlMessage, NymVpnExitError, NymVpnExitStatusMessage,
    NymVpnHandle, SpecificVpn,
};
use lazy_static::lazy_static;
use log::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use talpid_core::mpsc::Sender;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, Notify};
use url::Url;

#[cfg(target_os = "android")]
pub mod android;
pub(crate) mod error;
mod status_listener;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod swift;

lazy_static! {
    static ref VPN_SHUTDOWN_HANDLE: Mutex<Option<Arc<Notify>>> = Mutex::new(None);
    static ref RUNNING: AtomicBool = AtomicBool::new(false);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
    static ref LISTENER: std::sync::Mutex<Option<Arc<dyn TunnelStatusListener>>> =
        std::sync::Mutex::new(None);
}

async fn set_shutdown_handle(handle: Arc<Notify>) -> Result<(), FFIError> {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if guard.is_some() {
        return Err(FFIError::VpnNotStopped);
    }
    *guard = Some(handle);

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

async fn stop_and_reset_shutdown_handle() -> Result<(), FFIError> {
    debug!("Getting shutdown handle");
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if let Some(sh) = &*guard {
        debug!("notifying waiters");
        sh.notify_waiters();
        debug!("waiting for waiters to be notified");
        sh.notified().await;
        debug!("waiters notified");
    } else {
        return Err(FFIError::VpnNotStarted);
    }
    *guard = None;
    debug!("VPN shutdown handle reset");
    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
    Ok(())
}

async fn reset_shutdown_handle() -> Result<(), FFIError> {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    *guard = None;
    debug!("VPN shutdown handle reset");
    Ok(())
}

async fn _async_run_vpn(vpn: SpecificVpn) -> Result<(Arc<Notify>, NymVpnHandle), FFIError> {
    debug!("creating new stop handle");
    let stop_handle = Arc::new(Notify::new());
    debug!("new stop handle created");
    set_shutdown_handle(stop_handle.clone()).await?;
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
            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failed {
                error: error.to_string(),
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
    #[cfg(target_os = "ios")]
    pub tun_provider: Arc<dyn crate::OSTunProvider>,
    pub credential_data_path: Option<PathBuf>,
    pub tun_status_listener: Option<Arc<dyn TunnelStatusListener>>,
}

fn sync_run_vpn(config: VPNConfig) -> Result<SpecificVpn, FFIError> {
    #[cfg(target_os = "android")]
    let context = crate::platform::android::get_context().ok_or(FFIError::NoContext)?;
    #[cfg(target_os = "android")]
    debug!("got android context to create new vpn");

    let vpn: SpecificVpn = if config.enable_two_hop {
        debug!("Created new wireguard vpn");
        let mut vpn = NymVpn::new_wireguard_vpn(
            config.entry_gateway.into(),
            config.exit_router.into(),
            #[cfg(target_os = "android")]
            context,
            #[cfg(target_os = "ios")]
            config.tun_provider,
        );
        vpn.generic_config.gateway_config.api_url = config.api_url;
        vpn.generic_config
            .data_path
            .clone_from(&config.credential_data_path);

        vpn.into()
    } else {
        debug!("Created new mixnet vpn");
        let mut vpn = NymVpn::new_mixnet_vpn(
            config.entry_gateway.into(),
            config.exit_router.into(),
            #[cfg(target_os = "android")]
            context,
            #[cfg(target_os = "ios")]
            config.tun_provider,
        );
        vpn.generic_config.gateway_config.api_url = config.api_url;
        vpn.generic_config
            .data_path
            .clone_from(&config.credential_data_path);

        vpn.into()
    };

    Ok(vpn)
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn startVPN(config: VPNConfig) -> Result<(), FFIError> {
    if RUNNING.fetch_or(true, Ordering::Relaxed) {
        return Err(FFIError::VpnAlreadyRunning);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    crate::platform::swift::init_logs();

    LISTENER
        .lock()
        .unwrap()
        .clone_from(&config.tun_status_listener);

    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::InitializingClient));

    debug!("Trying to run VPN");
    let vpn = sync_run_vpn(config);
    debug!("Got VPN");
    if vpn.is_err() {
        error!("Err creating VPN");
        uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
        RUNNING.store(false, Ordering::Relaxed);
    }
    let ret = RUNTIME.block_on(run_vpn(vpn?.into()));
    if ret.is_err() {
        error!("Error running VPN");
        uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
        RUNNING.store(false, Ordering::Relaxed);
    }
    ret
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn importCredential(credential: String, path: String) -> Result<Option<SystemTime>, FFIError> {
    RUNTIME.block_on(import_credential_from_string(&credential, &path))
}

async fn import_credential_from_string(
    credential: &str,
    path: &str,
) -> Result<Option<SystemTime>, FFIError> {
    let path_result = PathBuf::from_str(path);
    let path_buf = match path_result {
        Ok(p) => p,
        Err(_) => return Err(FFIError::InvalidPath),
    };
    match import_credential_base58(credential, path_buf).await {
        Ok(time) => match time {
            None => Ok(None),
            Some(t) => Ok(Some(SystemTime::from(t))),
        },
        Err(_) => Err(FFIError::InvalidCredential),
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
pub fn checkCredential(credential: String) -> Result<Option<SystemTime>, FFIError> {
    RUNTIME.block_on(check_credential_string(&credential))
}

async fn check_credential_string(credential: &str) -> Result<Option<SystemTime>, FFIError> {
    check_credential_base58(credential)
        .await
        .map_err(|_| FFIError::InvalidCredential)
}

async fn run_vpn(vpn: SpecificVpn) -> Result<(), FFIError> {
    match _async_run_vpn(vpn).await {
        Err(err) => {
            debug!("Stopping and resetting shutdown handle");
            reset_shutdown_handle()
                .await
                .expect("Failed to reset shutdown handle");
            RUNNING.store(false, Ordering::Relaxed);
            error!("Could not start the VPN: {:?}", err);
            uniffi_set_listener_status(StatusEvent::Exit(ExitStatus::Failed {
                error: err.to_string(),
            }));
            uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Down));
            Err(err)
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
pub fn stopVPN() -> Result<(), FFIError> {
    if !RUNNING.fetch_and(false, Ordering::Relaxed) {
        return Err(FFIError::VpnNotStarted);
    }
    uniffi_set_listener_status(StatusEvent::Tun(TunStatus::Disconnecting));
    debug!("Stopping VPN");
    RUNTIME.block_on(stop_vpn())
}

async fn stop_vpn() -> Result<(), FFIError> {
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
) -> Result<Vec<Location>, FFIError> {
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
) -> Result<Vec<Location>, FFIError> {
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(|| nym_bin_common::bin_info!().into());
    let directory_config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url,
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
) -> Result<Location, FFIError> {
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
) -> Result<Location, FFIError> {
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
) -> Result<Location, FFIError> {
    let config = nym_gateway_directory::Config {
        api_url,
        nym_vpn_api_url: vpn_api_url,
    };
    let user_agent = user_agent
        .map(nym_sdk::UserAgent::from)
        .unwrap_or_else(|| nym_bin_common::bin_info!().into());
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
