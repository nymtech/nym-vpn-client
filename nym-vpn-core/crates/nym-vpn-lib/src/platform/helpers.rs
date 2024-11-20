// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Various helper functions that are exposed to the FFI layer.

use std::time::Duration;

use super::{error::VpnError, RUNTIME};

/// Call that blocks until the account state has been updated/synced. This is useful when you want
/// to wait for the account state to be updated before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn waitForUpdateAccount() -> Result<(), VpnError> {
    RUNTIME.block_on(super::account::wait_for_update_account())
}

/// Async variant of waitForUpdateAccount. This is useful when you want to wait for the account
/// state to be updated before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn waitForUpdateAccountAsync() -> Result<(), VpnError> {
    super::account::wait_for_update_account().await
}

/// Call that blocks until the device has been updated/synced. This is useful when you want to wait
/// for the device state to be updated before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn waitForUpdateDevice() -> Result<(), VpnError> {
    RUNTIME.block_on(super::account::wait_for_update_device())
}

/// Async variant of waitForUpdateDevice. This is useful when you want to wait for the device
/// state to be updated before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn waitForUpdateDeviceAsync() -> Result<(), VpnError> {
    super::account::wait_for_update_device().await
}

/// Call that blocks until the device has been registered. This is useful when you want to wait
/// for the device to be registered before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn waitForRegisterDevice() -> Result<(), VpnError> {
    RUNTIME.block_on(super::account::wait_for_register_device())
}

/// Async variant of waitForRegisterDevice. This is useful when you want to wait for the device
/// to be registered before proceeding with other operations.
///
/// # Errors
///
/// This function will return an error if the account controller is not running.
#[allow(non_snake_case)]
#[uniffi::export]
pub async fn waitForRegisterDeviceAsync() -> Result<(), VpnError> {
    super::account::wait_for_register_device().await
}

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
    RUNTIME.block_on(async {
        let credentials_mode = super::environment::get_feature_flag_credential_mode().await?;
        let timeout = Duration::from_secs(timeout_sec);
        super::account::wait_for_account_ready_to_connect(credentials_mode, timeout).await
    })
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
    let credentials_mode = super::environment::get_feature_flag_credential_mode().await?;
    let timeout = Duration::from_secs(timeout_sec);
    super::account::wait_for_account_ready_to_connect(credentials_mode, timeout).await
}
