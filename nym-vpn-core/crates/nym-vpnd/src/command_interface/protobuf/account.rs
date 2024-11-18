// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use maplit::hashmap;
use nym_vpn_account_controller::AccountStateSummary;
use nym_vpn_proto::account_error::AccountErrorType;

use crate::service::AccountError;

pub(crate) fn into_account_summary(state: AccountStateSummary) -> nym_vpn_proto::AccountSummary {
    nym_vpn_proto::AccountSummary {
        mnemonic: state.mnemonic.map(into_mnemonic).map(|m| m as i32),
        account_registered: state
            .account_registered
            .map(into_account_registered)
            .map(|m| m as i32),
        account_summary: state.account_summary.map(into_account_summary_inner),
        device: state.device.map(into_device).map(|m| m as i32),
        device_registration: state.device_registration.map(into_device_registration),
        pending_zk_nym: state.pending_zk_nym,
    }
}

fn into_mnemonic(
    mnemonic: nym_vpn_account_controller::shared_state::MnemonicState,
) -> nym_vpn_proto::MnemonicState {
    match mnemonic {
        nym_vpn_account_controller::shared_state::MnemonicState::Stored { .. } => {
            nym_vpn_proto::MnemonicState::Stored
        }
        nym_vpn_account_controller::shared_state::MnemonicState::NotStored => {
            nym_vpn_proto::MnemonicState::NotStored
        }
    }
}

fn into_account_registered(
    account_registered: nym_vpn_account_controller::shared_state::AccountRegistered,
) -> nym_vpn_proto::AccountRegistered {
    match account_registered {
        nym_vpn_account_controller::shared_state::AccountRegistered::Registered => {
            nym_vpn_proto::AccountRegistered::AccountRegistered
        }
        nym_vpn_account_controller::shared_state::AccountRegistered::NotRegistered => {
            nym_vpn_proto::AccountRegistered::AccountNotRegistered
        }
    }
}

fn into_account(
    account: nym_vpn_account_controller::shared_state::AccountState,
) -> nym_vpn_proto::AccountState {
    match account {
        nym_vpn_account_controller::shared_state::AccountState::Inactive => {
            nym_vpn_proto::AccountState::Inactive
        }
        nym_vpn_account_controller::shared_state::AccountState::Active => {
            nym_vpn_proto::AccountState::Active
        }
        nym_vpn_account_controller::shared_state::AccountState::DeleteMe => {
            nym_vpn_proto::AccountState::DeleteMe
        }
    }
}

fn into_subscription(
    subscription: nym_vpn_account_controller::shared_state::SubscriptionState,
) -> nym_vpn_proto::SubscriptionState {
    match subscription {
        nym_vpn_account_controller::shared_state::SubscriptionState::NotActive => {
            nym_vpn_proto::SubscriptionState::NotRegistered
        }
        nym_vpn_account_controller::shared_state::SubscriptionState::Pending => {
            nym_vpn_proto::SubscriptionState::Pending
        }
        nym_vpn_account_controller::shared_state::SubscriptionState::Active => {
            nym_vpn_proto::SubscriptionState::Active
        }
        nym_vpn_account_controller::shared_state::SubscriptionState::Complete => {
            nym_vpn_proto::SubscriptionState::Complete
        }
    }
}

fn into_device_summary(
    device_summary: nym_vpn_account_controller::shared_state::DeviceSummary,
) -> nym_vpn_proto::DeviceSummary {
    nym_vpn_proto::DeviceSummary {
        active: device_summary.active,
        max: device_summary.max,
        remaining: device_summary.remaining,
    }
}

fn into_account_summary_inner(
    account_summary: nym_vpn_account_controller::shared_state::AccountSummary,
) -> nym_vpn_proto::AccountSummaryInner {
    nym_vpn_proto::AccountSummaryInner {
        account: into_account(account_summary.account) as i32,
        subscription: into_subscription(account_summary.subscription) as i32,
        device_summary: Some(into_device_summary(account_summary.device_summary)),
    }
}

fn into_device(
    device: nym_vpn_account_controller::shared_state::DeviceState,
) -> nym_vpn_proto::DeviceState {
    match device {
        nym_vpn_account_controller::shared_state::DeviceState::NotRegistered => {
            nym_vpn_proto::DeviceState::NotRegistered
        }
        nym_vpn_account_controller::shared_state::DeviceState::Inactive => {
            nym_vpn_proto::DeviceState::Inactive
        }
        nym_vpn_account_controller::shared_state::DeviceState::Active => {
            nym_vpn_proto::DeviceState::Active
        }
        nym_vpn_account_controller::shared_state::DeviceState::DeleteMe => {
            nym_vpn_proto::DeviceState::DeleteMe
        }
    }
}

fn into_device_registration(
    device_registration: nym_vpn_account_controller::shared_state::DeviceRegistration,
) -> nym_vpn_proto::DeviceRegistration {
    let mut returned_message = None;
    let mut returned_message_id = None;
    let kind = match device_registration {
        nym_vpn_account_controller::shared_state::DeviceRegistration::InProgress => {
            nym_vpn_proto::device_registration::DeviceRegistrationType::DeviceRegistrationInProgress
        }
        nym_vpn_account_controller::shared_state::DeviceRegistration::Success => {
            nym_vpn_proto::device_registration::DeviceRegistrationType::DeviceRegistrationSuccess
        }
        nym_vpn_account_controller::shared_state::DeviceRegistration::Failed {
            message,
            message_id,
            code_reference_id: _,
        } => {
            returned_message = Some(message);
            returned_message_id = message_id;
            nym_vpn_proto::device_registration::DeviceRegistrationType::DeviceRegistrationFailed
        }
    } as i32;

    nym_vpn_proto::DeviceRegistration {
        kind,
        message: returned_message,
        message_id: returned_message_id,
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
            AccountError::FailedToForgetAccount { ref source } => nym_vpn_proto::AccountError {
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
            AccountError::FailedToInitDeviceKeys { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::FailedToResetDeviceKeys { .. } => nym_vpn_proto::AccountError {
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
            AccountError::RecvCommand { .. } => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::NoAccountStored => nym_vpn_proto::AccountError {
                kind: AccountErrorType::Storage as i32,
                message: err.to_string(),
                details: hashmap! {},
            },
            AccountError::AccountControllerError { .. } => nym_vpn_proto::AccountError {
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
        }
    }
}
