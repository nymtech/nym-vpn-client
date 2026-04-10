// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_lib_types::{SplitTunnelExcludedProcess, SplitTunnelExcludedProcessList};

use crate::proto;

impl From<proto::SplitTunnelExcludedProcess> for SplitTunnelExcludedProcess {
    fn from(value: proto::SplitTunnelExcludedProcess) -> Self {
        Self {
            pid: value.pid,
            exec_path: PathBuf::from(value.exec_path),
            responsible_exec_path: PathBuf::from(value.responsible_exec_path),
        }
    }
}

impl From<SplitTunnelExcludedProcess> for proto::SplitTunnelExcludedProcess {
    fn from(value: SplitTunnelExcludedProcess) -> Self {
        Self {
            pid: value.pid,
            exec_path: value.exec_path.display().to_string(),
            responsible_exec_path: value.responsible_exec_path.display().to_string(),
        }
    }
}

impl From<proto::SplitTunnelExcludedProcessList> for SplitTunnelExcludedProcessList {
    fn from(value: proto::SplitTunnelExcludedProcessList) -> Self {
        Self {
            processes: value
                .processes
                .into_iter()
                .map(SplitTunnelExcludedProcess::from)
                .collect(),
        }
    }
}

impl From<SplitTunnelExcludedProcessList> for proto::SplitTunnelExcludedProcessList {
    fn from(value: SplitTunnelExcludedProcessList) -> Self {
        Self {
            processes: value
                .processes
                .into_iter()
                .map(proto::SplitTunnelExcludedProcess::from)
                .collect(),
        }
    }
}
