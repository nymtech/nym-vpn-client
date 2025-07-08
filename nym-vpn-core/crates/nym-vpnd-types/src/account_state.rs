// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_vpn_account_controller::shared_state::{
    AccountRegistered, AccountState, AccountSummary, DeviceState, DeviceSummary, FairUsage,
    RegisterDeviceResult, SubscriptionState,
};
pub use nym_vpn_lib_types::{RequestZkNymErrorReason, RequestZkNymSuccess};

/// Public representation of `nym_vpn_account_controller::shared_state::MnemonicState`
#[derive(Debug)]
pub enum MnemonicState {
    Stored,
    NotStored,
}

/// Public representation of `nym_vpn_account_controller::shared_state::RequestZkNymResult`
#[derive(Debug)]
pub enum RequestZkNymResult {
    InProgress,
    Done {
        successes: Vec<RequestZkNymSuccess>,
        failures: Vec<RequestZkNymErrorReason>,
    },
    Error(RequestZkNymErrorReason),
}

#[derive(Debug)]
pub struct AccountStateSummary {
    pub mnemonic: Option<MnemonicState>,
    pub account_registered: Option<AccountRegistered>,
    pub account_summary: Option<AccountSummary>,
    pub device: Option<DeviceState>,
    pub register_device_result: Option<RegisterDeviceResult>,
    pub request_zk_nym_result: Option<RequestZkNymResult>,
}

impl From<nym_vpn_account_controller::shared_state::AccountStateSummary> for AccountStateSummary {
    fn from(value: nym_vpn_account_controller::shared_state::AccountStateSummary) -> Self {
        Self {
            mnemonic: value.mnemonic.map(|state| match state {
                nym_vpn_account_controller::shared_state::MnemonicState::Stored { .. } => {
                    MnemonicState::Stored
                }
                nym_vpn_account_controller::shared_state::MnemonicState::NotStored => {
                    MnemonicState::NotStored
                }
            }),
            account_registered: value.account_registered,
            account_summary: value.account_summary,
            device: value.device,
            register_device_result: value.register_device_result,
            request_zk_nym_result: value.request_zk_nym_result.map(RequestZkNymResult::from),
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::RequestZkNymResult> for RequestZkNymResult {
    fn from(value: nym_vpn_account_controller::shared_state::RequestZkNymResult) -> Self {
        match value {
            nym_vpn_account_controller::shared_state::RequestZkNymResult::InProgress => {
                RequestZkNymResult::InProgress
            }
            nym_vpn_account_controller::shared_state::RequestZkNymResult::Done {
                successes,
                failures,
            } => RequestZkNymResult::Done {
                successes,
                failures: failures
                    .into_iter()
                    .map(RequestZkNymErrorReason::from)
                    .collect(),
            },
            nym_vpn_account_controller::shared_state::RequestZkNymResult::Error(error) => {
                let error = RequestZkNymErrorReason::from(error);
                RequestZkNymResult::Error(error)
            }
        }
    }
}
