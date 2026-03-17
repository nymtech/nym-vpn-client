// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct StExcludedProcess {
    pub pid: i32, // libc::pid_t
    pub exec_path: PathBuf,
    pub responsible_exec_path: PathBuf,
}

impl Ord for StExcludedProcess {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.is_child(), other.is_child()) {
            (true, true) | (false, false) => self.exec_path.cmp(&other.exec_path),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for StExcludedProcess {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl StExcludedProcess {
    /// Returns true if process is a child process or launched on behalf of another process
    fn is_child(&self) -> bool {
        self.exec_path != self.responsible_exec_path
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct StExcludedProcessList {
    pub processes: Vec<StExcludedProcess>,
}
