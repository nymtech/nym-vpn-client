// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// The account controller is responsible for
// 1. checking if the account exists
// 2. register the device
// 3. request ticketbooks and top up the local credential store

mod controller;
mod error;
mod shared_state;
mod ecash_client;

pub use controller::{AccountCommand, AccountController};
pub use shared_state::SharedAccountState;
