// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct QrUrl {
    pub url: String,
    /// Identifies the started pairing process in its `Event::Pairing` events
    pub session_id: u32,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct QrPairingConfirmation {
    pub pin_code: String,
    /// Identifies the started pairing process in its `Event::Pairing` events
    pub session_id: u32,
}
