// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::gateway_client::{EntryPoint, ExitPoint};
use crate::{spawn_nym_vpn, NymVpn};
use jnix::jni::objects::{JObject, JString};
use jnix::jni::JNIEnv;
use jnix::{FromJava, JnixEnv};
use lazy_static::lazy_static;
use std::sync::Arc;
use talpid_types::android::AndroidContext;
use tokio::runtime::Runtime;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_runVPN(
    env: JNIEnv<'_>,
    entry_gateway: JString<'_>,
    exit_router: JString<'_>,
    vpn_service: JObject<'_>,
) {
    let env = JnixEnv::from(env);
    let context = AndroidContext {
        jvm: Arc::new(env.get_java_vm().expect("Get JVM instance")),
        vpn_service: env
            .new_global_ref(vpn_service)
            .expect("Create global reference"),
    };
    let entry_gateway: EntryPoint = serde_json::from_str(&String::from_java(&env, entry_gateway))
        .expect("Invalid entry gateway");
    let exit_router: ExitPoint =
        serde_json::from_str(&String::from_java(&env, exit_router)).expect("Invalid exit router");

    let vpn = NymVpn::new(entry_gateway, exit_router, context);

    match spawn_nym_vpn(vpn) {
        Ok(handle) => {}
        Err(e) => {
            log::error!("{:?}", e)
        }
    }
}
