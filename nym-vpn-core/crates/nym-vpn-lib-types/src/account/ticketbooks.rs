// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

// todo: this type is not used anywhere in the codebase for some reason
#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct AvailableTickets {
    pub mixnet_entry_tickets: u64,
    pub mixnet_entry_data: u64,
    pub mixnet_entry_data_si: String,

    pub mixnet_exit_tickets: u64,
    pub mixnet_exit_data: u64,
    pub mixnet_exit_data_si: String,

    pub vpn_entry_tickets: u64,
    pub vpn_entry_data: u64,
    pub vpn_entry_data_si: String,

    pub vpn_exit_tickets: u64,
    pub vpn_exit_data: u64,
    pub vpn_exit_data_si: String,
}
