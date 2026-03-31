// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AdBlockerError, Result};
use adblock::{
    lists::{FilterFormat, ParseOptions, RuleTypes},
    FilterSet,
};
use async_compression::tokio::{bufread::GzipDecoder, write::GzipEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

pub(crate) static SOURCES: &[Source] = &[
    Source {
        file_name: "easylist_adservers.txt.gz",
        builtin: include_bytes!("builtin/easylist_adservers.txt.gz"),
        url: "https://raw.githubusercontent.com/easylist/easylist/refs/heads/master/easylist/easylist_adservers.txt",
        meta_file_name: "easylist_adservers.txt.meta",
        meta_builtin: include_str!("builtin/easylist_adservers.txt.meta"),
        filterset_format: FilterFormat::Standard,
    },
    Source {
        file_name: "light.txt.gz",
        builtin: include_bytes!("builtin/light.txt.gz"),
        url: "https://cdn.jsdelivr.net/gh/hagezi/dns-blocklists@latest/hosts/light.txt",
        meta_file_name: "light.txt.meta",
        meta_builtin: include_str!("builtin/light.txt.meta"),
        filterset_format: FilterFormat::Hosts,
    },
];

/// Initialize the ad-blocker domain lists using the ones built-into the binary
/// and load them into a filter set.
pub(crate) async fn init_and_load_filter_set(
    data_dir: PathBuf,
    force: bool,
) -> Result<Box<FilterSet>> {
    init_files(&data_dir, force).await?;
    load_filter_set(&data_dir).await
}

/// Update the ad-blocker domain lists by downloading the latest versions,
/// and load thedm into a filter set.  If they were not updated when return `Ok(None)`.
pub(crate) async fn update_and_load_filter_set(
    data_dir: PathBuf,
    user_agent: String,
) -> Result<Option<Box<FilterSet>>> {
    let updated = update_files(&data_dir, &user_agent).await?;
    if updated {
        let filter_set = load_filter_set(&data_dir).await?;
        Ok(Some(filter_set))
    } else {
        Ok(None)
    }
}

/// Initialize the ad-blocker domain lists using the ones built-into the binary.
pub(crate) async fn init_files(data_dir: &Path, force: bool) -> Result<()> {
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    fs::create_dir_all(&ad_blocking_path)
        .await
        .map_err(|error| AdBlockerError::CreateDirectory {
            dir: ad_blocking_path.clone(),
            error,
        })?;

    for source in SOURCES.iter() {
        source.init(&ad_blocking_path, force).await?;
    }

    Ok(())
}

/// Update the ad-blocker domain lists by downloading the latest versions
pub(crate) async fn update_files(data_dir: &Path, user_agent: &str) -> Result<bool> {
    let ad_blocking_path = get_ad_blocking_path(data_dir);
    let mut updated = false;
    let http_client = reqwest::Client::new();

    for source in SOURCES.iter() {
        if source
            .update_data_file(&ad_blocking_path, &http_client, user_agent)
            .await?
        {
            updated = true;
        }
    }

    Ok(updated)
}

/// Create an `adblock::FilterSet` from the data files on disk.
pub(crate) async fn load_filter_set(data_dir: &Path) -> Result<Box<FilterSet>> {
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

    Ok(Box::new(filter_set))
}

pub(crate) fn get_ad_blocking_path(data_dir: &Path) -> PathBuf {
    PathBuf::from(data_dir).join("ad-blocking")
}

pub(crate) struct Source {
    pub file_name: &'static str,
    pub builtin: &'static [u8],
    pub url: &'static str,
    pub meta_file_name: &'static str,
    pub meta_builtin: &'static str,
    pub filterset_format: FilterFormat,
}

impl Source {
    const TEMP_DATA_FILE_NAME: &'static str = "temp_data";
    const TEMP_META_FILE_NAME: &'static str = "temp_meta";

    async fn init(&self, ad_blocking_path: &Path, force: bool) -> Result<()> {
        let data_path = ad_blocking_path.join(self.file_name);
        if force || !data_path.exists() {
            fs::write(&data_path, self.builtin).await.map_err(|error| {
                AdBlockerError::WriteFile {
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
                .map_err(|error| AdBlockerError::WriteFile {
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
        user_agent: &str,
    ) -> Result<bool> {
        if let Err(error) = Self::cleanup_temp_files(ad_blocking_path).await {
            tracing::warn!("Failed to clean up temporary ad-blocker files: {error}; Ignoring.");
        }

        // Read the current meta file
        let meta_path = ad_blocking_path.join(self.meta_file_name);
        let meta_data = SourceMetaData::from_file(&meta_path).await?;

        // Request a new version of the data file, as long as it's different to the current one
        // Note: Accept-Encoding: gzip is required to get the etag back in the right format.
        let request = http_client
            .get(self.url)
            .header(reqwest::header::IF_NONE_MATCH, &meta_data.etag)
            .header(reqwest::header::USER_AGENT, user_agent)
            .header(reqwest::header::ACCEPT, "text/plain; charset=utf-8,*/*")
            .header(reqwest::header::ACCEPT_CHARSET, "utf-8")
            .header(reqwest::header::ACCEPT_ENCODING, "gzip");
        let response = request
            .send()
            .await
            .map_err(|error| AdBlockerError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            tracing::debug!("Ad-blocker data file {} is up to date", self.file_name);
            return Ok(false);
        }

        // Grab the new etag from the HTTP response
        let etag = Self::get_response_header(self.url, &response, reqwest::header::ETAG)?;

        if etag == meta_data.etag {
            tracing::warn!(
                "Ad-blocker data file {} is up to date (etag matches). However server didn't return 'NOT_MODIFIED'!",
                self.file_name
            );
            return Ok(false);
        }

        tracing::trace!(
            "Updating ad-blocker data file {}. Etag: old='{}', new='{}'",
            self.file_name,
            meta_data.etag,
            etag
        );

        // Read the rest of the response
        let data_bytes = response
            .bytes()
            .await
            .map_err(|error| AdBlockerError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        // Write the data to a temporary file in the ad-blocker directory
        let temp_data_path = ad_blocking_path.join(Self::TEMP_DATA_FILE_NAME);
        let temp_meta_data = Self::save_data_file(&temp_data_path, &data_bytes, &etag).await?;

        // Write the new meta data to a temporary file in the ad-blocker directory
        let temp_meta_path = ad_blocking_path.join(Self::TEMP_META_FILE_NAME);
        temp_meta_data.write_to_file(&temp_meta_path).await?;

        // Now all the data is on-disk, switch the old files with the new ones by renaming them.
        let data_path = ad_blocking_path.join(self.file_name);
        fs::rename(&temp_data_path, &data_path)
            .await
            .map_err(|error| AdBlockerError::RenameFile {
                from: temp_data_path.clone(),
                to: data_path.clone(),
                error,
            })?;

        let meta_path = ad_blocking_path.join(self.meta_file_name);
        fs::rename(&temp_meta_path, &meta_path)
            .await
            .map_err(|error| AdBlockerError::RenameFile {
                from: temp_meta_path.clone(),
                to: meta_path.clone(),
                error,
            })?;

        tracing::debug!("Updated ad-blocker data file {}", self.file_name);

        Ok(true)
    }

    /// Cleanup any extraneous temporary files that may be left over from a failed update.
    async fn cleanup_temp_files(ad_blocking_path: &Path) -> Result<()> {
        let temp_data_path = ad_blocking_path.join(Self::TEMP_DATA_FILE_NAME);
        if temp_data_path.exists() {
            fs::remove_file(&temp_data_path)
                .await
                .map_err(|error| AdBlockerError::RemoveFile {
                    file_path: temp_data_path.clone(),
                    error,
                })?;
        }

        let temp_meta_path = ad_blocking_path.join(Self::TEMP_META_FILE_NAME);
        if temp_meta_path.exists() {
            fs::remove_file(&temp_meta_path)
                .await
                .map_err(|error| AdBlockerError::RemoveFile {
                    file_path: temp_meta_path.clone(),
                    error,
                })?;
        }

        Ok(())
    }

    /// Load the data file from disk, gunzip it, and check the uncompressed length and SHA256.
    async fn load_data_file(file_path: &Path, meta_data: &SourceMetaData) -> Result<String> {
        let file = fs::File::open(file_path)
            .await
            .map_err(|error| AdBlockerError::ReadFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        let reader = BufReader::new(file);
        let mut decoder = GzipDecoder::new(reader);
        let mut hasher = Sha256::new();
        let mut decompressed = Vec::with_capacity(meta_data.length);
        let mut total_len: usize = 0;

        let mut buf = vec![0; 4096];
        loop {
            let n =
                decoder
                    .read(&mut buf)
                    .await
                    .map_err(|error| AdBlockerError::DecompressData {
                        file_path: file_path.to_path_buf(),
                        error,
                    })?;

            if n == 0 {
                break;
            }

            hasher.update(&buf[..n]);
            decompressed.extend_from_slice(&buf[..n]);
            total_len = total_len.saturating_add(n);
        }

        if total_len != meta_data.length {
            return Err(AdBlockerError::InvalidDataFileLength {
                file_path: file_path.to_path_buf(),
                expected: meta_data.length,
                actual: total_len,
            });
        }

        let sha256 = hex::encode(hasher.finalize());
        if sha256 != meta_data.sha256 {
            return Err(AdBlockerError::InvalidDataFileHash {
                file_path: file_path.to_path_buf(),
                expected: meta_data.sha256.clone(),
                actual: sha256,
            });
        }

        let domain_list = String::from_utf8(decompressed).map_err(|error| {
            AdBlockerError::InvalidDataFileEncoding {
                file_path: file_path.to_path_buf(),
                error,
            }
        })?;

        Ok(domain_list)
    }

    /// Compress the data file contents and save it to disk.
    /// We regenerate and return the new meta data.
    async fn save_data_file(file_path: &Path, data: &[u8], etag: &str) -> Result<SourceMetaData> {
        let byte_len = data.len();
        let sha256 = hex::encode(Sha256::digest(data));

        let mut encoder = GzipEncoder::new(Vec::new());
        encoder
            .write_all(data)
            .await
            .map_err(|error| AdBlockerError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        let compressed_data = encoder
            .shutdown()
            .await
            .map_err(|error| AdBlockerError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })
            .map(|_| encoder.into_inner())?;

        fs::write(file_path, &compressed_data)
            .await
            .map_err(|error| AdBlockerError::WriteFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        Ok(SourceMetaData {
            length: byte_len,
            etag: etag.to_string(),
            sha256,
            updated_utc: OffsetDateTime::now_utc(),
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
            .ok_or(AdBlockerError::MissingHeader {
                header: header.clone(),
                url: url.to_string(),
            })?
            .to_str()
            .map_err(|error| AdBlockerError::InvalidHeader {
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
    pub etag: String,
    pub length: usize,  // Length of uncompressed data
    pub sha256: String, // Hash of uncompressed data
    #[serde(with = "time::serde::iso8601")]
    pub updated_utc: OffsetDateTime,
}

impl SourceMetaData {
    pub(crate) async fn from_file(file_path: &Path) -> Result<Self> {
        let meta_content =
            fs::read_to_string(&file_path)
                .await
                .map_err(|error| AdBlockerError::ReadFile {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;

        let meta_data: Self = serde_json::from_str(&meta_content).map_err(|error| {
            AdBlockerError::DeserializeMetaFile {
                file_path: file_path.to_path_buf(),
                error,
            }
        })?;

        Ok(meta_data)
    }

    pub(crate) async fn write_to_file(&self, file_path: &Path) -> Result<()> {
        let meta_content = serde_json::to_string_pretty(self)
            .map_err(|error| AdBlockerError::SerializeMetaFile { error })?;

        fs::write(&file_path, &meta_content)
            .await
            .map_err(|error| AdBlockerError::WriteFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        Ok(())
    }
}
