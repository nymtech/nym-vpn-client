// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{RequestZkNymError, RequestZkNymErrorReason, RequestZkNymSuccess};

use crate::{
    AccountIdentity as ProtoAccountIdentity, RequestZkNymError as ProtoRequestZkNymError,
    RequestZkNymSuccess as ProtoRequestZkNymSuccess,
};

impl From<Option<String>> for ProtoAccountIdentity {
    fn from(identity: Option<String>) -> Self {
        Self {
            account_identity: identity,
        }
    }
}

impl From<RequestZkNymSuccess> for ProtoRequestZkNymSuccess {
    fn from(value: RequestZkNymSuccess) -> Self {
        Self { id: value.id }
    }
}

impl From<RequestZkNymErrorReason> for ProtoRequestZkNymError {
    fn from(error: RequestZkNymErrorReason) -> Self {
        let outcome = match error {
            RequestZkNymErrorReason::NoAccountStored => {
                Some(crate::request_zk_nym_error::Outcome::NoAccountStored(true))
            }
            RequestZkNymErrorReason::NoDeviceStored => {
                Some(crate::request_zk_nym_error::Outcome::NoDeviceStored(true))
            }
            RequestZkNymErrorReason::VpnApi(vpn_api_endpoint_failure) => Some(
                crate::request_zk_nym_error::Outcome::VpnApi(vpn_api_endpoint_failure.into()),
            ),
            RequestZkNymErrorReason::UnexpectedVpnApiResponse(err) => {
                Some(crate::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(err))
            }
            RequestZkNymErrorReason::Storage(err) => {
                Some(crate::request_zk_nym_error::Outcome::Storage(err))
            }
            RequestZkNymErrorReason::Internal(err) => {
                Some(crate::request_zk_nym_error::Outcome::Internal(err))
            }
        };
        Self { outcome }
    }
}

impl From<RequestZkNymError> for ProtoRequestZkNymError {
    fn from(error: RequestZkNymError) -> Self {
        RequestZkNymErrorReason::from(error).into()
    }
}
