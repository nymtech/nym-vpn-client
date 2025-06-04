// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

const KIND: &str = "vpn_client_stats_report";
const VERSION: &str = "v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnStatsReport {
    pub(crate) kind: String,
    pub(crate) api_version: String,
    pub(crate) stats_id: String,
    pub static_information: StaticInformationReport,
    //SW called it basic so we can swap it easily down the line for more data
    pub basic_usage: Option<UsageReport>,
    // pub censorship: CensorshipReport,

    // pub gateway_quality: GatewayQualityReport,
}

impl VpnStatsReport {
    pub fn new(stats_id: String, static_information: StaticInformationReport) -> Self {
        VpnStatsReport {
            kind: KIND.into(),
            api_version: VERSION.into(),
            stats_id,
            static_information,
            basic_usage: None,
        }
    }

    #[must_use]
    pub fn with_usage_report(mut self, usage_report: UsageReport) -> Self {
        self.basic_usage = Some(usage_report);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticInformationReport {
    pub(crate) os_type: String,
    pub(crate) os_version: Option<String>,
    pub(crate) os_arch: String,
    pub(crate) app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    pub(crate) connection_time_ms: Option<u128>,
    pub(crate) two_hop: bool,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CensorshipReport {}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct GatewayQualityReport {}
