// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Various helper functions that are exposed to the FFI layer.

use std::time::Duration;

use super::{RUNTIME, error::VpnError};

/// Call that blocks until the account controller reports that we are ready to connect. This is
/// useful when you want to wait for the account to be ready before proceeding with other
/// operations.
///
/// # Errors
///
/// This function will return an error of the network environment is not set or the account.
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn waitForAccountReadyToConnect(timeout_sec: u64) -> Result<(), VpnError> {
    RUNTIME.block_on(waitForAccountReadyToConnectAsync(timeout_sec))
}

/// Async variant of waitForAccountReadyToConnect. This is useful when you want to wait for the
/// account to be ready before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error of the network environment is not set or the account.
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn waitForAccountReadyToConnectAsync(timeout_sec: u64) -> Result<(), VpnError> {
    let timeout = Duration::from_secs(timeout_sec);
    super::account::wait_for_account_ready_to_connect(timeout).await
}
