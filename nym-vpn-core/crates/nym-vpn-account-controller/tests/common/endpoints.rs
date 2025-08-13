// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::request::RequestZkNymRequestBody;
use nym_vpn_api_client::response::{
    NymErrorResponse, NymVpnAccountSummaryWithDeviceResponse, NymVpnDevice, NymVpnHealthResponse,
    NymVpnZkNymPost, NymVpnZkNymStatus,
};

use rand::distributions::{Alphanumeric, DistString};
use time::OffsetDateTime;
use wiremock::{
    Mock, Request, ResponseTemplate,
    matchers::{method, path, path_regex},
};

const ACCOUNT_REGEX: &str = r"n1\w*";
const DEVICE_REGEX: &str = r"\w*";
const ZK_NYM_REGEX: &str = r"\w*";

pub fn account_summary_with_device_200(response: NymVpnAccountSummaryWithDeviceResponse) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/summary"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
}

pub fn account_summary_with_device_403(error_response: NymErrorResponse) -> Mock {
    Mock::given(method("GET"))
        .and(path_regex(format!(
            "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/summary",
        )))
        .respond_with(ResponseTemplate::new(403).set_body_json(error_response))
}

pub fn register_account_200(response: NymVpnDevice) -> Mock {
    Mock::given(method("POST"))
        .and(path_regex(format!(
            "/public/v1/account/{ACCOUNT_REGEX}/device"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
}

pub fn zknym_200() -> Mock {
    Mock::given(method("POST"))
        .and(path_regex(format!(
            "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym",
        )))
        .respond_with(|req: &Request| {
            let request: RequestZkNymRequestBody = req.body_json().unwrap();
            let t_type = request.ticketbook_type;
            ResponseTemplate::new(200).set_body_json(test(t_type))
        })
}

// SW Implement this endpoint for polling
pub fn zknym_id_200() -> Mock {
    Mock::given(method("POST"))
        .and(path_regex(format!(
            "/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym/{ZK_NYM_REGEX}",
        )))
        .respond_with(|req: &Request| {
            let request: RequestZkNymRequestBody = req.body_json().unwrap();
            let t_type = request.ticketbook_type;
            ResponseTemplate::new(200).set_body_json(test(t_type))
        })
}

// httpmock doesn't allow dynamic answers, so that will only work for 60 secs, then it's gonna be considered desynced
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

pub fn test(ticketbook_type: String) -> NymVpnZkNymPost {
    let id: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 15);
    NymVpnZkNymPost {
        created_on_utc: "2025-08-06 13:13:21.456Z".into(),
        last_updated_utc: "2025-08-06 13:13:21.456Z".into(),
        id,
        ticketbook_type,
        valid_until_utc: "2025-09-05 13:13:16.747Z".into(),
        valid_from_utc: "2025-08-06 13:13:21.402Z".into(),
        issued_bandwidth_in_gb: 25f64,
        blinded_shares: None,
        status: NymVpnZkNymStatus::Pending,
    }
}
