// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::GatewayDirectoryError,
    wg_gateway_client::{GatewayData, WgGatewayClient},
};
use nym_gateway_directory::GatewayClient;

/// Register WG client public key with the WG gateway.
///
/// Returns gateway data that can be used to configure the wireguard interface and peer.
pub async fn register_client_pubkey(
    gateway_client: &GatewayClient,
    wg_gateway_client: &mut WgGatewayClient,
) -> crate::Result<GatewayData> {
    tracing::info!("Registering with wireguard gateway");
    let gateway_auth_recipient = wg_gateway_client
        .auth_recipient()
        .gateway()
        .to_base58_string();
    let gateway_host = gateway_client
        .lookup_gateway_ip(&gateway_auth_recipient)
        .await
        .map_err(|source| GatewayDirectoryError::FailedToLookupGatewayIp {
            gateway_id: gateway_auth_recipient,
            source,
        })?;

    let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
    tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");

    Ok(wg_gateway_data)
}
