// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// The account controller is responsible for
// 1. checking if the account exists
// 2. register the device
// 3. request ticketbooks and top up the local credential store

pub mod shared_state;

mod command_sender;
mod commands;
mod config;
mod connectivity;
mod controller;
mod error;
mod storage;
mod ticketbooks;
mod vpn_api_client;

pub use command_sender::AccountCommandSender;
pub use config::AccountControllerConfig;
pub use controller::AccountController;
pub use error::Error;
pub use shared_state::{AccountStateSummary, SharedAccountState};
pub use storage::remove_files_for_account;
pub use ticketbooks::{AvailableTicketbook, AvailableTicketbooks};
