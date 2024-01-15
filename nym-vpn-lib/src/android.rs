// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;
use jnix::jni::objects::JObject;
use jnix::jni::JNIEnv;
use jnix::JnixEnv;
use std::sync::Arc;
use talpid_types::android::AndroidContext;

pub fn create_android_context(
    env: &JNIEnv<'_>,
    vpn_service: JObject<'_>,
) -> Result<AndroidContext, Error> {
    let env = JnixEnv::from(env);
    Ok(AndroidContext {
        jvm: Arc::new(env.get_java_vm().map_err(Error::GetJvmInstance)?),
        vpn_service: env
            .new_global_ref(vpn_service)
            .map_err(Error::CreateGlobalReference)?,
    })
}
