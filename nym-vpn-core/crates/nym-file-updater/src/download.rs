// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use futures::StreamExt;
use nym_http_api_client::{ApiClientCore, Client as HttpClient};
use reqwest::Method;
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{UpdateOutcome, error::FileUpdaterError};

/// Connection timeout applied to the dedicated client used for each download.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) async fn download_file(
    url: &Url,
    dest_path: &Path,
    cancel_token: CancellationToken,
) -> Result<UpdateOutcome, FileUpdaterError> {
    if let Some(parent) = dest_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .await
            .map_err(|error| FileUpdaterError::CreateDirectory {
                dir: parent.to_path_buf(),
                error,
            })?;
    }

    // Bound to `url` itself so every request targets exactly the download URL. Built from the
    // registry-configured builder (not `reqwest::Client::builder()`) so platform-specific TLS
    // overrides (e.g. Android's webpki-roots backend, needed because rustls-platform-verifier
    // isn't initialized in this process) still apply.
    let http_client = HttpClient::builder(url.clone())
        .and_then(|builder| {
            builder
                .with_reqwest_builder(
                    nym_http_api_client::registry::default_builder().connect_timeout(CONNECT_TIMEOUT),
                )
                .build()
        })
        .map_err(|error| FileUpdaterError::BuildHttpClient { error })?;

    let etag_path = etag_path(dest_path);
    let current_etag = read_etag(&etag_path).await;

    // Step 1: HEAD request to read the server's current ETag without downloading.
    if let Some(ref stored) = current_etag {
        let head_request = http_client
            .create_request(Method::HEAD, "", nym_http_api_client::NO_PARAMS, None::<&()>)
            .map_err(|error| FileUpdaterError::Request {
                url: url.to_string(),
                error,
            })?;
        let head = cancel_token
            .run_until_cancelled(http_client.send(head_request))
            .await
            .ok_or(FileUpdaterError::Cancelled)?
            .map_err(|error| FileUpdaterError::Request {
                url: url.to_string(),
                error,
            })?;

        // Step 2: Compare with stored ETag.
        let server_etag = head
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        // Step 3: Match → skip the download entirely.
        if server_etag.as_deref() == Some(stored.as_str()) {
            tracing::debug!("HEAD etag matches from {url}, skipping download");
            return Ok(UpdateOutcome::NotModified);
        }
    }

    // Step 4: Different (or no stored etag) → full GET, write file, store new ETag.
    let get_request = http_client
        .create_request(Method::GET, "", nym_http_api_client::NO_PARAMS, None::<&()>)
        .map_err(|error| FileUpdaterError::Request {
            url: url.to_string(),
            error,
        })?;
    let response = cancel_token
        .run_until_cancelled(http_client.send(get_request))
        .await
        .ok_or(FileUpdaterError::Cancelled)?
        .map_err(|error| FileUpdaterError::Request {
            url: url.to_string(),
            error,
        })?;

    match response.status() {
        reqwest::StatusCode::OK => {
            tracing::debug!("Received 200 from {url}, downloading");
        }
        reqwest::StatusCode::NOT_MODIFIED => {
            tracing::debug!("File not modified (304) from {url}");
            return Ok(UpdateOutcome::NotModified);
        }
        status => {
            return Err(FileUpdaterError::UnexpectedStatus {
                url: url.to_string(),
                status,
            });
        }
    }

    let new_etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
        .ok_or_else(|| FileUpdaterError::MissingEtag {
            url: url.to_string(),
        })?;

    let temp_path = temp_path(dest_path);
    let url_str = url.to_string();
    cancel_token
        .run_until_cancelled(write_response_to_file(response, &temp_path, &url_str))
        .await
        .ok_or(FileUpdaterError::Cancelled)??;

    fs::rename(&temp_path, dest_path)
        .await
        .map_err(|error| FileUpdaterError::RenameFile {
            from: temp_path,
            to: dest_path.to_path_buf(),
            error,
        })?;

    write_etag(&etag_path, &new_etag).await?;

    tracing::debug!("{} updated from {url}", dest_path.display());
    Ok(UpdateOutcome::Updated)
}

async fn write_response_to_file(
    response: reqwest::Response,
    path: &Path,
    url: &str,
) -> Result<(), FileUpdaterError> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .await
        .map_err(|error| FileUpdaterError::OpenFile {
            path: path.to_path_buf(),
            error,
        })?;

    let mut writer = BufWriter::new(file);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| FileUpdaterError::Download {
            url: url.to_owned(),
            error,
        })?;
        writer
            .write_all(&chunk)
            .await
            .map_err(|error| FileUpdaterError::WriteFile {
                path: path.to_path_buf(),
                error,
            })?;
    }

    writer
        .flush()
        .await
        .map_err(|error| FileUpdaterError::FlushFile {
            path: path.to_path_buf(),
            error,
        })?;

    Ok(())
}

/// Derives the `.etag` path from the `.gz` dest path: `foo.txt.gz` → `foo.txt.etag`.
fn etag_path(dest_path: &Path) -> PathBuf {
    dest_path.with_extension("etag")
}

fn temp_path(dest_path: &Path) -> PathBuf {
    let name = dest_path
        .file_name()
        .map(|n| format!("{}.tmp", n.to_string_lossy()))
        .unwrap_or_else(|| "file.tmp".to_owned());
    dest_path.with_file_name(name)
}

async fn read_etag(etag_path: &Path) -> Option<String> {
    fs::read_to_string(etag_path)
        .await
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

async fn write_etag(etag_path: &Path, etag: &str) -> Result<(), FileUpdaterError> {
    fs::write(etag_path, etag)
        .await
        .map_err(|error| FileUpdaterError::WriteFile {
            path: etag_path.to_path_buf(),
            error,
        })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use tokio_util::sync::CancellationToken;
    use url::Url;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    use super::download_file;
    use crate::UpdateOutcome;

    const TEST_ETAG: &str = r#""v1-abc123""#;
    const TEST_BODY: &[u8] = b"fake filter content";

    /// HEAD returns the matching ETag; GET is never issued.
    async fn server_unchanged() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("HEAD"))
            .respond_with(ResponseTemplate::new(200).insert_header("ETag", TEST_ETAG))
            .mount(&server)
            .await;
        server
    }

    /// HEAD returns a new ETag; GET delivers the updated file.
    async fn server_updated(new_etag: &'static str) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("HEAD"))
            .respond_with(ResponseTemplate::new(200).insert_header("ETag", new_etag))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("ETag", new_etag)
                    .set_body_bytes(TEST_BODY),
            )
            .mount(&server)
            .await;
        server
    }

    /// No stored etag — HEAD is skipped, GET downloads the file.
    async fn server_first_download() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("ETag", TEST_ETAG)
                    .set_body_bytes(TEST_BODY),
            )
            .mount(&server)
            .await;
        server
    }

    fn make_url(server: &MockServer, path: &str) -> Url {
        format!("{}/{}", server.uri(), path).parse().unwrap()
    }

    /// HEAD etag matches stored etag → skip download, return NotModified.
    #[tokio::test]
    async fn test_not_modified_when_head_etag_matches() {
        let server = server_unchanged().await;

        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("filters.txt.gz");
        tokio::fs::write(dest_path.with_extension("etag"), TEST_ETAG)
            .await
            .unwrap();

        let outcome = download_file(
            &make_url(&server, "filters.txt.gz"),
            &dest_path,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(matches!(outcome, UpdateOutcome::NotModified));
        assert!(!dest_path.exists(), "file should not be written");
    }

    /// HEAD etag differs from stored etag → GET, write file, store new etag.
    #[tokio::test]
    async fn test_updated_when_head_etag_differs() {
        let new_etag = r#""v2-xyz789""#;
        let server = server_updated(new_etag).await;

        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("filters.txt.gz");
        let etag_path = dest_path.with_extension("etag");
        tokio::fs::write(&etag_path, TEST_ETAG).await.unwrap();

        let outcome = download_file(
            &make_url(&server, "filters.txt.gz"),
            &dest_path,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(matches!(outcome, UpdateOutcome::Updated));
        assert!(dest_path.exists());
        assert_eq!(
            tokio::fs::read_to_string(&etag_path).await.unwrap(),
            new_etag
        );
    }

    /// No stored etag (first run) → HEAD skipped, GET downloads and stores etag.
    #[tokio::test]
    async fn test_first_download_writes_file_and_etag() {
        let server = server_first_download().await;

        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("filters.txt.gz");
        let etag_path = dest_path.with_extension("etag");

        let outcome = download_file(
            &make_url(&server, "filters.txt.gz"),
            &dest_path,
            CancellationToken::new(),
        )
        .await
        .unwrap();

        assert!(matches!(outcome, UpdateOutcome::Updated));
        assert!(dest_path.exists());
        assert_eq!(
            tokio::fs::read_to_string(&etag_path).await.unwrap(),
            TEST_ETAG
        );
    }
}
