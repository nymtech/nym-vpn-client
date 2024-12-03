// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// The account controller is responsible for
// 1. checking if the account exists
// 2. register the device
// 3. request ticketbooks and top up the local credential store

pub mod shared_state;
pub mod util;

mod commander;
mod commands;
mod controller;
mod error;
mod storage;
mod ticketbooks;

pub use commander::AccountControllerCommander;
pub use commands::{AccountCommand, AccountCommandError};
pub use controller::AccountController;
pub use error::Error;
pub use shared_state::{AccountStateSummary, ReadyToConnect, SharedAccountState};
pub use ticketbooks::{AvailableTicketbook, AvailableTicketbooks};
