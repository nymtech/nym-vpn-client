// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::gateway_client::{EntryPoint, ExitPoint};
use crate::{spawn_nym_vpn, NymVpn};
use futures::StreamExt;
use jnix::jni::objects::{JObject, JString};
use jnix::jni::JNIEnv;
use jnix::{FromJava, JnixEnv};
use lazy_static::lazy_static;
use log::{debug, warn};
use nym_task::manager::TaskStatus;
use std::str::FromStr;
use std::sync::Arc;
use talpid_types::android::AndroidContext;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, Notify};
use url::Url;

lazy_static! {
    static ref VPN_SHUTDOWN_HANDLE: Mutex<Option<Arc<Notify>>> = Mutex::new(None);
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

async fn set_shutdown_handle(handle: Arc<Notify>) {
    let mut guard = VPN_SHUTDOWN_HANDLE.lock().await;
    if guard.is_some() {
        panic!("vpn wasn't properly stopped")
    }
    *guard = Some(handle)
}

async fn _async_run_vpn(vpn: NymVpn) -> anyhow::Result<()> {
    let stop_handle = Arc::new(Notify::new());
    set_shutdown_handle(stop_handle.clone()).await;

    let mut handle = spawn_nym_vpn(vpn)?;

    match handle
        .vpn_status_rx
        .next()
        .await
        .ok_or(crate::Error::VPNNotStarted)?
        .downcast_ref::<TaskStatus>()
        .ok_or(crate::Error::VPNNotStarted)?
    {
        TaskStatus::Ready => debug!("Started Nym VPN"),
    }

    // wait for notify to be set...
    stop_handle.notified().await;

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
pub extern "system" fn Java_net_nymtech_uniffi_lib_NymVPN_runVPN(
    env: JNIEnv<'_>,
    _this: JObject<'_>,
    api_url: JString<'_>,
    entry_gateway: JString<'_>,
    exit_router: JString<'_>,
    vpn_service: JObject<'_>,
) {
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

    RUNTIME.spawn(async move {
        _async_run_vpn(vpn)
            .await
            .map_err(|err| {
                warn!("failed to run vpn: {}", err);
            })
            .ok();
    });
}
