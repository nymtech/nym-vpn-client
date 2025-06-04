// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to setup nym-vpn-api client")]
    SetupVpnApiClient(nym_statistics_api_client::StatisticsApiClientError),

    #[error("failed to post statistics report")]
    ReportSendingFailure(nym_statistics_api_client::StatisticsApiClientError),
}
