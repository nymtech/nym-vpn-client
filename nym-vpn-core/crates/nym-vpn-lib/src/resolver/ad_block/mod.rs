// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod files;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum AdBlockingError {
    #[error("failed to set up ad-blocking data directory {dir}")]
    CreateDirectory {
        dir: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to write ad-blocking file {file_path}")]
    WriteFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to read ad-blocking file {file_path}")]
    ReadFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to rename ad-blocking file {from} to {to}")]
    RenameFile {
        from: PathBuf,
        to: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to serialize ad-blocking meta file")]
    SerializeMetaFile {
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to deserialize ad-blocking meta file {file_path}")]
    DeserializeMetaFile {
        file_path: PathBuf,
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to fetch ad-blocking data file from {url}")]
    FetchData {
        url: String,
        #[source]
        error: reqwest::Error,
    },

    #[error("missing etag in ad-blocking data file response from {url}")]
    MissingEtag { url: String },

    #[error("invalid etag in ad-blocking data file response from {url}")]
    InvalidEtag {
        url: String,
        #[source]
        error: reqwest::header::ToStrError,
    },

    #[error("failed to compress ad-blocking file data")]
    CompressData {
        #[source]
        error: std::io::Error,
    },

    #[error("failed to decompress ad-blocking file data")]
    DecompressData {
        #[source]
        error: std::io::Error,
    },

    #[error(
        "invalid ad-blocking data file length in {file_path}. expected {expected} bytes, got {actual} bytes"
    )]
    InvalidDataFileLength {
        file_path: PathBuf,
        expected: usize,
        actual: usize,
    },

    #[error(
        "invalid ad-blocking data file SHA256 hash in {file_path}. expected {expected}, got {actual}"
    )]
    InvalidDataFileHash {
        file_path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("invalid ad-blocking file encoding in {file_path}")]
    InvalidDataFileEncoding {
        file_path: PathBuf,
        #[source]
        error: std::string::FromUtf8Error,
    },
}

pub(crate) type Result<T, E = AdBlockingError> = std::result::Result<T, E>;
