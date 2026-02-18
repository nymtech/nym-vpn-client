// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::resolver::ad_block::{AdBlockingError, Result};
use adblock::{
    FilterSet,
    lists::{FilterFormat, ParseOptions, RuleTypes},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
use time::{Duration, OffsetDateTime};
use tokio::fs;
use xz2::{read::XzDecoder, write::XzEncoder};

pub(crate) static SOURCES: &[Source] = &[
    Source {
        file_name: "easylist_adservers.txt.xz",
        builtin: include_bytes!("builtin/easylist_adservers.txt.xz"),
        url: "https://raw.githubusercontent.com/easylist/easylist/refs/heads/master/easylist/easylist_adservers.txt",
        meta_file_name: "easylist_adservers.txt.meta",
        meta_builtin: include_str!("builtin/easylist_adservers.txt.meta"),
        filterset_format: FilterFormat::Standard,
    },
    Source {
        file_name: "light.txt.xz",
        builtin: include_bytes!("builtin/light.txt.xz"),
        url: "https://cdn.jsdelivr.net/gh/hagezi/dns-blocklists@latest/hosts/light.txt",
        meta_file_name: "light.txt.meta",
        meta_builtin: include_str!("builtin/light.txt.meta"),
        filterset_format: FilterFormat::Hosts,
    },
];

static USER_AGENT: &str = "nym-vpn-ad-blocker/1.0";

/// Initialize the ad-blocking domain lists using the ones built-into the binary.
pub async fn init_files(data_dir: &Path, force: bool) -> Result<()> {
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    fs::create_dir_all(&ad_blocking_path)
        .await
        .map_err(|error| AdBlockingError::CreateDirectory {
            dir: ad_blocking_path.clone(),
            error,
        })?;

    for source in SOURCES.iter() {
        source.init(&ad_blocking_path, force).await?;
    }

    Ok(())
}

/// Update the ad-blocking domain lists by downloading the latest versions from their respective URLs.
/// The data files are considered out-of-date if they were last updated longer than `expired_duration` ago.
pub async fn update_files(data_dir: &Path, expired_duration: Duration) -> Result<bool> {
    let ad_blocking_path = get_ad_blocking_path(data_dir);
    let mut updated = false;
    let http_client = reqwest::Client::new();

    for source in SOURCES.iter() {
        if source
            .update_data_file(&ad_blocking_path, &http_client, expired_duration)
            .await?
        {
            updated = true;
        }
    }

    Ok(updated)
}

/// Create an `adblock::FilterSet` from the data files on disk.
pub async fn load_filter_set(data_dir: &Path) -> Result<FilterSet> {
    let ad_blocking_path = get_ad_blocking_path(data_dir);
    let mut filter_set = FilterSet::new(cfg!(debug_assertions));

    for source in SOURCES.iter() {
        let meta_path = ad_blocking_path.join(source.meta_file_name);
        let meta_data = SourceMetaData::from_file(&meta_path).await?;
        let data_path = ad_blocking_path.join(source.file_name);
        let domain_list = Source::load_data_file(&data_path, &meta_data).await?;
        filter_set.add_filter_list(
            &domain_list,
            ParseOptions {
                format: source.filterset_format,
                rule_types: RuleTypes::NetworkOnly,
                ..Default::default()
            },
        );
    }

    Ok(filter_set)
}

pub(crate) fn get_ad_blocking_path(data_dir: &Path) -> PathBuf {
    PathBuf::from(data_dir).join("ad-blocking")
}

pub(crate) struct Source {
    pub(crate) file_name: &'static str,
    pub(crate) builtin: &'static [u8],
    pub(crate) url: &'static str,
    pub(crate) meta_file_name: &'static str,
    pub(crate) meta_builtin: &'static str,
    pub(crate) filterset_format: FilterFormat,
}

impl Source {
    async fn init(&self, ad_blocking_path: &Path, force: bool) -> Result<()> {
        let data_path = ad_blocking_path.join(self.file_name);
        if force || !data_path.exists() {
            fs::write(&data_path, self.builtin).await.map_err(|error| {
                AdBlockingError::WriteFile {
                    file_path: data_path.clone(),
                    error,
                }
            })?;
            tracing::debug!("Initialized ad-blocking data file {}", data_path.display());
        }

        let meta_path = ad_blocking_path.join(self.meta_file_name);
        if force || !meta_path.exists() {
            fs::write(&meta_path, self.meta_builtin)
                .await
                .map_err(|error| AdBlockingError::WriteFile {
                    file_path: meta_path.clone(),
                    error,
                })?;
            tracing::debug!("Initialized ad-blocking meta file {}", meta_path.display());
        }

        Ok(())
    }

    /// Update the data file on disk from the source website.
    /// Returns: Ok(true) if the file was updated.
    async fn update_data_file(
        &self,
        ad_blocking_path: &Path,
        http_client: &reqwest::Client,
        expired_duration: Duration,
    ) -> Result<bool> {
        // Read the current meta file
        let meta_path = ad_blocking_path.join(self.meta_file_name);
        let meta_data = SourceMetaData::from_file(&meta_path).await?;

        let now = OffsetDateTime::now_utc();
        if now - meta_data.updated_from_website_utc < expired_duration {
            tracing::trace!(
                "Ad-blocking data file {} is up-to-date (last updated {} and expired duration is {}).",
                self.file_name,
                meta_data.updated_from_website_utc,
                expired_duration
            );
            return Ok(false);
        }

        // Request a new version of the data file, as long as it's different to the current one
        // Note: Accept-Encoding: gzip is required to get the etag back in the right format.
        let request = http_client
            .get(self.url)
            .header(reqwest::header::IF_NONE_MATCH, &meta_data.etag)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header(reqwest::header::ACCEPT, "text/plain; charset=utf-8,*/*")
            .header(reqwest::header::ACCEPT_CHARSET, "utf-8")
            .header(reqwest::header::ACCEPT_ENCODING, "gzip");
        let response = request
            .send()
            .await
            .map_err(|error| AdBlockingError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            tracing::debug!("Ad-blocking data file {} is up to date", self.file_name);
            return Ok(false);
        }

        // Grab the new etag from the HTTP response
        let etag = Self::get_response_header(self.url, &response, reqwest::header::ETAG)?;

        if etag == meta_data.etag {
            tracing::warn!(
                "Ad-blocking data file {} is up to date (etag matches). However server didn't return 'NOT_MODIFIED'!",
                self.file_name
            );
            return Ok(false);
        }

        tracing::trace!(
            "Updating ad-blocking data file {}. Etag: old='{}', new='{}'",
            self.file_name,
            meta_data.etag,
            etag
        );

        // Read the rest of the response
        let data_bytes = response
            .bytes()
            .await
            .map_err(|error| AdBlockingError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        // Write the data to a temporary file in the ad-blocking directory
        let temp_data_path = ad_blocking_path.join("temp_data");
        let temp_meta_data = Self::save_data_file(&temp_data_path, &data_bytes, &etag).await?;

        // Write the new meta data to a temporary file in the ad-blocking directory
        let temp_meta_path = ad_blocking_path.join("temp_meta");
        temp_meta_data.write_to_file(&temp_meta_path).await?;

        // Now all the data is on-disk, switch the old files with the new ones by renaming them.
        let data_path = ad_blocking_path.join(self.file_name);
        fs::rename(&temp_data_path, &data_path)
            .await
            .map_err(|error| AdBlockingError::RenameFile {
                from: temp_data_path.clone(),
                to: data_path.clone(),
                error,
            })?;

        let meta_path = ad_blocking_path.join(self.meta_file_name);
        fs::rename(&temp_meta_path, &meta_path)
            .await
            .map_err(|error| AdBlockingError::RenameFile {
                from: temp_meta_path.clone(),
                to: meta_path.clone(),
                error,
            })?;

        tracing::info!("Updated ad-blocking data file {}", self.file_name);

        Ok(true)
    }

    /// Load the data file from disk and uncompress it and check the file length and SHA256 are correct.
    async fn load_data_file(file_path: &Path, meta_data: &SourceMetaData) -> Result<String> {
        let data_bytes = fs::read(&file_path)
            .await
            .map_err(|error| AdBlockingError::ReadFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        let mut decoder = XzDecoder::new(&data_bytes[..]);
        let mut decompressed_bytes = Vec::new();
        decoder
            .read_to_end(&mut decompressed_bytes)
            .map_err(|error| AdBlockingError::DecompressData {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        if decompressed_bytes.len() != meta_data.bytes {
            return Err(AdBlockingError::InvalidDataFileLength {
                file_path: file_path.to_path_buf(),
                expected: meta_data.bytes,
                actual: decompressed_bytes.len(),
            });
        }

        let actual_sha256 = format!("{:x}", Sha256::digest(&decompressed_bytes));
        if actual_sha256 != meta_data.sha256 {
            return Err(AdBlockingError::InvalidDataFileHash {
                file_path: file_path.to_path_buf(),
                expected: meta_data.sha256.clone(),
                actual: actual_sha256,
            });
        }

        let domain_list = String::from_utf8(decompressed_bytes).map_err(|error| {
            AdBlockingError::InvalidDataFileEncoding {
                file_path: file_path.to_path_buf(),
                error,
            }
        })?;

        Ok(domain_list)
    }

    /// Save the data file to disk and update the meta data with the new file length and SHA256 hash.
    async fn save_data_file(file_path: &Path, data: &[u8], etag: &str) -> Result<SourceMetaData> {
        let byte_len = data.len();
        let sha256 = format!("{:x}", Sha256::digest(data));

        // Compress the data

        let mut encoder = XzEncoder::new(Vec::new(), 9);
        encoder
            .write_all(data)
            .map_err(|error| AdBlockingError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })?;
        let compressed_data = encoder
            .finish()
            .map_err(|error| AdBlockingError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        // Write the compressed data to file
        fs::write(&file_path, &compressed_data)
            .await
            .map_err(|error| AdBlockingError::WriteFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        // Return the new meta data
        Ok(SourceMetaData {
            bytes: byte_len,
            etag: etag.to_string(),
            sha256,
            updated_from_website_utc: OffsetDateTime::now_utc(),
        })
    }

    fn get_response_header(
        url: &str,
        response: &reqwest::Response,
        header: reqwest::header::HeaderName,
    ) -> Result<String> {
        let etag = response
            .headers()
            .get(&header)
            .ok_or(AdBlockingError::MissingHeader {
                header: header.clone(),
                url: url.to_string(),
            })?
            .to_str()
            .map_err(|error| AdBlockingError::InvalidHeader {
                header: header.clone(),
                url: url.to_string(),
                error,
            })?
            .to_string();
        Ok(etag)
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SourceMetaData {
    pub(crate) bytes: usize, // Size of uncompressed data
    pub(crate) etag: String,
    pub(crate) sha256: String, // Hash of uncompressed data
    #[serde(with = "time::serde::iso8601")]
    pub(crate) updated_from_website_utc: OffsetDateTime,
}

impl SourceMetaData {
    pub(crate) async fn from_file(file_path: &Path) -> Result<Self> {
        let meta_content =
            fs::read_to_string(&file_path)
                .await
                .map_err(|error| AdBlockingError::ReadFile {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;

        let meta_data: Self = serde_json::from_str(&meta_content).map_err(|error| {
            AdBlockingError::DeserializeMetaFile {
                file_path: file_path.to_path_buf(),
                error,
            }
        })?;

        Ok(meta_data)
    }

    pub(crate) async fn write_to_file(&self, file_path: &Path) -> Result<()> {
        let meta_content = serde_json::to_string_pretty(self)
            .map_err(|error| AdBlockingError::SerializeMetaFile { error })?;

        fs::write(&file_path, &meta_content)
            .await
            .map_err(|error| AdBlockingError::WriteFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        Ok(())
    }
}
