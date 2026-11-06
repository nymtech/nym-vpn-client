// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use nym_vpn_lib_types::AccountControllerState;

#[derive(Debug, Clone)]
/// Structure needed to pass the mnemonic inside A/C, but not leak it outside of it.
pub enum AccountControllerPairingResult {
    Cancelled,
    TimedOut,
    Error(String),
    PinCode(String),
    Mnemonic(Mnemonic),
}

impl From<AccountControllerPairingResult> for nym_vpn_lib_types::PairingResult {
    fn from(result: AccountControllerPairingResult) -> Self {
        match result {
            AccountControllerPairingResult::Cancelled => Self::Cancelled,
            AccountControllerPairingResult::TimedOut => Self::TimedOut,
            AccountControllerPairingResult::Error(err) => Self::Error(err),
            AccountControllerPairingResult::PinCode(pin) => Self::PinCode(pin),
            AccountControllerPairingResult::Mnemonic(_) => Self::StoredMnemonic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountControllerPairingEvent {
    /// The session ID returned by the call that started the pairing process
    pub session_id: u32,
    pub result: AccountControllerPairingResult,
}

impl From<AccountControllerPairingEvent> for nym_vpn_lib_types::PairingEvent {
    fn from(event: AccountControllerPairingEvent) -> Self {
        Self {
            session_id: event.session_id,
            result: event.result.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AccountControllerEvent {
    NewState(AccountControllerState),
    Pairing(AccountControllerPairingEvent),
}
