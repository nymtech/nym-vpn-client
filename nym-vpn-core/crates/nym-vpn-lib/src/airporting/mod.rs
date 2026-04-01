// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

pub mod files;

#[derive(thiserror::Error, Debug)]
pub enum AirportingError {
    #[error("data path is not available")]
    DataPathUnavailable,

    #[error("failed to set up airporting data directory {dir}")]
    CreateDirectory {
        dir: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to write airporting file {file_path}")]
    WriteFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to read airporting file {file_path}")]
    ReadFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to rename airporting file {from} to {to}")]
    RenameFile {
        from: PathBuf,
        to: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to remove airporting file {file_path}")]
    RemoveFile {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to serialize airporting meta file")]
    SerializeMetaFile {
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to deserialize airporting meta file {file_path}")]
    DeserializeMetaFile {
        file_path: PathBuf,
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to deserialize airporting meta data")]
    DeserializeMetaData {
        #[source]
        error: serde_json::Error,
    },

    #[error("failed to fetch airporting data file from {url}")]
    FetchData {
        url: String,
        #[source]
        error: reqwest::Error,
    },

    #[error("missing header {header} in airporting data file response from {url}")]
    MissingHeader {
        header: reqwest::header::HeaderName,
        url: String,
    },

    #[error("invalid header {header} in airporting data file response from {url}")]
    InvalidHeader {
        header: reqwest::header::HeaderName,
        url: String,
        #[source]
        error: reqwest::header::ToStrError,
    },

    #[error("failed to compress airporting file data to {file_path}")]
    CompressData {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to decompress airporting file data from {file_path}")]
    DecompressData {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error(
        "invalid airporting data file length in {file_path}. expected {expected} bytes, got {actual} bytes"
    )]
    InvalidDataFileLength {
        file_path: PathBuf,
        expected: usize,
        actual: usize,
    },

    #[error(
        "invalid airporting data file SHA256 hash in {file_path}. expected {expected}, got {actual}"
    )]
    InvalidDataFileHash {
        file_path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("invalid airporting file encoding in {file_path}")]
    InvalidDataFileEncoding {
        file_path: PathBuf,
        #[source]
        error: std::string::FromUtf8Error,
    },

    #[error("failed to create airporting update request for URL {url}")]
    CreateRequest {
        url: String,
        #[source]
        error: adblock::request::RequestError,
    },

    #[error("failed to parse IP network '{ip_network}' from airporting data file {file_path}")]
    ParseIpNetwork {
        ip_network: String,
        file_path: PathBuf,
        #[source]
        error: ipnetwork::IpNetworkError,
    },
}

pub(crate) type Result<T, E = AirportingError> = std::result::Result<T, E>;
