// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AirportingError, Result};
use async_compression::tokio::{bufread::GzipDecoder, write::GzipEncoder};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

static BASE_URL: Option<&'static str> = None;

static SOURCES: &[Source] = &[Source {
    country_code: "CN",
    file_name: "airporting_cn.txt.gz",
    builtin: include_bytes!("builtin/airporting_cn.txt.gz"),
    meta_file_name: "airporting_cn.txt.meta",
    meta_builtin: include_bytes!("builtin/airporting_cn.txt.meta"),
}];

/// Load the airporting IP networks from the data files.
pub(crate) async fn load_ip_networks(
    data_dir: &Path,
    country_codes: &[&str],
) -> Result<Vec<IpNetwork>> {
    let airporting_path = get_airporting_path(data_dir);
    let mut ip_networks = Vec::new();

    let load_from_file =
        async |meta_path: &Path, data_path: &Path| -> Result<(SourceMetaData, String)> {
            let meta_data = SourceMetaData::from_file(&meta_path).await?;
            let ip_networks_txt = Source::load_data_file(&data_path, &meta_data).await?;
            Ok((meta_data, ip_networks_txt))
        };

    for source in SOURCES.iter() {
        if !country_codes.contains(&source.country_code) {
            continue;
        }

        // Attempt to load the meta data and IP networks data from disk, but if that
        // fails then load the built-in data instead.
        let meta_path = airporting_path.join(source.meta_file_name);
        let data_path = airporting_path.join(source.file_name);
        let (meta_data, ip_networks_txt) =
            match load_from_file(meta_path.as_path(), data_path.as_path()).await {
                Ok((meta_data, ip_networks_txt)) => (meta_data, ip_networks_txt),
                Err(error) => {
                    tracing::debug!(
                        "Failed to load airporting files from disk for country: {}: {error}",
                        source.country_code
                    );

                    // The files are obviously broken, so remove them (they might not even exist)
                    let _ = fs::remove_file(&meta_path).await;
                    let _ = fs::remove_file(&data_path).await;

                    let meta_data = SourceMetaData::from_slice(source.meta_builtin).await?;
                    let ip_networks_txt = Source::load_builtin(source.builtin, &meta_data).await?;
                    (meta_data, ip_networks_txt)
                }
            };

        ip_networks.reserve(meta_data.line_count);

        for ip_network_txt in ip_networks_txt.lines() {
            let ip_network = ip_network_txt.parse::<IpNetwork>().map_err(|error| {
                AirportingError::ParseIpNetwork {
                    ip_network: ip_network_txt.to_string(),
                    file_path: data_path.clone(),
                    error,
                }
            })?;
            ip_networks.push(ip_network);
        }
    }

    Ok(ip_networks)
}

/// Update all the airporting files for the given countries.
/// Returns `Ok(true)` if any file was updated, `Ok(false)` if everything was already up-to-date.
pub(crate) async fn update_files(
    data_dir: &Path,
    country_codes: &[&str],
    user_agent: &str,
) -> Result<bool> {
    let airporting_path = get_airporting_path(data_dir);
    let http_client = reqwest::Client::new();

    // Ensure the airporting directory exists
    fs::create_dir_all(&airporting_path)
        .await
        .map_err(|error| AirportingError::CreateDirectory {
            dir: airporting_path.clone(),
            error,
        })?;

    let mut any_updated = false;

    for source in SOURCES.iter() {
        if !country_codes.contains(&source.country_code) {
            continue;
        }

        match source
            .update_data_file(&airporting_path, &http_client, user_agent)
            .await
        {
            Ok(true) => {
                tracing::debug!(
                    "Updated airporting data files for country: {}",
                    source.country_code
                );
                any_updated = true;
            }
            Ok(false) => {
                tracing::debug!(
                    "Airporting data files are already up-to-date for country: {}",
                    source.country_code
                );
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to update airporting data files for country {}: {error}",
                    source.country_code
                );
            }
        }
    }

    Ok(any_updated)
}

pub(crate) fn get_airporting_path(data_dir: &Path) -> PathBuf {
    PathBuf::from(data_dir).join("airporting")
}

pub(crate) struct Source {
    pub country_code: &'static str,
    pub file_name: &'static str,
    pub builtin: &'static [u8],
    pub meta_file_name: &'static str,
    pub meta_builtin: &'static [u8],
}

impl Source {
    const TEMP_DATA_FILE_NAME: &'static str = "temp_data";
    const TEMP_META_FILE_NAME: &'static str = "temp_meta";

    /// Update the data file on disk from the source website.
    /// Returns: Ok(true) if the file was updated.
    async fn update_data_file(
        &self,
        airporting_path: &Path,
        http_client: &reqwest::Client,
        user_agent: &str,
    ) -> Result<bool> {
        if let Err(error) = Self::cleanup_temp_files(airporting_path).await {
            tracing::warn!("Failed to clean up temporary airporting files: {error}; Ignoring.");
        }

        let Some(base_url) = BASE_URL else {
            return Ok(false); // Updating is disabled.
        };

        // Download the meta file and check if the file has been updated
        let meta_url = format!("{base_url}/{}", self.meta_file_name);
        let request = http_client
            .get(&meta_url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .header(reqwest::header::ACCEPT, "text/plain; charset=utf-8,*/*")
            .header(reqwest::header::ACCEPT_CHARSET, "utf-8");
        let response = request
            .send()
            .await
            .map_err(|error| AirportingError::FetchData {
                url: meta_url.to_string(),
                error,
            })?;

        // Read the rest of the response
        let data_bytes = response
            .bytes()
            .await
            .map_err(|error| AirportingError::FetchData {
                url: meta_url.to_string(),
                error,
            })?;

        let meta_data_path = airporting_path.join(self.meta_file_name);
        let meta_data = SourceMetaData::from_slice(data_bytes.as_ref()).await?;

        // Read the current meta data file from disk, and if that fails use the built-in meta data file.
        let current_meta_data = match SourceMetaData::from_file(&meta_data_path).await {
            Ok(meta) => Some(meta),
            Err(_) => SourceMetaData::from_slice(self.meta_builtin).await.ok(),
        };

        // Are we arleady up-to-date?
        if let Some(current_meta_data) = current_meta_data
            && current_meta_data.updated_utc >= meta_data.updated_utc
        {
            return Ok(false);
        }

        // Yes, let's use the updated data file from the website.
        let data_url = format!("{base_url}/{}", self.file_name);
        let request = http_client
            .get(&data_url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .header(reqwest::header::ACCEPT, "text/plain; charset=utf-8,*/*")
            .header(reqwest::header::ACCEPT_CHARSET, "utf-8")
            .header(reqwest::header::ACCEPT_ENCODING, "gzip");
        let response = request
            .send()
            .await
            .map_err(|error| AirportingError::FetchData {
                url: data_url.to_string(),
                error,
            })?;

        // Read the rest of the response
        let data_bytes = response
            .bytes()
            .await
            .map_err(|error| AirportingError::FetchData {
                url: data_url.to_string(),
                error,
            })?;

        // Write the data to a temporary file in the airporting directory.
        // 1. We regenerate the meta data when we save it.
        // 2. We re-compress the data file when we save it.  This might be wasteful,
        //    but it does ensure the data is valid.
        let temp_data_path = airporting_path.join(Self::TEMP_DATA_FILE_NAME);
        let temp_meta_data = Self::save_data_file(&temp_data_path, &data_bytes).await?;

        // Write the new meta data to a temporary file in the airporting directory
        let temp_meta_path = airporting_path.join(Self::TEMP_META_FILE_NAME);
        temp_meta_data.write_to_file(&temp_meta_path).await?;

        // Now all the data is on-disk, switch the old files with the new ones by renaming them.
        let data_path = airporting_path.join(self.file_name);
        fs::rename(&temp_data_path, &data_path)
            .await
            .map_err(|error| AirportingError::RenameFile {
                from: temp_data_path.clone(),
                to: data_path.clone(),
                error,
            })?;

        let meta_path = airporting_path.join(self.meta_file_name);
        fs::rename(&temp_meta_path, &meta_path)
            .await
            .map_err(|error| AirportingError::RenameFile {
                from: temp_meta_path.clone(),
                to: meta_path.clone(),
                error,
            })?;

        tracing::debug!("Updated airporting data file {}", self.file_name);

        Ok(true)
    }

    /// Cleanup any extraneous temporary files that may be left over from a failed update.
    async fn cleanup_temp_files(airporting_path: &Path) -> Result<()> {
        let temp_data_path = airporting_path.join(Self::TEMP_DATA_FILE_NAME);
        if temp_data_path.exists() {
            fs::remove_file(&temp_data_path).await.map_err(|error| {
                AirportingError::RemoveFile {
                    file_path: temp_data_path.clone(),
                    error,
                }
            })?;
        }

        let temp_meta_path = airporting_path.join(Self::TEMP_META_FILE_NAME);
        if temp_meta_path.exists() {
            fs::remove_file(&temp_meta_path).await.map_err(|error| {
                AirportingError::RemoveFile {
                    file_path: temp_meta_path.clone(),
                    error,
                }
            })?;
        }

        Ok(())
    }

    /// Load the data file from disk, gunzip it, and check the uncompressed length and SHA256.
    async fn load_data_file(file_path: &Path, meta_data: &SourceMetaData) -> Result<String> {
        let file = fs::File::open(file_path)
            .await
            .map_err(|error| AirportingError::ReadFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        let reader = BufReader::new(file);
        Self::load(reader, meta_data, file_path).await
    }

    /// Load the data file from slice containing compressed data
    async fn load_builtin(bytes: &[u8], meta_data: &SourceMetaData) -> Result<String> {
        let reader = Cursor::new(bytes);
        Self::load(reader, meta_data, Path::new("built-in")).await
    }

    async fn load<R>(reader: R, meta_data: &SourceMetaData, file_path: &Path) -> Result<String>
    where
        R: tokio::io::AsyncBufRead + Unpin,
    {
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
                    .map_err(|error| AirportingError::DecompressData {
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
            return Err(AirportingError::InvalidDataFileLength {
                file_path: file_path.to_path_buf(),
                expected: meta_data.length,
                actual: total_len,
            });
        }

        let sha256 = hex::encode(hasher.finalize());
        if sha256 != meta_data.sha256 {
            return Err(AirportingError::InvalidDataFileHash {
                file_path: file_path.to_path_buf(),
                expected: meta_data.sha256.clone(),
                actual: sha256,
            });
        }

        let domain_list = String::from_utf8(decompressed).map_err(|error| {
            AirportingError::InvalidDataFileEncoding {
                file_path: file_path.to_path_buf(),
                error,
            }
        })?;

        Ok(domain_list)
    }

    /// Compress the data file contents and save it to disk.
    /// We regenerate and return the new meta data.
    async fn save_data_file(file_path: &Path, data: &[u8]) -> Result<SourceMetaData> {
        let length = data.len();
        let line_count = data.iter().filter(|&&byte| byte == b'\n').count() + 1;
        let sha256 = hex::encode(Sha256::digest(data));

        let mut encoder = GzipEncoder::new(Vec::new());
        encoder
            .write_all(data)
            .await
            .map_err(|error| AirportingError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        let compressed_data = encoder
            .shutdown()
            .await
            .map_err(|error| AirportingError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })
            .map(|_| encoder.into_inner())?;

        fs::write(file_path, &compressed_data)
            .await
            .map_err(|error| AirportingError::WriteFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        Ok(SourceMetaData {
            length,
            line_count,
            sha256,
            updated_utc: OffsetDateTime::now_utc(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SourceMetaData {
    pub length: usize,     // Length of uncompressed data
    pub line_count: usize, // Numbers of lines in uncompressed data
    pub sha256: String,    // Hash of uncompressed data
    #[serde(with = "time::serde::iso8601")]
    pub updated_utc: OffsetDateTime,
}

impl SourceMetaData {
    pub(crate) async fn from_file(file_path: &Path) -> Result<Self> {
        let meta_content =
            fs::read_to_string(&file_path)
                .await
                .map_err(|error| AirportingError::ReadFile {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;

        let meta_data: Self = serde_json::from_str(&meta_content).map_err(|error| {
            AirportingError::DeserializeMetaFile {
                file_path: file_path.to_path_buf(),
                error,
            }
        })?;

        Ok(meta_data)
    }

    pub(crate) async fn from_slice(bytes: &[u8]) -> Result<Self> {
        let meta_data: Self = serde_json::from_slice(&bytes)
            .map_err(|error| AirportingError::DeserializeMetaData { error })?;

        Ok(meta_data)
    }

    pub(crate) async fn write_to_file(&self, file_path: &Path) -> Result<()> {
        let meta_content = serde_json::to_string_pretty(self)
            .map_err(|error| AirportingError::SerializeMetaFile { error })?;

        fs::write(&file_path, &meta_content)
            .await
            .map_err(|error| AirportingError::WriteFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        Ok(())
    }
}
