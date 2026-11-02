// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bip39::Mnemonic;
use nym_vpn_lib_types::AccountControllerState;

#[derive(Debug, Clone)]
/// Structure needed to pass the mnemonic inside A/C, but not leak it outside of it.
pub enum AccountControllerPairingEvent {
    Cancelled,
    TimedOut,
    Error(String),
    PinCode(String),
    Mnemonic(Mnemonic),
}

impl From<AccountControllerPairingEvent> for nym_vpn_lib_types::PairingEvent {
    fn from(event: AccountControllerPairingEvent) -> Self {
        match event {
            AccountControllerPairingEvent::Cancelled => Self::Cancelled,
            AccountControllerPairingEvent::TimedOut => Self::TimedOut,
            AccountControllerPairingEvent::Error(err) => Self::Error(err),
            AccountControllerPairingEvent::PinCode(pin) => Self::PinCode(pin),
            AccountControllerPairingEvent::Mnemonic(_) => Self::StoredMnemonic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountControllerResult {
    /// The session ID returned by the call that started the pairing process
    pub session_id: u32,
    pub event: AccountControllerPairingEvent,
}

impl From<AccountControllerResult> for nym_vpn_lib_types::PairingResult {
    fn from(result: AccountControllerResult) -> Self {
        Self {
            session_id: result.session_id,
            event: result.event.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AccountControllerEvent {
    NewState(AccountControllerState),
    Pairing(AccountControllerResult),
}
