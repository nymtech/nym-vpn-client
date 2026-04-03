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
pub struct SplitTunnelExcludedProcess {
    pub pid: i32, // libc::pid_t
    pub exec_path: PathBuf,
    // Executable responsible for launching binary at `exec_path` (macOS only)
    pub responsible_exec_path: Option<PathBuf>,
    // Parent executables that led to launch of binary at `exec_path` (Linux only)
    pub ancestor_exec_paths: Vec<PathBuf>,
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
pub struct SplitTunnelExcludedProcessList {
    pub processes: Vec<SplitTunnelExcludedProcess>,
}
