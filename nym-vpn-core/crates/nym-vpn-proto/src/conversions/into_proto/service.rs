// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{VpnServiceConnectError, VpnServiceInfo};

use crate::{
    ConnectRequestError as ProtoConnectRequestError, InfoResponse as ProtoInfoResponse,
    NymNetworkDetails as ProtoNymNetworkDetails, NymVpnNetworkDetails as ProtoNymVpnNetworkDetails,
};

impl From<VpnServiceInfo> for ProtoInfoResponse {
    fn from(info: VpnServiceInfo) -> Self {
        let build_timestamp = info
            .build_timestamp
            .map(crate::conversions::prost::offset_datetime_into_proto_timestamp);

        let nym_network = Some(ProtoNymNetworkDetails::from(info.nym_network.clone()));
        let nym_vpn_network = Some(ProtoNymVpnNetworkDetails::from(info.nym_vpn_network));

        Self {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            platform: info.platform,
            git_commit: info.git_commit,
            nym_network,
            nym_vpn_network,
        }
    }
}

impl From<VpnServiceConnectError> for ProtoConnectRequestError {
    fn from(err: VpnServiceConnectError) -> Self {
        match err {
            VpnServiceConnectError::Internal(ref _account_error) => ProtoConnectRequestError {
                kind: crate::connect_request_error::ConnectRequestErrorType::Internal as i32,
                message: err.to_string(),
            },
            VpnServiceConnectError::Cancel => ProtoConnectRequestError {
                kind: crate::connect_request_error::ConnectRequestErrorType::Internal as i32,
                message: err.to_string(),
            },
        }
    }
}
