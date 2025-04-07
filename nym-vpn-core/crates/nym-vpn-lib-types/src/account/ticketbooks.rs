// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone, Debug)]
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
