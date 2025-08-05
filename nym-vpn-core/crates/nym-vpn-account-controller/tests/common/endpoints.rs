// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use httpmock::{Method::GET, MockServer, Regex};
use nym_vpn_api_client::response::{
    NymErrorResponse, NymVpnAccountSummaryWithDeviceResponse, NymVpnHealthResponse,
};
use time::OffsetDateTime;

const ACCOUNT_REGEX: &str = r"n1\w*";
const DEVICE_REGEX: &str = r"\w*";

pub fn account_summary_with_device_200(
    server: &MockServer,
    response: NymVpnAccountSummaryWithDeviceResponse,
) {
    server.mock(|when, then| {
        when.method(GET).path_matches(
            Regex::new(&format!(
                "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/summary"
            ))
            .unwrap(),
        );
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&response);
    });
}

pub fn account_summary_with_device_403(server: &MockServer, error_response: NymErrorResponse) {
    server.mock(|when, then| {
        when.method(GET).path_matches(
            Regex::new(&format!(
                "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/summary"
            ))
            .unwrap(),
        );
        then.status(403)
            .header("content-type", "application/json")
            .json_body_obj(&error_response);
    });
}

// httpmock doesn't allow dynamic answers, so that will only work for 60 secs, then it's gonna be considered desynced
pub fn synced_health(server: &MockServer) {
    let response = NymVpnHealthResponse {
        status: "ok".to_string(),
        timestamp_utc: OffsetDateTime::now_utc(),
    };
    server.mock(|when, then| {
        when.method(GET).path("/public/v1/health");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&response);
    });
}

pub fn desynced_health(server: &MockServer) {
    let response = NymVpnHealthResponse {
        status: "ok".to_string(),
        timestamp_utc: OffsetDateTime::from_unix_timestamp(42).unwrap(),
    };
    server.mock(|when, then| {
        when.method(GET).path("/public/v1/health");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&response);
    });
}
