// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod download;
mod error;
mod updater;

pub use error::FileUpdaterError;
pub use updater::{FileUpdater, FileUpdaterHandle};

/// Outcome of a file update request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutcome {
    /// The file was downloaded and written to disk.
    Updated,
    /// The server indicated the file has not changed (HTTP 304 Not Modified).
    NotModified,
}
