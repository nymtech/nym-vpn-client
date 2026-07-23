// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Pre-run cleanup: delete all devices registered on the test account.
//!
//! Each e2e run registers a fresh device on the shared test account, and the
//! in-run cleanup can't run when the daemon wedges, so devices accumulate until
//! the account hits `MaxDeviceReached`.

use anyhow::{Context, Result};
use bip39::Mnemonic;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{VpnAccount, VpnAccountMode},
};

pub async fn delete_all_devices(mnemonic: &str) -> Result<()> {
    let network = nym_network_defaults::NymNetworkDetails::new_mainnet();
    let client = VpnApiClient::from_network(&network, None)
        .await
        .context("Failed to create VPN API client")?;

    let mnemonic = Mnemonic::parse(mnemonic).context("Failed to parse account mnemonic")?;
    let account =
        VpnAccount::new(mnemonic, VpnAccountMode::Api).context("Failed to derive account")?;

    let devices = client
        .get_devices(&account)
        .await
        .context("Failed to list account devices")?;

    log::info!(
        "Account {} has {} registered device(s); deleting all",
        account.id(),
        devices.items.len()
    );

    let mut failures = 0;
    for device in &devices.items {
        match client
            .delete_device(&account, &device.device_identity_key)
            .await
        {
            Ok(_) => log::info!("Deleted device {}", device.device_identity_key),
            Err(err) => {
                failures += 1;
                log::warn!(
                    "Failed to delete device {}: {err:#}",
                    device.device_identity_key
                );
            }
        }
    }

    let remaining = client
        .get_devices(&account)
        .await
        .context("Failed to verify account device cleanup")?
        .items
        .len();
    validate_cleanup_result(failures, remaining)?;

    log::info!("All devices deleted and cleanup verified");
    Ok(())
}

fn validate_cleanup_result(failures: usize, remaining: usize) -> Result<()> {
    if failures > 0 || remaining > 0 {
        anyhow::bail!(
            "Device cleanup incomplete: {failures} deletion failure(s), {remaining} device(s) remaining"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_cleanup_result;

    #[test]
    fn cleanup_succeeds_only_when_all_devices_are_deleted() {
        assert!(validate_cleanup_result(0, 0).is_ok());
    }

    #[test]
    fn cleanup_fails_on_delete_error() {
        let error = validate_cleanup_result(1, 0).expect_err("delete failures must be fatal");
        assert!(error.to_string().contains("1 deletion failure"));
    }

    #[test]
    fn cleanup_fails_when_devices_remain() {
        let error = validate_cleanup_result(0, 2).expect_err("remaining devices must be fatal");
        assert!(error.to_string().contains("2 device(s) remaining"));
    }
}
