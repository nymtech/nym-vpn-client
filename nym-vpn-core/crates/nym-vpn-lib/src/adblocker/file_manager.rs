// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use adblock::lists::FilterFormat;
use async_compression::tokio::bufread::GzipDecoder;
use async_stream::try_stream;
use futures::{
    AsyncBufReadExt, FutureExt, StreamExt, TryStreamExt, future::BoxFuture, stream::BoxStream,
};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, BufReader},
};

use super::{AdBlockerError, Result};

pub(crate) static SOURCES: &[Source] = &[
    Source {
        file_name: "easylist_adservers.txt.gz",
        builtin: include_bytes!("builtin/easylist_adservers.txt.gz"),
        builtin_etag: include_str!("builtin/easylist_adservers.txt.etag"),
        url: concat!(
            "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com",
            "/easylist_adservers.txt.gz"
        ),
        filterset_format: FilterFormat::Standard,
    },
    Source {
        file_name: "light.txt.gz",
        builtin: include_bytes!("builtin/light.txt.gz"),
        builtin_etag: include_str!("builtin/light.txt.etag"),
        url: concat!(
            "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com",
            "/light.txt.gz"
        ),
        filterset_format: FilterFormat::Hosts,
    },
];

pub(crate) struct Source {
    pub file_name: &'static str,
    pub builtin: &'static [u8],
    pub builtin_etag: &'static str,
    pub url: &'static str,
    pub filterset_format: FilterFormat,
}

impl Source {
    /// Write the builtin compressed data file and its ETag sidecar to `cache_dir`
    /// if they do not already exist (or unconditionally when `force` is true).
    async fn init(&self, cache_dir: &Path, force: bool) -> Result<()> {
        let data_path = cache_dir.join(self.file_name);
        if force || !data_path.exists() {
            fs::write(&data_path, self.builtin).await.map_err(|error| {
                AdBlockerError::WriteFile {
                    file_path: data_path.clone(),
                    error,
                }
            })?;
            tracing::debug!("Initialized ad-blocking data file {}", data_path.display());
        }

        let etag_path = data_path.with_extension("etag");
        if force || !etag_path.exists() {
            fs::write(&etag_path, self.builtin_etag.trim())
                .await
                .map_err(|error| AdBlockerError::WriteFile {
                    file_path: etag_path.clone(),
                    error,
                })?;
            tracing::debug!("Initialized ad-blocking etag file {}", etag_path.display());
        }

        Ok(())
    }

    /// Load and decompress the data file from disk, returning its contents as a `String`.
    // Note: Pinned box is necessary in order to avoid "large future size" warnings.
    pub fn load_data_file(file_path: &Path) -> BoxFuture<'_, Result<String>> {
        async move {
            let file = File::open(file_path)
                .await
                .map_err(|error| AdBlockerError::ReadFile {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;

            let reader = BufReader::new(file);
            let mut decoder = GzipDecoder::new(reader);
            let mut decompressed = Vec::new();
            let mut buf = [0u8; 32 * 1024];

            loop {
                let n = decoder.read(&mut buf).await.map_err(|error| {
                    AdBlockerError::DecompressData {
                        file_path: file_path.to_path_buf(),
                        error,
                    }
                })?;
                if n == 0 {
                    break;
                }
                decompressed.extend_from_slice(&buf[..n]);
            }

            String::from_utf8(decompressed).map_err(|error| {
                AdBlockerError::InvalidDataFileEncoding {
                    file_path: file_path.to_path_buf(),
                    error,
                }
            })
        }
        .boxed()
    }

    /// Stream lines from a data file, decompressing it on the fly.
    pub fn stream_lines(file_path: &Path) -> BoxStream<'_, Result<String>> {
        let stream = Self::stream_chunks(file_path).map_err(std::io::Error::other);
        let async_read = stream.into_async_read();

        async_read
            .lines()
            .map_err(|err: std::io::Error| {
                err.downcast::<AdBlockerError>()
                    .unwrap_or_else(AdBlockerError::UnknownLineReadError)
            })
            .boxed()
    }

    /// Stream chunks from a data file, decompressing it on the fly.
    fn stream_chunks(file_path: &Path) -> BoxStream<'_, Result<Vec<u8>>> {
        try_stream! {
            let file = File::open(file_path)
                .await
                .map_err(|error| AdBlockerError::ReadFile {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;

            let reader = BufReader::new(file);
            let mut decoder = GzipDecoder::new(reader);

            let mut buf = [0u8; 32 * 1024];
            loop {
                let n = decoder.read(&mut buf).await.map_err(|error| {
                    AdBlockerError::DecompressData {
                        file_path: file_path.to_path_buf(),
                        error,
                    }
                })?;

                if n == 0 {
                    break;
                } else {
                    yield buf[..n].to_vec();
                }
            }
        }
        .boxed()
    }
}

/// Write the builtin data files to `cache_dir`.
///
/// Existing files are left untouched unless `force` is true.
pub(crate) async fn init_files(cache_dir: &Path, force: bool) -> Result<()> {
    fs::create_dir_all(cache_dir)
        .await
        .map_err(|error| AdBlockerError::CreateDirectory {
            dir: cache_dir.to_owned(),
            error,
        })?;

    for source in SOURCES.iter() {
        source.init(cache_dir, force).await?;
    }

    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    use tempfile::TempDir;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_init_files() {
        let temp_dir = init_tests()
            .await
            .expect("Failed to initialize ad-blocker files");
        let cache_dir = temp_dir.path();

        for source in SOURCES.iter() {
            let file_path = cache_dir.join(source.file_name);
            assert!(
                file_path.exists(),
                "ad-blocker data file {} was not created",
                file_path.display()
            );
        }
    }

    pub async fn init_tests() -> Result<TempDir, String> {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| format!("failed to create temporary directory: {e}"))?;
        let data_dir = temp_dir.path();

        init_files(data_dir, false)
            .await
            .map_err(|e| format!("failed to create initial ad-blocker files: {e}"))?;

        Ok(temp_dir)
    }
}
