// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_vpn_account_controller::shared_state::{
    AccountRegistered, AccountState, AccountSummary, DeviceState, DeviceSummary, FairUsage,
    RegisterDeviceResult, RequestZkNymResult, SubscriptionState,
};

/// Public representation of `nym_vpn_account_controller::shared_state::MnemonicState`
#[derive(Debug)]
pub enum MnemonicState {
    Stored,
    NotStored,
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
            request_zk_nym_result: value.request_zk_nym_result,
        }
    }
}
