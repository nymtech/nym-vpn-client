// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AirportingError, Result};
use async_compression::tokio::{bufread::GzipDecoder, write::GzipEncoder};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};
use time::OffsetDateTime;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

pub(crate) static SOURCES: &[Source] = &[Source {
    file_name: "airporting.txt.gz",
    builtin: include_bytes!("builtin/airporting.txt.gz"),
    url: "disabled", // Updating is currently disabled.
    meta_file_name: "airporting.txt.meta",
    meta_builtin: include_str!("builtin/airporting.txt.meta"),
}];

/// Initialize the airporting lists using the ones built-into the binary
/// and load them into a list of IP network addresses.
pub(crate) async fn init_and_load_ip_networks(
    data_dir: PathBuf,
    force: bool,
) -> Result<Vec<IpNetwork>> {
    init_files(&data_dir, force).await?;
    load_ip_networks(&data_dir).await
}

/// Update the airporting lists by downloading the latest versions,
/// and load them into a list of network addresses.  If they were not updated when return `Ok(None)`.
pub(crate) async fn update_and_load_ip_networks(
    data_dir: PathBuf,
    user_agent: String,
) -> Result<Option<Vec<IpNetwork>>> {
    let updated = update_files(&data_dir, &user_agent).await?;
    if updated {
        let ip_networks = load_ip_networks(&data_dir).await?;
        Ok(Some(ip_networks))
    } else {
        Ok(None)
    }
}

/// Initialize the airporting lists using the ones built-into the binary.
pub(crate) async fn init_files(data_dir: &Path, force: bool) -> Result<()> {
    let airporting_path = get_airporting_path(data_dir);

    fs::create_dir_all(&airporting_path)
        .await
        .map_err(|error| AirportingError::CreateDirectory {
            dir: airporting_path.clone(),
            error,
        })?;

    for source in SOURCES.iter() {
        source.init(&airporting_path, force).await?;
    }

    Ok(())
}

/// Update the airporting lists by downloading the latest versions
pub(crate) async fn update_files(data_dir: &Path, user_agent: &str) -> Result<bool> {
    let airporting_path = get_airporting_path(data_dir);
    let mut updated = false;
    let http_client = reqwest::Client::new();

    for source in SOURCES.iter() {
        if source
            .update_data_file(&airporting_path, &http_client, user_agent)
            .await?
        {
            updated = true;
        }
    }

    Ok(updated)
}

/// Load the airporting IP networks from all the files in the directory.
pub(crate) async fn load_ip_networks(data_dir: &Path) -> Result<Vec<IpNetwork>> {
    let airporting_path = get_airporting_path(data_dir);
    let mut ip_networks = Vec::new();

    for source in SOURCES.iter() {
        let meta_path = airporting_path.join(source.meta_file_name);
        let meta_data = SourceMetaData::from_file(&meta_path).await?;
        let data_path = airporting_path.join(source.file_name);
        let address_list_txt = Source::load_data_file(&data_path, &meta_data).await?;

        ip_networks.reserve(meta_data.line_count);

        for line in address_list_txt.lines() {
            let ip_network =
                line.parse::<IpNetwork>()
                    .map_err(|error| AirportingError::ParseIpNetwork {
                        file_path: data_path.clone(),
                        error,
                    })?;
            ip_networks.push(ip_network);
        }
    }

    Ok(ip_networks)
}

pub(crate) fn get_airporting_path(data_dir: &Path) -> PathBuf {
    PathBuf::from(data_dir).join("ad-blocking")
}

pub(crate) struct Source {
    pub file_name: &'static str,
    pub builtin: &'static [u8],
    pub url: &'static str,
    pub meta_file_name: &'static str,
    pub meta_builtin: &'static str,
}

impl Source {
    const TEMP_DATA_FILE_NAME: &'static str = "temp_data";
    const TEMP_META_FILE_NAME: &'static str = "temp_meta";

    async fn init(&self, airporting_path: &Path, force: bool) -> Result<()> {
        let data_path = airporting_path.join(self.file_name);
        if force || !data_path.exists() {
            fs::write(&data_path, self.builtin).await.map_err(|error| {
                AirportingError::WriteFile {
                    file_path: data_path.clone(),
                    error,
                }
            })?;
            tracing::debug!("Initialized airporting data file {}", data_path.display());
        }

        let meta_path = airporting_path.join(self.meta_file_name);
        if force || !meta_path.exists() {
            fs::write(&meta_path, self.meta_builtin)
                .await
                .map_err(|error| AirportingError::WriteFile {
                    file_path: meta_path.clone(),
                    error,
                })?;
            tracing::debug!("Initialized airporting meta file {}", meta_path.display());
        }

        Ok(())
    }

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

        // Is it disabled? (TODO: Remove this test)
        if self.url == "disabled" {
            tracing::debug!("Airporting file updating is currently disabled");
            return Ok(false);
        }

        // Request a new version of the data file, as long as it's different to the current one
        // Note: Accept-Encoding: gzip is required to get the etag back in the right format.
        let request = http_client
            .get(self.url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .header(reqwest::header::ACCEPT, "text/plain; charset=utf-8,*/*")
            .header(reqwest::header::ACCEPT_CHARSET, "utf-8")
            .header(reqwest::header::ACCEPT_ENCODING, "gzip");
        let response = request
            .send()
            .await
            .map_err(|error| AirportingError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        tracing::trace!("Updating airporting data file {}", self.file_name);

        // Read the rest of the response
        let data_bytes = response
            .bytes()
            .await
            .map_err(|error| AirportingError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        // Write the data to a temporary file in the ad-blocker directory
        let temp_data_path = airporting_path.join(Self::TEMP_DATA_FILE_NAME);
        let temp_meta_data = Self::save_data_file(&temp_data_path, &data_bytes).await?;

        // Write the new meta data to a temporary file in the ad-blocker directory
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

        tracing::debug!("Updated ad-blocker data file {}", self.file_name);

        Ok(true)
    }

    /// Clean-up any extraneous temporary files that may be left over from a failed update.
    async fn cleanup_temp_files(ad_blocking_path: &Path) -> Result<()> {
        let temp_data_path = ad_blocking_path.join(Self::TEMP_DATA_FILE_NAME);
        if temp_data_path.exists() {
            fs::remove_file(&temp_data_path).await.map_err(|error| {
                AirportingError::RemoveFile {
                    file_path: temp_data_path.clone(),
                    error,
                }
            })?;
        }

        let temp_meta_path = ad_blocking_path.join(Self::TEMP_META_FILE_NAME);
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
    // Note: Pinned box is necessary in order to avoid "large future size" warnings.
    fn load_data_file<'a>(
        file_path: &'a Path,
        meta_data: &'a SourceMetaData,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let file =
                fs::File::open(file_path)
                    .await
                    .map_err(|error| AirportingError::ReadFile {
                        file_path: file_path.to_path_buf(),
                        error,
                    })?;

            let reader = BufReader::new(file);
            let mut decoder = GzipDecoder::new(reader);
            let mut hasher = Sha256::new();
            let mut decompressed = Vec::with_capacity(meta_data.length);
            let mut total_len: usize = 0;

            let mut buf = [0u8; 32 * 1024];
            loop {
                let n = decoder.read(&mut buf).await.map_err(|error| {
                    AirportingError::DecompressData {
                        file_path: file_path.to_path_buf(),
                        error,
                    }
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
        })
    }

    /// Save the data file to disk (gzip) and update the meta data with the new uncompressed length and SHA256.
    async fn save_data_file(file_path: &Path, data: &[u8]) -> Result<SourceMetaData> {
        let length = data.len();
        let line_count = data.iter().filter(|&&byte| byte == b'\n').count();
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

    fn get_response_header(
        url: &str,
        response: &reqwest::Response,
        header: reqwest::header::HeaderName,
    ) -> Result<String> {
        let etag = response
            .headers()
            .get(&header)
            .ok_or(AirportingError::MissingHeader {
                header: header.clone(),
                url: url.to_string(),
            })?
            .to_str()
            .map_err(|error| AirportingError::InvalidHeader {
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
