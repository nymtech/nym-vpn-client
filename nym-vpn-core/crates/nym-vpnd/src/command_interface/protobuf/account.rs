// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use maplit::hashmap;

use crate::service::AccountError;

// Most of these are just mapped to AccountErrorType::Storage for now. Ideally we start to further
// differentiate them in the future.

impl From<AccountError> for nym_vpn_proto::AccountError {
    fn from(err: AccountError) -> Self {
        use nym_vpn_proto::account_error::AccountErrorType;
        match err {
            AccountError::InvalidMnemonic { source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::InvalidMnemonic as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => source.to_string(),
                },
            },
            AccountError::FailedToResetDeviceKeys { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::AccountController { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::AccountCommand { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::AccountManagementNotConfigured => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::FailedToParseAccountLinks => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::IsConnected => nym_vpn_proto::AccountError {
                kind: AccountErrorType::IsConnected as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
        }
    }
}
