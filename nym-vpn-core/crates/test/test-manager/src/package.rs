// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Architecture, OsType, PackageType, VmConfig};
use anyhow::{Context, Result, bail};
use itertools::Itertools;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[derive(Debug, Clone)]
pub struct Manifest {
    pub app_package_path: PathBuf,
    pub app_package_to_upgrade_from_path: Option<PathBuf>,
    pub gui_package_path: Option<PathBuf>,
}

/// Basic metadata about the test runner target platform such as OS, architecture and package
/// manager.
#[derive(Debug, Clone, Copy)]
pub enum TargetInfo {
    Windows {
        arch: Architecture,
    },
    Macos {
        arch: Architecture,
    },
    Linux {
        arch: Architecture,
        package_type: PackageType,
    },
}

pub fn get_version_from_path(app_package_path: &Path) -> Result<String, anyhow::Error> {
    static VERSION_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\d{4}\.\d+((-beta\d+)?(-dev)?-([0-9a-z])+)?").unwrap());

    VERSION_REGEX
        .captures(app_package_path.to_str().unwrap())
        .with_context(|| format!("Cannot parse version: {}", app_package_path.display()))?
        .get(0)
        .map(|c| c.as_str().to_owned())
        .context("Could not parse version from package name: {app_package}")
}

impl TargetInfo {
    const fn is_linux(self) -> bool {
        matches!(self, TargetInfo::Linux { .. })
    }

    const fn get_ext(self) -> &'static str {
        match self {
            TargetInfo::Windows { .. } => "exe",
            TargetInfo::Macos { .. } => "pkg",
            TargetInfo::Linux { package_type, .. } => match package_type {
                PackageType::Deb => "deb",
                PackageType::Rpm => "rpm",
            },
        }
    }

    const fn get_os_name(self) -> &'static str {
        match self {
            TargetInfo::Windows { .. } => "windows",
            TargetInfo::Macos { .. } => "apple",
            TargetInfo::Linux { .. } => "linux",
        }
    }

    fn get_identifiers(self) -> impl Iterator<Item = &'static str> {
        match self {
            TargetInfo::Windows { arch }
            | TargetInfo::Macos { arch }
            | TargetInfo::Linux { arch, .. } => arch.get_identifiers().into_iter(),
        }
    }
}

impl TryFrom<&VmConfig> for TargetInfo {
    type Error = anyhow::Error;

    fn try_from(config: &VmConfig) -> std::result::Result<Self, Self::Error> {
        let target_info = match config.os_type {
            OsType::Windows => TargetInfo::Windows {
                arch: config.architecture,
            },
            OsType::Macos => TargetInfo::Macos {
                arch: config.architecture,
            },
            OsType::Linux => {
                let Some(package_type) = config.package_type else {
                    bail!("Linux VM configuration did not specify any package type (Deb|Rpm)!");
                };
                TargetInfo::Linux {
                    arch: config.architecture,
                    package_type,
                }
            }
        };
        Ok(target_info)
    }
}
