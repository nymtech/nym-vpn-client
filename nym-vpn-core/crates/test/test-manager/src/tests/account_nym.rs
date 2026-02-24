// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;

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
pub async fn forget_current_device(nym_client: &mut NymProxyClient) -> anyhow::Result<()> {
    log::debug!("Removing this device from account");

    nym_client.forget_account().await?;
    log::debug!("Successfully forgot account");

    Ok(())
}
