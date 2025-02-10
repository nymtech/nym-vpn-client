// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_config::defaults::NymNetworkDetails;

// The response type we fetch from the discovery endpoint
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct DiscoveryResponse {
    pub(super) network_name: String,
    pub(super) nym_api_url: String,
    pub(super) nym_vpn_api_url: String,
    pub(super) account_management: Option<AccountManagementResponse>,
    pub(super) feature_flags: Option<serde_json::Value>,
    pub(super) system_configuration: Option<SystemConfigurationResponse>,
    pub(super) system_messages: Option<Vec<SystemMessageResponse>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(super) struct AccountManagementResponse {
    pub(super) url: String,
    pub(super) paths: AccountManagementPathsResponse,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(super) struct AccountManagementPathsResponse {
    pub(super) sign_up: String,
    pub(super) sign_in: String,
    pub(super) account: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct SystemMessageResponse {
    pub(super) name: String,
    pub(super) display_from: String,
    pub(super) display_until: String,
    pub(super) message: String,
    pub(super) properties: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(super) struct SystemConfigurationResponse {
    pub(super) mix_thresholds: ScoreThresholdsResponse,
    pub(super) wg_thresholds: ScoreThresholdsResponse,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(super) struct ScoreThresholdsResponse {
    pub(super) high: u8,
    pub(super) medium: u8,
    pub(super) low: u8,
}

// The response type we fetch from the network details endpoint. This will be added to and exported
// from nym-api-requests.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct NymNetworkDetailsResponse {
    pub(super) network: NymNetworkDetails,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct NymWellknownDiscoveryItem {
    pub(super) network_name: String,
    pub(super) nym_api_url: String,
    pub(super) nym_vpn_api_url: String,
}
