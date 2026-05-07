// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod engines;
mod file_manager;
mod state;

mod adblock;
pub use adblock::AdBlocker;

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum AdBlockerError {
    #[error("failed to set up ad-blocker data directory {dir}")]
    CreateDirectory {
        dir: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to open ad-blocker file for writing {file_path}")]
    OpenFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to write ad-blocker file {file_path}")]
    WriteFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to flush ad-blocker file {file_path}")]
    FlushFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to read ad-blocker file {file_path}")]
    ReadFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to rename ad-blocker file {from} to {to}")]
    RenameFile {
        from: PathBuf,
        to: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to remove ad-blocker file {file_path}")]
    RemoveFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to serialize ad-blocker meta file")]
    SerializeMetaFile {
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to deserialize ad-blocker meta file {file_path}")]
    DeserializeMetaFile {
        file_path: PathBuf,
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to fetch ad-blocker data file from {url}")]
    FetchData {
        url: String,
        #[source]
        error: reqwest::Error,
    },

    #[error("missing header {header} in ad-blocker data file response from {url}")]
    MissingHeader {
        header: reqwest::header::HeaderName,
        url: String,
    },

    #[error("invalid header {header} in ad-blocker data file response from {url}")]
    InvalidHeader {
        header: reqwest::header::HeaderName,
        url: String,
        #[source]
        error: reqwest::header::ToStrError,
    },

    #[error("failed to compress ad-blocker file data to {file_path}")]
    CompressData {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to decompress ad-blocker file data from {file_path}")]
    DecompressData {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error(
        "invalid ad-blocker data file length in {file_path}. expected {expected} bytes, got {actual} bytes"
    )]
    InvalidDataFileLength {
        file_path: PathBuf,
        expected: usize,
        actual: usize,
    },

    #[error(
        "invalid ad-blocker data file SHA256 hash in {file_path}. expected {expected}, got {actual}"
    )]
    InvalidDataFileHash {
        file_path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("invalid ad-blocker file encoding in {file_path}")]
    InvalidDataFileEncoding {
        file_path: PathBuf,
        #[source]
        error: std::string::FromUtf8Error,
    },

    #[error("failed to create adblock engine request for URL {url}")]
    CreateRequest {
        url: String,
        #[source]
        error: ::adblock::request::RequestError,
    },

    #[error("failed to build ad-blocker HTTP client")]
    BuildHttpClient {
        #[source]
        error: reqwest::Error,
    },

    #[error("unknown line reader error")]
    UnknownLineReadError(#[source] std::io::Error),

    #[error("failed to open database")]
    OpenDb(#[source] sqlx::Error),

    #[error("failed to migrate database")]
    MigrateDb(#[source] sqlx::migrate::MigrateError),

    #[error("failed to acquire connection to database")]
    AcquireDbConnection(#[source] sqlx::Error),

    #[error("failed to populate database")]
    PopulateDb(#[source] sqlx::Error),

    #[error("cancelled")]
    Cancelled,
}

impl AdBlockerError {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, AdBlockerError::Cancelled)
    }
}

pub type Result<T, E = AdBlockerError> = std::result::Result<T, E>;
