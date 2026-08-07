// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use async_compression::tokio::bufread::GzipDecoder;
use nym_vpn_api_client::response::NymDirectoryGateway;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, BufReader};

use crate::GatewayType;

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

async fn decode_builtin() -> anyhow::Result<Vec<BuiltinEntry>> {
    let mut decompressed = Vec::new();
    GzipDecoder::new(BufReader::new(BUILTIN_GATEWAYS))
        .read_to_end(&mut decompressed)
        .await?;

    Ok(serde_json::from_slice(&decompressed)?)
}

/// The builtin snapshot, already decompressed and deserialized. Decoding it is not free — it's
/// ~561 gateway objects, each with nested location/probe/performance data — so callers that need
/// more than one [`GatewayType`] out of it (see `crate::gateway_store::seed_all`) should decode
/// once via [`load_builtin_snapshot`] and reuse this, rather than decoding it again per type.
pub(crate) struct BuiltinSnapshot(Vec<BuiltinEntry>);

impl BuiltinSnapshot {
    pub(crate) fn raw_gateways(&self, gw_type: GatewayType) -> Vec<NymDirectoryGateway> {
        self.0
            .iter()
            .filter(|entry| entry.types.iter().any(|t| t.matches(gw_type)))
            .map(|entry| entry.gateway.clone())
            .collect()
    }
}

pub(crate) async fn load_builtin_snapshot() -> anyhow::Result<BuiltinSnapshot> {
    decode_builtin().await.map(BuiltinSnapshot)
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::entries::gateway::gateways_from_directory_response;

    #[tokio::test]
    async fn builtin_snapshot_is_deserializable() {
        let entries = decode_builtin()
            .await
            .expect("committed builtin/gateways.json.gz must decompress and deserialize under the current schema");
        assert!(
            !entries.is_empty(),
            "builtin snapshot deserialized but contained no entries"
        );
    }

    #[tokio::test]
    async fn loads_non_empty_builtin_gateways_for_every_type() {
        let snapshot = load_builtin_snapshot()
            .await
            .expect("failed to load builtin snapshot");

        for gw_type in GatewayType::iter() {
            let raw = snapshot.raw_gateways(gw_type);
            let gateways = gateways_from_directory_response(raw, gw_type);
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
