// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use anyhow::Context;
use tokio::fs::{self, try_exists};

pub(crate) struct Source {
    pub file_name: &'static str,
    pub builtin: &'static [u8],
    pub builtin_etag: &'static str,
    pub url: &'static str,
}

pub(crate) static SOURCES: &[Source] = &[
    Source {
        file_name: "CN-ip.json.gz",
        builtin: include_bytes!("../builtin/CN-ip.json.gz"),
        builtin_etag: include_str!("../builtin/CN-ip.json.etag"),
        url: "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com/CN-ip.json.gz",
    },
    Source {
        file_name: "CN-domain.txt.gz",
        builtin: include_bytes!("../builtin/CN-domain.txt.gz"),
        builtin_etag: include_str!("../builtin/CN-domain.txt.etag"),
        url: "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com/CN-domain.txt.gz",
    },
    Source {
        file_name: "RU-ip.json.gz",
        builtin: include_bytes!("../builtin/RU-ip.json.gz"),
        builtin_etag: include_str!("../builtin/RU-ip.json.etag"),
        url: "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com/RU-ip.json.gz",
    },
    Source {
        file_name: "RU-domain.txt.gz",
        builtin: include_bytes!("../builtin/RU-domain.txt.gz"),
        builtin_etag: include_str!("../builtin/RU-domain.txt.etag"),
        url: "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com/RU-domain.txt.gz",
    },
];

/// Seed builtin files and their ETag sidecars into `data_dir` if not already present.
pub(crate) async fn init_files(data_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(data_dir)
        .await
        .with_context(|| format!("Failed to create directory '{}'", data_dir.display()))?;

    for source in SOURCES.iter() {
        let dest = data_dir.join(source.file_name);
        if !try_exists(&dest).await.unwrap_or(false) {
            fs::write(&dest, source.builtin)
                .await
                .with_context(|| format!("Failed to write builtin '{}'", dest.display()))?;
            tracing::debug!("Initialized '{}' from builtin", dest.display());
        }

        let etag_path = dest.with_extension("etag");
        if !try_exists(&etag_path).await.unwrap_or(false) {
            fs::write(&etag_path, source.builtin_etag.trim())
                .await
                .with_context(|| {
                    format!("Failed to write builtin etag '{}'", etag_path.display())
                })?;
            tracing::debug!("Initialized '{}' from builtin", etag_path.display());
        }
    }

    Ok(())
}
