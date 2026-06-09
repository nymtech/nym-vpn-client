// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use futures::StreamExt;
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{UpdateOutcome, error::UpdaterError};

pub(crate) async fn download_file(
    http_client: &reqwest::Client,
    url: &Url,
    dest_path: &Path,
    cancel_token: CancellationToken,
) -> Result<UpdateOutcome, UpdaterError> {
    if let Some(parent) = dest_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .await
            .map_err(|error| UpdaterError::CreateDirectory {
                dir: parent.to_path_buf(),
                error,
            })?;
    }

    let etag_path = etag_path(dest_path);
    let current_etag = read_etag(&etag_path).await;

    let mut builder = http_client.get(url.as_str());
    if let Some(etag) = &current_etag {
        builder = builder.header(reqwest::header::IF_NONE_MATCH, etag);
    }

    let response = cancel_token
        .run_until_cancelled(builder.send())
        .await
        .ok_or(UpdaterError::Cancelled)?
        .map_err(|error| UpdaterError::Request {
            url: url.to_string(),
            error,
        })?;

    match response.status() {
        reqwest::StatusCode::OK => {
            tracing::debug!(%url, "Received 200, downloading");
        }
        reqwest::StatusCode::NOT_MODIFIED => {
            tracing::debug!(%url, "File not modified (304)");
            return Ok(UpdateOutcome::NotModified);
        }
        status => {
            return Err(UpdaterError::UnexpectedStatus {
                url: url.to_string(),
                status,
            });
        }
    }

    let new_etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);

    if new_etag.is_some() && new_etag == current_etag {
        tracing::warn!(%url, "Etag from {url} is unchanged, even though the server passed the If-None-Match check. Skipping file update.");
        return Ok(UpdateOutcome::NotModified);
    }

    let temp_path = temp_path(dest_path);
    let url_str = url.to_string();
    cancel_token
        .run_until_cancelled(write_response_to_file(response, &temp_path, &url_str))
        .await
        .ok_or(UpdaterError::Cancelled)??;

    fs::rename(&temp_path, dest_path)
        .await
        .map_err(|error| UpdaterError::RenameFile {
            from: temp_path,
            to: dest_path.to_path_buf(),
            error,
        })?;

    if let Some(etag) = new_etag {
        write_etag(&etag_path, &etag).await;
    }

    tracing::debug!("{} updated from {url}", dest_path.display());
    Ok(UpdateOutcome::Updated)
}

async fn write_response_to_file(
    response: reqwest::Response,
    path: &Path,
    url: &str,
) -> Result<(), UpdaterError> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .await
        .map_err(|error| UpdaterError::OpenFile {
            path: path.to_path_buf(),
            error,
        })?;

    let mut writer = BufWriter::new(file);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| UpdaterError::Download {
            url: url.to_owned(),
            error,
        })?;
        writer
            .write_all(&chunk)
            .await
            .map_err(|error| UpdaterError::WriteFile {
                path: path.to_path_buf(),
                error,
            })?;
    }

    writer
        .flush()
        .await
        .map_err(|error| UpdaterError::FlushFile {
            path: path.to_path_buf(),
            error,
        })?;

    Ok(())
}

fn etag_path(dest_path: &Path) -> PathBuf {
    let name = dest_path
        .file_name()
        .map(|n| format!("{}.etag", n.to_string_lossy()))
        .unwrap_or_else(|| "file.etag".to_owned());
    dest_path.with_file_name(name)
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

async fn write_etag(etag_path: &Path, etag: &str) {
    if let Err(error) = fs::write(etag_path, etag).await {
        tracing::warn!("Failed to write etag file {}: {error}", etag_path.display());
    }
}
