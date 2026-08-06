// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use async_compression::tokio::{bufread::GzipDecoder, write::GzipEncoder};
use nym_vpn_api_client::response::NymDirectoryGateway;
use strum::IntoEnumIterator;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

use crate::{
    Error, GatewayType, builtin,
    entries::gateway::{GatewayList, gateways_from_raw},
    error::Result,
};

fn file_name(gw_type: GatewayType) -> &'static str {
    match gw_type {
        GatewayType::MixnetEntry => "entry.json.gz",
        GatewayType::MixnetExit => "exit.json.gz",
        GatewayType::Wg => "wg.json.gz",
    }
}

/// Seed the in-memory cache for every gateway type: try the on-disk cache for each type first,
/// falling back to the builtin snapshot for whichever types aren't on disk yet. The builtin
/// snapshot is decoded at most once per call — not once per missing type — since decompressing
/// and JSON-parsing ~500+ gateway objects is real work, especially in a debug build; repeating it
/// per type was previously adding several seconds to a cold-start seed.
pub(crate) async fn seed_all(
    data_dir: &Path,
    allow_builtin_fallback: bool,
) -> Vec<(GatewayType, Result<Option<GatewayList>>)> {
    let mut results = Vec::new();
    let mut disk_misses = Vec::new();

    for gw_type in GatewayType::iter() {
        let path = data_dir.join(file_name(gw_type));
        match load_raw(&path).await {
            Ok(raw) => {
                let gateways = gateways_from_raw(raw, gw_type);
                if gateways.is_empty() {
                    // A file that parses but yields nothing usable (e.g. every entry failed
                    // conversion or is mixnet-blacklisted) is not a usable seed either — treat
                    // it the same as a missing file so callers fall through to a real fetch
                    // (and get `Error::Offline` if that fails) instead of an empty gateway list.
                    tracing::debug!(
                        "On-disk gateway cache for {gw_type:?} at '{}' parsed but yielded no usable gateways; treating as a miss",
                        path.display()
                    );
                    disk_misses.push(gw_type);
                } else {
                    tracing::debug!(
                        "Loaded {} gateways for {gw_type:?} from '{}'",
                        gateways.len(),
                        path.display()
                    );
                    results.push((gw_type, Ok(Some(gateways))));
                }
            }
            Err(err) => {
                tracing::debug!(
                    "No usable on-disk gateway cache for {gw_type:?} at '{}': {err}",
                    path.display()
                );
                disk_misses.push(gw_type);
            }
        }
    }

    if disk_misses.is_empty() {
        return results;
    }

    if !allow_builtin_fallback {
        for gw_type in disk_misses {
            tracing::debug!(
                "Not seeding {gw_type:?} from builtin: not applicable for this network (mainnet-only)"
            );
            results.push((gw_type, Ok(None)));
        }
        return results;
    }

    match builtin::load_builtin_snapshot().await {
        Ok(snapshot) => {
            for gw_type in disk_misses {
                let raw = snapshot.raw_gateways(gw_type);
                let gateways = gateways_from_raw(raw.clone(), gw_type);

                if gateways.is_empty() {
                    tracing::warn!(
                        "Builtin snapshot yielded no usable gateways for {gw_type:?}; not seeding"
                    );
                    results.push((gw_type, Ok(None)));
                    continue;
                }

                let path = data_dir.join(file_name(gw_type));
                if let Err(err) = save_raw(&path, &raw).await {
                    tracing::warn!(
                        "Failed to seed on-disk gateway cache for {gw_type:?} at '{}': {err}",
                        path.display()
                    );
                }
                results.push((gw_type, Ok(Some(gateways))));
            }
        }
        Err(err) => {
            for gw_type in disk_misses {
                results.push((
                    gw_type,
                    Err(Error::BuiltinGatewayList {
                        gw_type,
                        reason: err.to_string(),
                    }),
                ));
            }
        }
    }

    results
}

pub(crate) async fn save(
    data_dir: &Path,
    gw_type: GatewayType,
    raw: &[NymDirectoryGateway],
) -> Result<()> {
    save_raw(&data_dir.join(file_name(gw_type)), raw).await
}

async fn load_raw(path: &Path) -> anyhow::Result<Vec<NymDirectoryGateway>> {
    let bytes = fs::read(path).await?;

    let mut decompressed = Vec::new();
    GzipDecoder::new(BufReader::new(bytes.as_slice()))
        .read_to_end(&mut decompressed)
        .await?;

    Ok(serde_json::from_slice(&decompressed)?)
}

async fn save_raw(path: &Path, raw: &[NymDirectoryGateway]) -> Result<()> {
    let store_err = |reason: String| Error::GatewayStore {
        path: path.to_path_buf(),
        reason,
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| store_err(format!("failed to create directory: {err}")))?;
    }

    let json =
        serde_json::to_vec(raw).map_err(|err| store_err(format!("failed to serialize: {err}")))?;

    let mut encoder = GzipEncoder::new(Vec::new());
    encoder
        .write_all(&json)
        .await
        .map_err(|err| store_err(format!("failed to compress: {err}")))?;
    encoder
        .shutdown()
        .await
        .map_err(|err| store_err(format!("failed to compress: {err}")))?;

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, encoder.into_inner())
        .await
        .map_err(|err| store_err(format!("failed to write: {err}")))?;
    fs::rename(&tmp, path)
        .await
        .map_err(|err| store_err(format!("failed to rename into place: {err}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;
    use tempfile::tempdir;

    use super::*;

    fn find(
        results: &[(GatewayType, Result<Option<GatewayList>>)],
        gw_type: GatewayType,
    ) -> &Result<Option<GatewayList>> {
        &results
            .iter()
            .find(|(t, _)| *t == gw_type)
            .expect("missing result for gateway type")
            .1
    }

    #[tokio::test]
    async fn seeds_from_builtin_when_disk_cache_is_absent_and_allowed() {
        let dir = tempdir().unwrap();

        for gw_type in GatewayType::iter() {
            assert!(!dir.path().join(file_name(gw_type)).exists());
        }

        let results = seed_all(dir.path(), true).await;

        for gw_type in GatewayType::iter() {
            let gateways = find(&results, gw_type)
                .as_ref()
                .unwrap()
                .clone()
                .expect("expected builtin-seeded gateways");
            assert!(
                !gateways.is_empty(),
                "expected builtin-seeded gateways for {gw_type:?}"
            );
            let path = dir.path().join(file_name(gw_type));
            assert!(
                path.exists(),
                "expected seed_all to write the seeded data to '{}'",
                path.display()
            );
        }
    }

    #[tokio::test]
    async fn does_not_seed_from_builtin_when_not_allowed() {
        let dir = tempdir().unwrap();
        let gw_type = GatewayType::MixnetExit;
        let path = dir.path().join(file_name(gw_type));

        let results = seed_all(dir.path(), false).await;

        let result = find(&results, gw_type).as_ref().unwrap();
        assert!(
            result.is_none(),
            "expected no seed data when builtin fallback is disallowed and no disk cache exists"
        );
        assert!(
            !path.exists(),
            "expected seed_all not to write anything to disk when it can't seed"
        );
    }

    #[tokio::test]
    async fn treats_empty_on_disk_cache_as_a_miss() {
        let dir = tempdir().unwrap();
        let gw_type = GatewayType::MixnetExit;
        let path = dir.path().join(file_name(gw_type));

        // A file that exists and parses, but yields zero gateways (e.g. every entry failed
        // conversion or was blacklisted), must not be treated as a usable seed.
        save_raw(&path, &[]).await.unwrap();
        assert!(path.exists());

        let results = seed_all(dir.path(), true).await;
        let gateways =
            find(&results, gw_type).as_ref().unwrap().clone().expect(
                "expected the empty disk file to be ignored in favor of the builtin fallback",
            );
        assert!(!gateways.is_empty());

        // Reset to the empty file and check the not-allowed case falls through to `None` rather
        // than an empty `Some`.
        save_raw(&path, &[]).await.unwrap();
        let results = seed_all(dir.path(), false).await;
        let result = find(&results, gw_type).as_ref().unwrap();
        assert!(
            result.is_none(),
            "expected an empty on-disk cache with no builtin fallback to be treated as no seed"
        );
    }

    #[tokio::test]
    async fn persisted_data_round_trips_and_wins_over_builtin() {
        let dir = tempdir().unwrap();
        let gw_type = GatewayType::MixnetExit;

        let snapshot = builtin::load_builtin_snapshot().await.unwrap();
        let mut raw = snapshot.raw_gateways(gw_type);
        raw.truncate(1);
        let persisted_identity = raw[0].identity_key.clone();

        save(dir.path(), gw_type, &raw).await.unwrap();

        // Even with builtin fallback disallowed, the on-disk cache (from a prior successful
        // live fetch on this network) should still be used.
        let results = seed_all(dir.path(), false).await;
        let loaded = find(&results, gw_type)
            .as_ref()
            .unwrap()
            .clone()
            .expect("expected the on-disk cache to be used");
        assert_eq!(loaded.len(), 1);
        assert_eq!(
            loaded
                .into_iter()
                .next()
                .unwrap()
                .identity()
                .to_base58_string(),
            persisted_identity
        );
    }
}
