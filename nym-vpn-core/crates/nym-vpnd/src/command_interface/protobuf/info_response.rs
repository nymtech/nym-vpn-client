// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::VpnServiceInfoResult;
use nym_vpn_proto::InfoResponse;

impl From<VpnServiceInfoResult> for InfoResponse {
    fn from(info: VpnServiceInfoResult) -> Self {
        let build_timestamp = info.build_timestamp.map(|ts| prost_types::Timestamp {
            seconds: ts.unix_timestamp(),
            nanos: ts.nanosecond() as i32,
        });
        InfoResponse {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            git_commit: info.git_commit,
        }
    }
}
