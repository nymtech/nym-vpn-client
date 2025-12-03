// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::NetworkStatisticsConfig;
use usage::UsageHandler;

use crate::{
    commands::ConfigCommand, events::StatisticsEvent,
    handler::report_sending::ReportSendingHandler, storage::StatsStorage,
};
use nym_statistics_api_client::StatisticsApiClient;

mod report_sending;
mod usage;

pub(crate) struct StatisticsHandler {
    storage: StatsStorage,

    // Handlers
    report_sending_handler: ReportSendingHandler,
    usage_handler: UsageHandler,
}

impl StatisticsHandler {
    pub fn new(
        storage: StatsStorage,
        stats_api_client: StatisticsApiClient,
        config: NetworkStatisticsConfig,
    ) -> Self {
        StatisticsHandler {
            storage: storage.clone(),
            report_sending_handler: ReportSendingHandler::new(
                storage.clone(),
                config,
                stats_api_client,
            ),
            usage_handler: UsageHandler::new(storage),
        }
    }

    pub(crate) async fn close(&self) {
        self.storage.close().await
    }

    pub async fn handle_event(&mut self, event: StatisticsEvent) {
        match event {
            StatisticsEvent::Usage(e) => self.usage_handler.handle_event(e).await,
            StatisticsEvent::ReportSending(e) => self.report_sending_handler.handle_event(e),
        }
    }

    pub async fn handle_command(&mut self, command: ConfigCommand) {
        self.usage_handler.handle_command(command).await;
        self.report_sending_handler.handle_command(command).await;
    }

    pub async fn on_shutdown(&mut self) {
        self.usage_handler.on_shutdown().await;
        self.report_sending_handler.on_shutdown().await;
    }
}
