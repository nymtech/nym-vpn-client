// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::response::{
    NymErrorResponse, NymVpnAccountSummaryWithDeviceResponse, NymVpnDevice, NymVpnHealthResponse,
};

use time::OffsetDateTime;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path, path_regex},
};

use crate::common::{
    account_summary::{mock_active_devices_response, mock_devices_response, mock_usage_response},
    credential_proxy::MockCredentialProxy,
};

const ACCOUNT_REGEX: &str = r"n1\w*";
const DEVICE_REGEX: &str = r"\w*";
const ZK_NYM_REGEX: &str = r"\w*";

// We use Wiremock to allow dynamic response

// Health endpoints

/// Mock the health endpoint, with the current timestamp
pub fn synced_health() -> Mock {
    Mock::given(method("GET"))
        .and(path("/public/v1/health"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(NymVpnHealthResponse {
                status: "ok".to_string(),
                timestamp_utc: OffsetDateTime::now_utc(),
            }),
        )
}

/// Mock the health endpoint, with an out of sync timestamp
pub fn desynced_health() -> Mock {
    Mock::given(method("GET"))
        .and(path("/public/v1/health"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(NymVpnHealthResponse {
                status: "ok".to_string(),
                timestamp_utc: OffsetDateTime::from_unix_timestamp(42).unwrap(),
            }),
        )
}

// Account summary endpoint. Give it the response you want
pub fn account_summary_with_device_200(response: NymVpnAccountSummaryWithDeviceResponse) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/summary$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
}

pub fn account_summary_with_device_403(error_response: NymErrorResponse) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/summary$",
        )))
        .respond_with(ResponseTemplate::new(403).set_body_json(error_response))
}

pub fn account_update_device_200(response: NymVpnDevice) -> Mock {
    Mock::given(method("PATCH"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
}

pub fn register_account_200(response: NymVpnDevice) -> Mock {
    Mock::given(method("POST"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
}

pub fn register_account_403(error_response: NymErrorResponse) -> Mock {
    Mock::given(method("POST"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device$"
        )))
        .respond_with(ResponseTemplate::new(403).set_body_json(error_response))
}

pub fn get_device_by_id_200(response: NymVpnDevice) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
}

pub fn get_device_by_id_404() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}$"
        )))
        .respond_with(ResponseTemplate::new(404))
}

// ZK-Nyms endpoints. They need a MockCredentialProxy

pub fn zknym_id(credential_proxy: MockCredentialProxy) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym/{ZK_NYM_REGEX}$",
        )))
        .respond_with(credential_proxy.zknym_id())
}

pub fn zknym_post(credential_proxy: MockCredentialProxy) -> Mock {
    Mock::given(method("POST"))
        .and(path_regex(format!(
            "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym",
        )))
        .respond_with(credential_proxy.zknym_post())
}

pub fn zknym_available_200(credential_proxy: MockCredentialProxy) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym/available$",
        )))
        .respond_with(credential_proxy.zknym_available_200())
        .with_priority(10) // We want this to have precedence over zknym_id
}

pub fn partial_verification_key_200(credential_proxy: MockCredentialProxy) -> Mock {
    Mock::given(method("GET"))
        .and(path(
            "/public/v1/directory/zk-nyms/ticketbook/partial-verification-keys",
        ))
        .respond_with(credential_proxy.partial_verification_key_200())
}

pub fn confirm_zk_nym_download_by_id_200(credential_proxy: MockCredentialProxy) -> Mock {
    Mock::given(method("DELETE"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym/{ZK_NYM_REGEX}$",
        )))
        .respond_with(credential_proxy.confirm_zk_nym_download_by_id_200())
}

// Other

pub fn get_usage_200() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/usage$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_usage_response()))
}

pub fn get_devices_200() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_devices_response()))
}

pub fn get_active_devices_200() -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "^/public/v1/account/{ACCOUNT_REGEX}/device/active$"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_active_devices_response()))
}
