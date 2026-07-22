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

    if failures > 0 {
        log::warn!("{failures} device(s) could not be deleted");
    } else {
        log::info!("All devices deleted");
    }

    Ok(())
}
