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

    #[error("failed to write ad-blocker file {file_path}")]
    WriteFile {
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

    #[error("failed to decompress ad-blocker file data from {file_path}")]
    DecompressData {
        file_path: PathBuf,
        #[source]
        error: std::io::Error,
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
}

pub type Result<T, E = AdBlockerError> = std::result::Result<T, E>;
