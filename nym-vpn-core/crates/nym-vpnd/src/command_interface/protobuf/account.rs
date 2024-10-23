// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use maplit::hashmap;
use nym_vpn_account_controller::AccountStateSummary;
use nym_vpn_proto::account_error::AccountErrorType;

use crate::service::AccountError;

pub(crate) fn into_account_summary(state: AccountStateSummary) -> nym_vpn_proto::AccountSummary {
    nym_vpn_proto::AccountSummary {
        mnemonic: state.mnemonic.map(into_mnemonic).map(|m| m as i32),
        account: state.account.map(into_account).map(|m| m as i32),
        subscription: state.subscription.map(into_subscription).map(|m| m as i32),
        device: state.device.map(into_device).map(|m| m as i32),
        pending_zk_nym: state.pending_zk_nym,
    }
}

fn into_mnemonic(
    mnemonic: nym_vpn_account_controller::shared_state::MnemonicState,
) -> nym_vpn_proto::MnemonicState {
    match mnemonic {
        nym_vpn_account_controller::shared_state::MnemonicState::Stored => {
            nym_vpn_proto::MnemonicState::Stored
        }
        nym_vpn_account_controller::shared_state::MnemonicState::NotStored => {
            nym_vpn_proto::MnemonicState::NotStored
        }
    }
}

fn into_account(
    account: nym_vpn_account_controller::shared_state::AccountState,
) -> nym_vpn_proto::AccountState {
    match account {
        nym_vpn_account_controller::shared_state::AccountState::NotRegistered => {
            todo!()
        }
        nym_vpn_account_controller::shared_state::AccountState::Inactive => {
            nym_vpn_proto::AccountState::Inactive
        }
        nym_vpn_account_controller::shared_state::AccountState::Active => {
            nym_vpn_proto::AccountState::Active
        }
        nym_vpn_account_controller::shared_state::AccountState::DeleteMe => {
            todo!()
        }
    }
}

fn into_subscription(
    subscription: nym_vpn_account_controller::shared_state::SubscriptionState,
) -> nym_vpn_proto::SubscriptionState {
    match subscription {
        nym_vpn_account_controller::shared_state::SubscriptionState::NotSubscribed => todo!(),
        nym_vpn_account_controller::shared_state::SubscriptionState::Pending => todo!(),
        nym_vpn_account_controller::shared_state::SubscriptionState::Active => todo!(),
    }
}

fn into_device(
    device: nym_vpn_account_controller::shared_state::DeviceState,
) -> nym_vpn_proto::DeviceState {
    match device {
        nym_vpn_account_controller::shared_state::DeviceState::NotRegistered => todo!(),
        nym_vpn_account_controller::shared_state::DeviceState::Inactive => todo!(),
        nym_vpn_account_controller::shared_state::DeviceState::Active => todo!(),
        nym_vpn_account_controller::shared_state::DeviceState::DeleteMe => todo!(),
    }
}

impl From<AccountError> for nym_vpn_proto::AccountError {
    fn from(err: AccountError) -> Self {
        match err {
            AccountError::InvalidMnemonic { source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::InvalidMnemonic as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => source.to_string(),
                },
            },
            AccountError::FailedToStoreAccount { ref source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => source.to_string(),
                },
            },
            AccountError::FailedToCheckIfAccountIsStored { ref source } => {
                nym_vpn_proto::AccountError {
                    kind: AccountErrorType::Storage as i32,
                    message: err.to_string(),
                    details: hashmap! {
                        "reason".to_string() => source.to_string(),
                    },
                }
            }
            AccountError::FailedToRemoveAccount { ref source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => source.to_string(),
                },
            },
            AccountError::FailedToLoadAccount { ref source } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {
                    "reason".to_string() => source.to_string(),
                },
            },
            AccountError::MissingApiUrl => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::InvalidApiUrl => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::VpnApiClientError(_) => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::FailedToLoadKeys { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::FailedToGetAccountSummary { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::SendCommand { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
        }
    }
}
