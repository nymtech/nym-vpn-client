// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use async_compression::tokio::{bufread::GzipDecoder, write::GzipEncoder};
use nym_vpn_api_client::response::NymDirectoryGateway;
use strum::IntoEnumIterator;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

use crate::{
    Error, GatewayType, builtin,
    entries::gateway::{GatewayList, gateways_from_directory_response},
    error::Result,
};

/// A gateway list seeded from either a genuine on-disk cache or the builtin fallback snapshot.
/// The two are kept distinct because only the former carries a meaningful age: the builtin
/// snapshot is static data bundled at build time, not the result of a real fetch.
#[derive(Clone)]
pub(crate) enum SeededGateways {
    /// Loaded from an on-disk cache written by a previous run, `age` ago.
    FromDisk {
        gateways: GatewayList,
        age: Duration,
    },
    /// Seeded from the builtin (mainnet-only) fallback snapshot because no disk cache existed.
    FromBuiltin(GatewayList),
}

impl SeededGateways {
    #[cfg(test)]
    pub(crate) fn into_gateways(self) -> GatewayList {
        match self {
            SeededGateways::FromDisk { gateways, .. } => gateways,
            SeededGateways::FromBuiltin(gateways) => gateways,
        }
    }
}

fn file_name(gw_type: GatewayType) -> &'static str {
    match gw_type {
        GatewayType::MixnetEntry => "entry.json.gz",
        GatewayType::MixnetExit => "exit.json.gz",
        GatewayType::Wg => "wg.json.gz",
    }
}

/// Try to load a usable on-disk cache entry for `gw_type`, along with how long ago it was
/// written. A missing file, a read/decode error, and a file that parses but yields zero usable
/// gateways (e.g. every entry failed conversion or is mixnet-blacklisted) are all treated as a
/// cache miss, so callers fall through to a real fetch (and get `Error::Offline` if that fails)
/// instead of an empty gateway list.
async fn load_disk_cache(path: &Path, gw_type: GatewayType) -> Option<(GatewayList, Duration)> {
    let (raw, modified) = match read_from_path(path).await {
        Ok(result) => result,
        Err(err) => {
            tracing::debug!(
                "No usable on-disk gateway cache for {gw_type:?} at '{}': {err}",
                path.display()
            );
            return None;
        }
    };

    let gateways = gateways_from_directory_response(raw, gw_type);
    if gateways.is_empty() {
        tracing::debug!(
            "On-disk gateway cache for {gw_type:?} at '{}' parsed but yielded no usable gateways; treating as a miss",
            path.display()
        );
        return None;
    }

    // A modified time in the future (clock skew, moved files) is treated as "just written"
    // rather than propagating an error through the whole seed path over a cosmetic freshness
    // detail.
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    tracing::debug!(
        "Loaded {} gateways for {gw_type:?} from '{}' (age: {age:?})",
        gateways.len(),
        path.display()
    );
    Some((gateways, age))
}

/// Seed the in-memory cache for every gateway type: try the on-disk cache for each type first,
/// falling back to the builtin snapshot for whichever types aren't on disk yet. The builtin
/// snapshot is decoded at most once per call — not once per missing type — since decompressing
/// and JSON-parsing ~500+ gateway objects is real work, especially in a debug build; repeating it
/// per type was previously adding several seconds to a cold-start seed. It's loaded lazily, on the
/// first miss, so a data dir with every type already cached on disk never touches it at all.
pub(crate) async fn seed_all(
    data_dir: &Path,
    allow_builtin_fallback: bool,
) -> Vec<(GatewayType, Result<Option<SeededGateways>>)> {
    let mut results = Vec::new();
    let mut builtin_snapshot = None;

    for gw_type in GatewayType::iter() {
        let path = data_dir.join(file_name(gw_type));

        if let Some((gateways, age)) = load_disk_cache(&path, gw_type).await {
            results.push((
                gw_type,
                Ok(Some(SeededGateways::FromDisk { gateways, age })),
            ));
            continue;
        }

        if !allow_builtin_fallback {
            tracing::debug!(
                "Not seeding {gw_type:?} from builtin: not applicable for this network (mainnet-only)"
            );
            results.push((gw_type, Ok(None)));
            continue;
        }

        if builtin_snapshot.is_none() {
            builtin_snapshot = Some(builtin::load_builtin_snapshot().await);
        }

        match builtin_snapshot.as_ref().expect("just populated above") {
            Ok(snapshot) => {
                let raw = snapshot.raw_gateways(gw_type);
                let gateways = gateways_from_directory_response(raw.clone(), gw_type);

                if gateways.is_empty() {
                    tracing::warn!(
                        "Builtin snapshot yielded no usable gateways for {gw_type:?}; not seeding"
                    );
                    results.push((gw_type, Ok(None)));
                    continue;
                }

                if let Err(err) = write_to_path(&path, &raw).await {
                    tracing::warn!(
                        "Failed to seed on-disk gateway cache for {gw_type:?} at '{}': {err}",
                        path.display()
                    );
                }
                results.push((gw_type, Ok(Some(SeededGateways::FromBuiltin(gateways)))));
            }
            Err(err) => {
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
    write_to_path(&data_dir.join(file_name(gw_type)), raw).await
}

/// Read and decompress the gateway list stored at `path`, along with its last-modified time.
async fn read_from_path(path: &Path) -> anyhow::Result<(Vec<NymDirectoryGateway>, SystemTime)> {
    let mut file = fs::File::open(path).await?;
    let modified = file.metadata().await?.modified()?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;

    let mut decompressed = Vec::new();
    GzipDecoder::new(BufReader::new(bytes.as_slice()))
        .read_to_end(&mut decompressed)
        .await?;

    Ok((serde_json::from_slice(&decompressed)?, modified))
}

/// Compress and write the gateway list to `path`, streaming the compressed bytes straight to the
/// temp file instead of buffering them in memory first.
async fn write_to_path(path: &Path, raw: &[NymDirectoryGateway]) -> Result<()> {
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

    let tmp = path.with_extension("tmp");
    let tmp_file = fs::File::create(&tmp)
        .await
        .map_err(|err| store_err(format!("failed to create temp file: {err}")))?;

    let mut encoder = GzipEncoder::new(tmp_file);
    encoder
        .write_all(&json)
        .await
        .map_err(|err| store_err(format!("failed to compress: {err}")))?;
    encoder
        .shutdown()
        .await
        .map_err(|err| store_err(format!("failed to compress: {err}")))?;

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
        results: &[(GatewayType, Result<Option<SeededGateways>>)],
        gw_type: GatewayType,
    ) -> &Result<Option<SeededGateways>> {
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
            let seeded = find(&results, gw_type)
                .as_ref()
                .unwrap()
                .clone()
                .expect("expected builtin-seeded gateways");
            assert!(
                matches!(seeded, SeededGateways::FromBuiltin(_)),
                "expected {gw_type:?} to be seeded from builtin, not an on-disk cache"
            );
            let gateways = seeded.into_gateways();
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
        write_to_path(&path, &[]).await.unwrap();
        assert!(path.exists());

        let results = seed_all(dir.path(), true).await;
        let gateways = find(&results, gw_type)
            .as_ref()
            .unwrap()
            .clone()
            .expect("expected the empty disk file to be ignored in favor of the builtin fallback")
            .into_gateways();
        assert!(!gateways.is_empty());

        // Reset to the empty file and check the not-allowed case falls through to `None` rather
        // than an empty `Some`.
        write_to_path(&path, &[]).await.unwrap();
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
        let seeded = find(&results, gw_type)
            .as_ref()
            .unwrap()
            .clone()
            .expect("expected the on-disk cache to be used");
        assert!(
            matches!(seeded, SeededGateways::FromDisk { .. }),
            "expected the persisted cache to be reported as FromDisk"
        );
        let loaded = seeded.into_gateways();
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

    #[tokio::test]
    async fn reports_the_real_age_of_an_on_disk_cache() {
        let dir = tempdir().unwrap();
        let gw_type = GatewayType::MixnetExit;

        let snapshot = builtin::load_builtin_snapshot().await.unwrap();
        let raw = snapshot.raw_gateways(gw_type);
        save(dir.path(), gw_type, &raw).await.unwrap();

        // A cache written moments ago should be reported as fresh (age near zero), not
        // unconditionally treated as maximally stale like a builtin-seeded entry would be -
        // otherwise every restart would trigger a needless refetch.
        let results = seed_all(dir.path(), false).await;
        let seeded = find(&results, gw_type)
            .as_ref()
            .unwrap()
            .clone()
            .expect("expected the on-disk cache to be used");
        match seeded {
            SeededGateways::FromDisk { age, .. } => {
                assert!(
                    age < Duration::from_secs(10),
                    "unexpectedly large age: {age:?}"
                );
            }
            SeededGateways::FromBuiltin(_) => panic!("expected FromDisk, got FromBuiltin"),
        }
    }
}
