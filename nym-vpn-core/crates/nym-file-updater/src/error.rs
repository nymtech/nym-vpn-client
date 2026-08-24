// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileUpdaterError {
    #[error("Failed to build HTTP client: {error}")]
    BuildHttpClient {
        error: nym_http_api_client::HttpClientError,
    },

    #[error("HTTP request failed for {url}: {error}")]
    Request {
        url: String,
        error: nym_http_api_client::HttpClientError,
    },

    #[error("Failed to download chunk from {url}: {error}")]
    Download { url: String, error: reqwest::Error },

    #[error("Unexpected HTTP status {status} for {url}")]
    UnexpectedStatus {
        url: String,
        status: reqwest::StatusCode,
    },

    #[error("Failed to create directory {dir}: {error}")]
    CreateDirectory { dir: PathBuf, error: std::io::Error },

    #[error("Failed to open {path}: {error}")]
    OpenFile {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("Failed to write to {path}: {error}")]
    WriteFile {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("Failed to flush {path}: {error}")]
    FlushFile {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("Failed to rename {from} to {to}: {error}")]
    RenameFile {
        from: PathBuf,
        to: PathBuf,
        error: std::io::Error,
    },

    #[error("Response from {url} did not include an ETag header")]
    MissingEtag { url: String },

    #[error("Update cancelled")]
    Cancelled,

    #[error("File updater channel closed")]
    ChannelClosed,
}
