// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use async_compression::tokio::bufread::GzipDecoder;
use nym_vpn_api_client::response::NymDirectoryGateway;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, BufReader};
use tracing::error;

use crate::{
    Error, GatewayType,
    entries::gateway::{Gateway, GatewayList},
    error::Result,
};

static BUILTIN_GATEWAYS: &[u8] = include_bytes!("../builtin/gateways.json.gz");

#[derive(Deserialize)]
struct BuiltinEntry {
    types: Vec<BuiltinGatewayType>,
    gateway: NymDirectoryGateway,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BuiltinGatewayType {
    Entry,
    Exit,
    Wg,
}

impl BuiltinGatewayType {
    fn matches(&self, gw_type: GatewayType) -> bool {
        matches!(
            (self, gw_type),
            (BuiltinGatewayType::Entry, GatewayType::MixnetEntry)
                | (BuiltinGatewayType::Exit, GatewayType::MixnetExit)
                | (BuiltinGatewayType::Wg, GatewayType::Wg)
        )
    }
}

/// Load the builtin gateway list for `gw_type`, applying the same conversion and filtering that
/// [`crate::gateway_client::GatewayClient::lookup_gateways`] applies to a live response.
pub(crate) async fn load_builtin_gateways(gw_type: GatewayType) -> Result<GatewayList> {
    let builtin_err = |reason: String| Error::BuiltinGatewayList { gw_type, reason };

    let mut decompressed = Vec::new();
    GzipDecoder::new(BufReader::new(BUILTIN_GATEWAYS))
        .read_to_end(&mut decompressed)
        .await
        .map_err(|err| builtin_err(format!("failed to decompress: {err}")))?;

    let entries: Vec<BuiltinEntry> = serde_json::from_slice(&decompressed)
        .map_err(|err| builtin_err(format!("failed to parse json: {err}")))?;

    let gateways: Vec<_> = entries
        .into_iter()
        .filter(|entry| entry.types.iter().any(|t| t.matches(gw_type)))
        .filter_map(|entry| {
            Gateway::try_from(entry.gateway)
                .inspect_err(|err| error!("Failed to parse builtin gateway: {err}"))
                .ok()
        })
        .filter(Gateway::not_mixnet_blacklisted)
        .collect();

    Ok(GatewayList::new(Some(gw_type), gateways))
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[tokio::test]
    async fn loads_non_empty_builtin_gateways_for_every_type() {
        for gw_type in GatewayType::iter() {
            let gateways = load_builtin_gateways(gw_type).await.unwrap_or_else(|err| {
                panic!("failed to load builtin gateways for {gw_type:?}: {err}")
            });
            assert!(
                !gateways.is_empty(),
                "expected at least one builtin gateway for {gw_type:?}"
            );
            for gateway in gateways.into_iter() {
                assert!(
                    gateway.not_mixnet_blacklisted(),
                    "builtin gateway {} for {gw_type:?} should not be mixnet-blacklisted",
                    gateway.identity()
                );
            }
        }
    }
}
