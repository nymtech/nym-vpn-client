// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_statistics_common::report::vpn_client::StaticInformationReport;

use sysinfo::System;

pub(crate) struct StaticInformationHandler {
    static_report: StaticInformationReport,
}

impl StaticInformationHandler {
    pub(crate) fn new() -> Self {
        let bin_info = nym_bin_common::bin_info!();
        StaticInformationHandler {
            static_report: StaticInformationReport {
                os_type: System::distribution_id(),
                os_version: System::long_os_version(),
                os_arch: System::cpu_arch(),
                app_version: bin_info.build_version.into(),
            },
        }
    }

    pub(crate) fn get_report(&self) -> StaticInformationReport {
        self.static_report.clone()
    }
}
