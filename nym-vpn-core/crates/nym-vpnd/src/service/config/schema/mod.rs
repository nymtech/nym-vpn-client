// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! External, versioned, representation of the vpn service config file.

pub mod v1;
pub mod v2;

use serde::{Deserialize, Serialize};

pub use v1::{EntryPointExtV1, ExitPointExtV1, VpnServiceConfigExtV1};
pub use v2::VpnServiceConfigExtV2;

pub type VpnServiceConfigExtLatest = VpnServiceConfigExtV2;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
pub enum VpnServiceConfigExt {
    V1(VpnServiceConfigExtV1),
    V2(VpnServiceConfigExtV2),
}

impl From<VpnServiceConfigExtLatest> for VpnServiceConfigExt {
    fn from(latest: VpnServiceConfigExtLatest) -> Self {
        VpnServiceConfigExt::V2(latest)
    }
}

pub enum MigrationStatus {
    UpToDate,
    Migrated,
    UseDefault,
}

pub struct MigrationResult {
    pub config: VpnServiceConfigExtLatest,
    pub status: MigrationStatus,
}

pub fn migrate_if_needed(mut source: VpnServiceConfigExt) -> MigrationResult {
    let mut progressive_count = 0;
    let migrated_config = loop {
        match source {
            VpnServiceConfigExt::V1(v1) => {
                source = VpnServiceConfigExt::V2(VpnServiceConfigExtV2::from(v1));
            }
            VpnServiceConfigExt::V2(v2) => break v2,
        }
        progressive_count += 1;
    };

    let status = if progressive_count == 0 {
        MigrationStatus::UpToDate
    } else {
        MigrationStatus::Migrated
    };

    MigrationResult {
        config: migrated_config,
        status,
    }
}
