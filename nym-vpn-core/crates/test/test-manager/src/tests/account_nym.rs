// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::tests::helpers_nym::{login_with_retries};

use super::{Error, TestContext};
use anyhow::Context;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::time::Duration;
use test_rpc::{NymServiceClient, ServiceClient};

// TODO dz implement for Nym
// /// Log out and remove the current device
// /// from the account.
// #[test_function(priority = 100)]
// pub async fn test_logout(
//     _: TestContext,
//     _rpc: NymServiceClientServiceClient,
//     mut mullvad_client: MullvadProxyClient,
// ) -> Result<(), Error> {
//     log::info!("Removing device");
// 
//     mullvad_client
//         .logout_account()
//         .await
//         .expect("logout failed");
// 
//     Ok(())
// }
// 
// async fn get_current_wireguard_key(
//     mullvad_client: &mut MullvadProxyClient,
// ) -> anyhow::Result<PublicKey> {
//     let pubkey = mullvad_client
//         .get_device()
//         .await?
//         .logged_in()
//         .context("Client is not logged in to a valid account")?
//         .device
//         .pubkey;
//     Ok(pubkey)
// }

/// Remove all devices on the current account
pub async fn clear_devices(nym_client: &mut NymProxyClient) -> anyhow::Result<()> {
    log::debug!("Removing all devices for account");

    // TODO dz there is no explicit way to remove a device through vpn client
    // The low-level API client does have the capability:
    // - update_device(account, device, DeviceStatus::DeleteMe) exists in nym-vpn-core/crates/nym-vpn-api-client/src/client.rs
    // - DeviceStatus::DeleteMe is available as a status option
    // In the meantime, we forget the account (but that does NOT deregister device from the server)

    nym_client.forget_account().await?;
    log::debug!("Successfully forgot account");

    Ok(())
}

