// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::NonNull;

use objc2::rc::Retained;
use objc2_core_foundation::CFString;
use objc2_security::{SecCSFlags, SecCode, SecRequirement, errSecSuccess};
use tokio_stream::Stream;

#[cfg(any(not(debug_assertions), feature = "xpc"))]
use crate::xpc::{common::XpcConnection, daemon::XpcService};
use crate::{
    AuthenticationMaterial,
    authentication::{AuthenticationLayer, error::AuthenticationError},
};

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub type Transport = XpcConnection;
#[cfg(all(debug_assertions, not(feature = "xpc")))]
pub type Transport = tokio::net::UnixStream;

#[derive(Clone)]
pub struct SigningRequirements {
    pub daemon_req: String,
    pub client_req: String,
}

// Authentication happens in XPC layer, so if stream got through it means it's
// authenticated
pub(crate) async fn is_authenticated(
    _stream: &mut Transport,
    _auth_material: AuthenticationMaterial,
) -> Result<(), AuthenticationError> {
    Ok(())
}

#[allow(unused)]
fn self_is_signed(signing_requirement: &str) -> bool {
    let mut raw_sec_code: *mut SecCode = std::ptr::null_mut();
    let status =
        unsafe { SecCode::copy_self(SecCSFlags::DefaultFlags, NonNull::from(&mut raw_sec_code)) };
    if status != errSecSuccess {
        tracing::error!("Could not obtain self code");
        return false;
    }
    let ret = unsafe { Retained::from_raw(raw_sec_code) };
    let Some(sec_code) = ret else {
        tracing::error!("SecCodeCopySelf returned null on success");
        return false;
    };

    let mut raw_sec_req: *mut SecRequirement = std::ptr::null_mut();
    let status = unsafe {
        SecRequirement::create_with_string(
            &CFString::from_str(signing_requirement),
            SecCSFlags::DefaultFlags,
            NonNull::from(&mut raw_sec_req),
        )
    };
    if status != errSecSuccess {
        tracing::error!("Could not create a SecRequirement");
        return false;
    }
    let ret = unsafe { Retained::from_raw(raw_sec_req) };
    let Some(sec_req) = ret else {
        tracing::error!("Creating a SecRequirement returned null on success");
        return false;
    };

    let status = unsafe { sec_code.check_validity(SecCSFlags::DefaultFlags, Some(&sec_req)) };
    tracing::debug!("Daemon signature validation check: {status}");
    status == errSecSuccess
}

#[allow(unused)]
pub(crate) fn skip_authentication_checks(auth_material: &AuthenticationMaterial) -> bool {
    auth_material.disable_client_verification
        || !self_is_signed(&auth_material.signing_requirements.daemon_req)
}

#[cfg(any(not(debug_assertions), feature = "xpc"))]
pub(crate) fn incoming(xpc_service: XpcService) -> impl Stream<Item = std::io::Result<Transport>> {
    // XPC has built in authentication mechanism
    let auth_layer = AuthenticationLayer::new(xpc_service, None);
    auth_layer.stream()
}

#[cfg(all(debug_assertions, not(feature = "xpc")))]
pub(crate) fn incoming(
    uds: crate::uds::Uds,
    _auth_material: AuthenticationMaterial,
) -> impl Stream<Item = std::io::Result<Transport>> {
    // No authentication mechanism for MacOS UDS
    let auth_layer = AuthenticationLayer::new(uds, None);
    auth_layer.stream()
}
