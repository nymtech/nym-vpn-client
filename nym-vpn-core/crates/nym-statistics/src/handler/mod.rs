// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_statistics_common::{
    generate_vpn_client_stats_id, report::vpn_client::VpnClientStatsReport,
};
use nym_vpn_store::VpnStorage;
use static_information::StaticInformationHandler;
use usage::UsageHandler;

use crate::{
    config::StatisticsControllerConfig, error::Error, events::StatisticsEvent,
    storage::StatsStorage,
};

mod static_information;
mod usage;

pub(crate) struct StatisticsHandler<S>
where
    S: VpnStorage,
{
    storage: StatsStorage<S>,
    config: StatisticsControllerConfig,

    static_information_handler: StaticInformationHandler,
    usage_handler: UsageHandler<S>,
    //SW TODO investigate using trait like Andrew did in Nym-nodes
}

impl<S> StatisticsHandler<S>
where
    S: VpnStorage,
{
    pub fn new(storage: StatsStorage<S>, config: StatisticsControllerConfig) -> Self {
        StatisticsHandler {
            storage: storage.clone(),
            config,
            static_information_handler: StaticInformationHandler::new(),
            usage_handler: UsageHandler::new(storage),
        }
    }

    pub fn handle_event(&mut self, event: StatisticsEvent) {
        match event {
            StatisticsEvent::Usage(e) => self.usage_handler.handle_event(e),
        }
    }

    pub async fn get_report(&mut self) -> Result<VpnClientStatsReport, Error> {
        // Use seed override or storage one
        let seed = self
            .config
            .stats_id_seed
            .clone()
            .unwrap_or(self.storage.maybe_init_and_load_seed().await?);
        let identifier = generate_vpn_client_stats_id(seed);
        Ok(
            VpnClientStatsReport::new(identifier, self.static_information_handler.get_report())
                .with_usage_report(self.usage_handler.get_report()),
        )
    }
}
