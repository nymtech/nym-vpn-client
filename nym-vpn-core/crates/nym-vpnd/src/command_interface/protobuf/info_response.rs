// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::VpnServiceInfoResult;
use nym_vpn_proto::InfoResponse;

impl From<VpnServiceInfoResult> for InfoResponse {
    fn from(info: VpnServiceInfoResult) -> Self {
        let build_timestamp = info.build_timestamp.map(offset_datetime_to_timestamp);

        let endpoints = info
            .endpoints
            .into_iter()
            .map(validator_details_to_endpoints)
            .collect();

        let nym_vpn_api_url = info.nym_vpn_api_url.map(string_to_url);

        InfoResponse {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            git_commit: info.git_commit,
            network_name: info.network_name,
            endpoints,
            nym_vpn_api_url,
        }
    }
}

fn offset_datetime_to_timestamp(datetime: time::OffsetDateTime) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: datetime.unix_timestamp(),
        nanos: datetime.nanosecond() as i32,
    }
}

fn validator_details_to_endpoints(
    validator_details: nym_vpn_lib::nym_config::defaults::ValidatorDetails,
) -> nym_vpn_proto::Endpoints {
    nym_vpn_proto::Endpoints {
        nyxd_url: Some(string_to_url(validator_details.nyxd_url)),
        websocket_url: validator_details.websocket_url.map(string_to_url),
        api_url: validator_details.api_url.map(string_to_url),
    }
}

fn string_to_url(url: String) -> nym_vpn_proto::Url {
    nym_vpn_proto::Url { url }
}
