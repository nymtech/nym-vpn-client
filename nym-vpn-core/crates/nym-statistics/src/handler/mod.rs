// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::VpnStorage;
use static_information::StaticInformationHandler;
use usage::UsageHandler;

use crate::{events::StatisticsEvent, report::VpnStatsReport, storage::StatsStorage};

mod static_information;
mod usage;

pub(crate) struct StatisticsHandler<S>
where
    S: VpnStorage,
{
    static_information_handler: StaticInformationHandler,
    usage_handler: UsageHandler<S>,
    //SW TODO investigate using trait like Andrew did in Nym-nodes
}

impl<S> StatisticsHandler<S>
where
    S: VpnStorage,
{
    pub fn new(storage: StatsStorage<S>) -> Self {
        StatisticsHandler {
            static_information_handler: StaticInformationHandler::new(),
            usage_handler: UsageHandler::new(storage),
        }
    }

    pub fn handle_event(&mut self, event: StatisticsEvent) {
        match event {
            StatisticsEvent::Usage(e) => self.usage_handler.handle_event(e),
        }
    }

    pub fn get_report(&mut self, identifier: String) -> VpnStatsReport {
        VpnStatsReport::new(identifier, self.static_information_handler.get_report())
            .with_usage_report(self.usage_handler.get_report())
    }
}
