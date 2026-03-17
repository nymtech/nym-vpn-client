// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_lib_types::{StExcludedProcess, StExcludedProcessList};

use crate::proto;

impl From<proto::StExcludedProcess> for StExcludedProcess {
    fn from(value: proto::StExcludedProcess) -> Self {
        Self {
            pid: value.pid,
            exec_path: PathBuf::from(value.exec_path),
            responsible_exec_path: PathBuf::from(value.responsible_exec_path),
        }
    }
}

impl From<StExcludedProcess> for proto::StExcludedProcess {
    fn from(value: StExcludedProcess) -> Self {
        Self {
            pid: value.pid,
            exec_path: value.exec_path.display().to_string(),
            responsible_exec_path: value.responsible_exec_path.display().to_string(),
        }
    }
}

impl From<proto::StExcludedProcessList> for StExcludedProcessList {
    fn from(value: proto::StExcludedProcessList) -> Self {
        Self {
            processes: value
                .processes
                .into_iter()
                .map(StExcludedProcess::from)
                .collect(),
        }
    }
}

impl From<StExcludedProcessList> for proto::StExcludedProcessList {
    fn from(value: StExcludedProcessList) -> Self {
        Self {
            processes: value
                .processes
                .into_iter()
                .map(proto::StExcludedProcess::from)
                .collect(),
        }
    }
}
