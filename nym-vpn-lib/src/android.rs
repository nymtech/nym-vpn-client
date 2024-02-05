// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::gateway_client::{EntryPoint, ExitPoint};
use crate::{spawn_nym_vpn, NymVpn, NymVpnCtrlMessage, NymVpnExitError, NymVpnExitStatusMessage};
use futures::StreamExt;
use jnix::jni::objects::{JClass, JObject, JString};
use jnix::jni::JNIEnv;
use jnix::IntoJava;
use jnix::{FromJava, JnixEnv};
use lazy_static::lazy_static;
use log::{debug, error, warn};
use nym_task::manager::TaskStatus;
use std::str::FromStr;
use std::sync::Arc;
use talpid_core::mpsc::Sender;
use talpid_tunnel::tun_provider::TunConfig;
use talpid_types::android::AndroidContext;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, Notify};
use url::Url;

lazy_static! {
    static ref VPN_SHUTDOWN_HANDLE: Mutex<Option<Arc<Notify>>> = Mutex::new(None);
    static ref VPN: Mutex<Option<NymVpn>> = Mutex::new(None);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

#[derive(Eq, PartialEq, Debug)]
pub enum ClientState {
    Uninitialised,
    Connected,
    Disconnected,
}

async fn is_vpn_inited() -> bool {
    let guard = VPN.lock().await;
    guard.is_some()
}

async fn take_vpn() -> Option<NymVpn> {
    let mut guard = VPN.lock().await;
    guard.take()
}

async fn is_shutdown_handle_set() -> bool {
    VPN_SHUTDOWN_HANDLE.lock().await.is_some()
}

pub fn get_vpn_state() -> ClientState {
    if !RUNTIME.block_on(is_vpn_inited()) {
        ClientState::Uninitialised
    } else if RUNTIME.block_on(is_shutdown_handle_set()) {
        ClientState::Connected
    } else {
        ClientState::Disconnected
    }
}

async fn set_inited_vpn(vpn: NymVpn) {
    let mut guard = VPN.lock().await;
    if guard.is_some() {
        panic!("vpn was already inited");
    }
    *guard = Some(vpn)
}

async fn set_shutdown_handle(handle: Arc<Notify>) {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if guard.is_some() {
        panic!("vpn wasn't properly stopped")
    }
    *guard = Some(handle)
}

async fn stop_and_reset_shutdown_handle() {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if let Some(sh) = &*guard {
        sh.notify_waiters()
    } else {
        panic!("client wasn't properly started")
    }

    *guard = None
}

async fn _async_run_vpn(vpn: NymVpn) -> anyhow::Result<()> {
    let stop_handle = Arc::new(Notify::new());
    set_shutdown_handle(stop_handle.clone()).await;

    let mut handle = spawn_nym_vpn(vpn)?;

    match handle
        .vpn_status_rx
        .next()
        .await
        .ok_or(crate::Error::NotStarted)?
        .downcast_ref::<TaskStatus>()
        .ok_or(crate::Error::NotStarted)?
    {
        TaskStatus::Ready => debug!("Started Nym VPN"),
    }

    // wait for notify to be set...
    stop_handle.notified().await;
    handle.vpn_ctrl_tx.send(NymVpnCtrlMessage::Stop)?;
    match handle.vpn_exit_rx.await? {
        NymVpnExitStatusMessage::Failed(error) => {
            error!(
                "{:?}",
                error
                    .downcast_ref::<NymVpnExitError>()
                    .ok_or(crate::Error::StopError)?
            );
        }
        NymVpnExitStatusMessage::Stopped => debug!("Stopped Nym VPN"),
    }

    Ok(())
}

fn init_jni_logger() {
    use android_logger::{Config, FilterBuilder};
    use log::LevelFilter;

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libnymvpn")
            .with_filter(
                FilterBuilder::new()
                    .parse("debug,tungstenite=warn,mio=warn,tokio_tungstenite=warn")
                    .build(),
            ),
    );
    log::debug!("Logger initialized");
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_nymtech_uniffi_lib_NymVPN_initVPN(
    env: JNIEnv<'_>,
    _this: JObject<'_>,
    api_url: JString<'_>,
    entry_gateway: JString<'_>,
    exit_router: JString<'_>,
    vpn_service: JObject<'_>,
) {
    if get_vpn_state() != ClientState::Uninitialised {
        warn!("VPN was already inited. Try starting it");
        return;
    }

    init_jni_logger();

    let env = JnixEnv::from(env);
    let context = AndroidContext {
        jvm: Arc::new(env.get_java_vm().expect("Get JVM instance")),
        vpn_service: env
            .new_global_ref(vpn_service)
            .expect("Create global reference"),
    };
    let api_url = Url::from_str(&String::from_java(&env, api_url)).expect("Invalid url");
    let entry_gateway: EntryPoint = serde_json::from_str(&String::from_java(&env, entry_gateway))
        .expect("Invalid entry gateway");
    let exit_router: ExitPoint =
        serde_json::from_str(&String::from_java(&env, exit_router)).expect("Invalid exit router");

    let mut vpn = NymVpn::new(entry_gateway, exit_router, context);
    vpn.gateway_config.api_url = api_url;

    RUNTIME.block_on(set_inited_vpn(vpn));
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_nymtech_uniffi_lib_NymVPN_runVPN(_env: JNIEnv, _class: JClass) {
    let state = get_vpn_state();
    if state != ClientState::Disconnected {
        warn!("Invalid vpn state: {:?}", state);
        return;
    }

    let vpn = RUNTIME
        .block_on(take_vpn())
        .expect("VPN configuration was cleared before it could be used");

    RUNTIME.spawn(async move {
        _async_run_vpn(vpn)
            .await
            .map_err(|err| {
                warn!("failed to run vpn: {}", err);
            })
            .ok();
    });
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_net_nymtech_uniffi_lib_NymVPN_stopVPN(_env: JNIEnv, _class: JClass) {
    if get_vpn_state() != ClientState::Connected {
        warn!("could not stop the vpn as it's not running");
        return;
    }
    RUNTIME.block_on(stop_and_reset_shutdown_handle());
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_TalpidVpnService_defaultTunConfig<'env>(
    env: JNIEnv<'env>,
    _this: JObject<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);

    TunConfig::default().into_java(&env).forget()
}
