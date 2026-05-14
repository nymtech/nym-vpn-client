// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use adblock::lists::FilterFormat;
use async_compression::tokio::{bufread::GzipDecoder, write::GzipEncoder};
use async_stream::try_stream;
use bytes::Bytes;
use futures::{
    AsyncBufReadExt, FutureExt, Stream, StreamExt, TryStreamExt, future::BoxFuture,
    stream::BoxStream,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};
use tokio_util::sync::CancellationToken;

use super::{AdBlockerError, Result};

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

#[async_trait::async_trait]
pub trait AdBlockFileManager: Send + Sync {
    /// Initialize the ad-blocker domain lists using the ones built-into the binary.
    async fn init_files(&self, force: bool) -> Result<()>;

    /// Update the ad-blocker domain lists by downloading the latest versions
    async fn update_files(&self, cancel_token: CancellationToken) -> Result<bool>;
}

pub struct RealFileManager {
    user_agent: String,
    cache_dir: PathBuf,
}

impl RealFileManager {
    pub fn new(user_agent: String, cache_dir: PathBuf) -> Self {
        Self {
            user_agent,
            cache_dir,
        }
    }
}

#[async_trait::async_trait]
impl AdBlockFileManager for RealFileManager {
    async fn init_files(&self, force: bool) -> Result<()> {
        init_files(self.cache_dir.as_ref(), force).await
    }

    async fn update_files(&self, cancel_token: CancellationToken) -> Result<bool> {
        update_files(
            self.cache_dir.as_ref(),
            self.user_agent.as_str(),
            cancel_token,
        )
        .await
    }
}

async fn init_files(cache_dir: &Path, force: bool) -> Result<()> {
    fs::create_dir_all(&cache_dir)
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

async fn update_files(
    cache_dir: &Path,
    user_agent: &str,
    cancel_token: CancellationToken,
) -> Result<bool> {
    let mut updated = false;
    let http_client = nym_http_api_client::registry::build_client()
        .map_err(|error| AdBlockerError::BuildHttpClient { error })?;

    for source in SOURCES.iter() {
        if source
            .update_data_file(
                cache_dir,
                &http_client,
                user_agent,
                cancel_token.child_token(),
            )
            .await?
        {
            updated = true;
        }
    }

    Ok(updated)
}

#[cfg(test)]
pub struct MockFileManager;

#[cfg(test)]
#[async_trait::async_trait]
impl AdBlockFileManager for MockFileManager {
    async fn init_files(&self, _force: bool) -> Result<()> {
        Ok(())
    }

    async fn update_files(&self, _cancel_token: CancellationToken) -> Result<bool> {
        Ok(true)
    }
}

// Static dispatch alternative to Arc<dyn AdBlockFileManager>
pub enum AdBlockFileManagerWrap {
    Real(RealFileManager),
    #[cfg(test)]
    Mock(MockFileManager),
}

#[async_trait::async_trait]
impl AdBlockFileManager for AdBlockFileManagerWrap {
    async fn init_files(&self, force: bool) -> Result<()> {
        match self {
            Self::Real(manager) => manager.init_files(force).await,
            #[cfg(test)]
            Self::Mock(manager) => manager.init_files(force).await,
        }
    }
    async fn update_files(&self, cancel_token: CancellationToken) -> Result<bool> {
        match self {
            Self::Real(manager) => manager.update_files(cancel_token).await,
            #[cfg(test)]
            Self::Mock(manager) => manager.update_files(cancel_token).await,
        }
    }
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

        let meta_path = cache_dir.join(self.meta_file_name);
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
        cache_dir: &Path,
        http_client: &reqwest::Client,
        user_agent: &str,
        cancel_token: CancellationToken,
    ) -> Result<bool> {
        if let Err(error) = Self::cleanup_temp_files(cache_dir).await {
            tracing::warn!("Failed to clean up temporary ad-blocker files: {error}; Ignoring.");
        }

        // Read the current meta file
        let meta_path = cache_dir.join(self.meta_file_name);
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
        let response = cancel_token
            .run_until_cancelled(request.send())
            .await
            .ok_or(AdBlockerError::Cancelled)?
            .map_err(|error| AdBlockerError::FetchData {
                url: self.url.to_string(),
                error,
            })?;

        match response.status() {
            reqwest::StatusCode::OK => {
                tracing::debug!("Received HTTP/200 for {}", self.url);
            }
            reqwest::StatusCode::NOT_MODIFIED => {
                tracing::debug!("Ad-blocker data file {} is up to date", self.file_name);
                return Ok(false);
            }
            status => {
                tracing::debug!("Unexpected response for {}: {}", self.url, status);
                return Ok(false);
            }
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

        // Stream the rest of the response to file on disk
        let response_stream = response
            .bytes_stream()
            .map_err(|err| AdBlockerError::FetchData {
                url: self.url.to_string(),
                error: err,
            });

        // Write the data to a temporary file in the ad-blocker directory
        let temp_data_path = cache_dir.join(Self::TEMP_DATA_FILE_NAME);
        let temp_meta_data = cancel_token
            .run_until_cancelled(Self::stream_to_data_file(
                &temp_data_path,
                response_stream,
                etag,
            ))
            .await
            .ok_or(AdBlockerError::Cancelled)??;

        if Self::contains_embedded_http_429_error(&temp_data_path).await? {
            tracing::warn!("Received embedded HTTP/429 for {}", self.url);
            return Ok(false);
        }

        // Write the new meta data to a temporary file in the ad-blocker directory
        let temp_meta_path = cache_dir.join(Self::TEMP_META_FILE_NAME);
        temp_meta_data.write_to_file(&temp_meta_path).await?;

        // Now all the data is on-disk, switch the old files with the new ones by renaming them.
        let data_path = cache_dir.join(self.file_name);
        fs::rename(&temp_data_path, &data_path)
            .await
            .map_err(|error| AdBlockerError::RenameFile {
                from: temp_data_path.clone(),
                to: data_path.clone(),
                error,
            })?;

        let meta_path = cache_dir.join(self.meta_file_name);
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

    /// Clean-up any extraneous temporary files that may be left over from a failed update.
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
    // Note: Pinned box is necessary in order to avoid "large future size" warnings.
    pub fn load_data_file<'a>(
        file_path: &'a Path,
        meta_data: &'a SourceMetaData,
    ) -> BoxFuture<'a, Result<String>> {
        async move {
            let file = File::open(file_path)
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
        .boxed()
    }

    /// Stream lines from a data file, decompressing it on the fly.
    pub fn stream_lines<'a>(file_path: &'a Path) -> BoxStream<'a, Result<String>> {
        // ugly: map to io::Error for compatibility with `AsyncBufReadExt`
        let stream = Self::stream_chunks(file_path).map_err(std::io::Error::other);
        let async_read = stream.into_async_read();

        // ugly: rewrap error back to custom error
        async_read
            .lines()
            .map_err(
                |err: std::io::Error| match err.downcast::<AdBlockerError>() {
                    Ok(err) => err,
                    Err(err) => AdBlockerError::UnknownLineReadError(err),
                },
            )
            .boxed()
    }

    /// Stream chunks from a data file, decompressing it on the fly.
    fn stream_chunks<'a>(file_path: &'a Path) -> BoxStream<'a, Result<Vec<u8>>> {
        try_stream! {
            let file = File::open(file_path)
                .await
                .map_err(|error| AdBlockerError::ReadFile {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;

            let reader = BufReader::new(file);
            let mut decoder = GzipDecoder::new(reader);

            // todo: consider verifying the hash and length of the decompressed data?
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

    /// Stream data to file on disk, compressing it on the fly, and update the meta data with the new uncompressed length and SHA256.
    async fn stream_to_data_file<S>(
        file_path: &Path,
        mut stream: S,
        etag: String,
    ) -> Result<SourceMetaData>
    where
        S: Stream<Item = Result<Bytes>> + Unpin,
    {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)
            .await
            .map_err(|error| AdBlockerError::OpenFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;
        let writer = BufWriter::new(file);

        let mut hasher = Sha256::new();
        let mut encoder = GzipEncoder::new(writer);

        let mut byte_len: usize = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            byte_len = byte_len.saturating_add(chunk.len());
            hasher.update(&chunk);
            encoder
                .write_all(&chunk)
                .await
                .map_err(|error| AdBlockerError::CompressData {
                    file_path: file_path.to_path_buf(),
                    error,
                })?;
        }

        let mut writer = encoder
            .shutdown()
            .await
            .map_err(|error| AdBlockerError::CompressData {
                file_path: file_path.to_path_buf(),
                error,
            })
            .map(|_| encoder.into_inner())?;

        // Ensure all data is written to disk
        writer
            .flush()
            .await
            .map_err(|error| AdBlockerError::FlushFile {
                file_path: file_path.to_path_buf(),
                error,
            })?;

        let sha256 = hex::encode(hasher.finalize());

        Ok(SourceMetaData {
            length: byte_len,
            etag,
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

    /// Returns `Ok(true)` if the first bytes of the given file contain embedded HTTP/429 error string.
    async fn contains_embedded_http_429_error(file_path: &Path) -> Result<bool> {
        const MATCH_ERROR_STRING: &str = "429: too many requests";

        let mut stream = Self::stream_lines(file_path);

        match stream.next().await {
            Some(res) => Ok(res?.to_lowercase().starts_with(MATCH_ERROR_STRING)),
            None => Ok(false),
        }
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

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    use std::path::Path;
    use tempfile::TempDir;
    use tokio_util::sync::CancellationToken;

    const USER_AGENT: &str = "nym-vpn-ad-blocker-tests/0.1";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_init_files() {
        let temp_dir = init_tests()
            .await
            .expect("Failed to initialize ad-blocker files");
        let cache_dir = temp_dir.path();

        for descr in SOURCES.iter() {
            let file_path = cache_dir.join(descr.file_name);
            assert!(
                file_path.exists(),
                "ad-blocker data file {} was not created",
                file_path.display()
            );

            let meta_file_path = cache_dir.join(descr.meta_file_name);
            assert!(
                meta_file_path.exists(),
                "ad-blocker meta file {} was not created",
                meta_file_path.display()
            );
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    #[ignore] // This test is not practical as the easylist_adservers.txt file changes very frequently
    async fn test_update_nothing() {
        let temp_dir = init_tests()
            .await
            .expect("Failed to initialize ad-blocker files");
        let cache_dir = temp_dir.path();

        let updated = update_files(cache_dir, USER_AGENT, CancellationToken::new())
            .await
            .expect("Failed to update ad-blocker files");

        assert!(
            !updated,
            "ad-blocker files were updated when they should not have been"
        );
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_update_0() {
        let temp_dir = init_tests()
            .await
            .expect("Failed to initialize ad-blocker files");
        let cache_dir = temp_dir.path();

        write_fake_etag(cache_dir, 0)
            .await
            .expect("Failed to update ad-blocker metadata");

        let updated = update_files(cache_dir, USER_AGENT, CancellationToken::new())
            .await
            .expect("Failed to update ad-blocker files");

        assert!(
            updated,
            "ad-blocker files were not updated when they should have been"
        );
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_update_1() {
        let temp_dir = init_tests()
            .await
            .expect("Failed to initialize ad-blocker files");
        let cache_dir = temp_dir.path();

        write_fake_etag(cache_dir, 1)
            .await
            .expect("Failed to update ad-blocker metadata");

        let updated = update_files(cache_dir, USER_AGENT, CancellationToken::new())
            .await
            .expect("Failed to update ad-blocker files");

        assert!(
            updated,
            "ad-blocker files were not updated when they should have been"
        );
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_contains_embedded_http_429_error() {
        let temp_dir = init_tests().await.unwrap();

        for source in SOURCES.iter() {
            let blocking_rules_file_path = temp_dir.path().join(source.file_name);
            assert!(
                !Source::contains_embedded_http_429_error(&blocking_rules_file_path)
                    .await
                    .unwrap()
            );
        }

        let test_file_path = temp_dir.path().join("test.txt.gz");
        let writer = File::create(&test_file_path).await.unwrap();
        let mut encoder = GzipEncoder::new(writer);
        encoder
            .write_all("429: Too Many Requests".as_bytes())
            .await
            .unwrap();
        encoder.shutdown().await.unwrap();
        encoder.into_inner().flush().await.unwrap();

        assert!(
            Source::contains_embedded_http_429_error(&test_file_path)
                .await
                .unwrap()
        );
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

    // Open the meta file and change the etag in order to force an update of the data file
    async fn write_fake_etag(ad_blocking_path: &Path, index: usize) -> Result<(), String> {
        let meta_path = ad_blocking_path.join(SOURCES[index].meta_file_name);
        let mut meta_data = SourceMetaData::from_file(&meta_path)
            .await
            .map_err(|e| format!("failed to read ad-blocker meta file: {e}"))?;
        meta_data.etag = "fake-etag".to_string();
        meta_data
            .write_to_file(&meta_path)
            .await
            .map_err(|e| format!("failed to write ad-blocker meta file: {e}"))?;
        Ok(())
    }
}
