// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// The account controller is responsible for
// 1. checking if the account exists
// 2. register the device
// 3. request ticketbooks and top up the local credential store

mod controller;
mod ecash_client;
mod error;
pub mod shared_state;

pub use controller::{AccountCommand, AccountController};
pub use shared_state::{AccountStateSummary, ReadyToConnect, SharedAccountState};
