// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sdk::UserAgent;
use url::Url;

const STATS_REPORT_MAX_DELAY_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct StatisticsControllerConfig {
    /// Is stats reporting enabled
    pub(crate) enabled: bool,

    /// Address of the stats collector. If this is none, no reporting will happen, regardless of `enabled`
    pub stats_collector_url: Option<Url>,

    pub user_agent: UserAgent,

    // Allow sending reports even with a not connected tunnel
    // SW : This actually automatically takes the correct reportint channel :
    // - If the tunnel is not connected, it will go directly to the endpoint
    // - If the tunnel is connected in mixnet mode, it will go through the mixnet
    // - If the tunnel is connected in wireguard mode, it will go through wireguard
    // We thus need this option to allow users to report even if they can't connect (i.e. if they're being censored)
    /// Allow statistics sending, even if not conncetced to the mixnet/vpn
    pub(crate) allow_disconnected: bool,

    /// Maximum delay after which to send the report when connected
    pub(crate) max_reporting_delay: u64,

    /// Optional stats_id seed to override the random one
    pub(crate) stats_id_seed: Option<String>,
}

impl StatisticsControllerConfig {
    pub fn new(stats_collector_url: Option<Url>, user_agent: UserAgent) -> Self {
        StatisticsControllerConfig {
            enabled: true,
            stats_collector_url,
            user_agent,
            allow_disconnected: false,
            max_reporting_delay: STATS_REPORT_MAX_DELAY_SECS,
            stats_id_seed: None,
        }
    }

    #[must_use]
    pub fn with_allow_disconnected(mut self, allow_disonncected: bool) -> Self {
        self.allow_disconnected = allow_disonncected;
        self
    }

    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn with_max_reporting_delay(mut self, max_reporting_delay: u64) -> Self {
        self.max_reporting_delay = max_reporting_delay;
        self
    }

    #[must_use]
    pub fn with_stats_id_seed(mut self, stats_id_seed: Option<String>) -> Self {
        self.stats_id_seed = stats_id_seed;
        self
    }
}
