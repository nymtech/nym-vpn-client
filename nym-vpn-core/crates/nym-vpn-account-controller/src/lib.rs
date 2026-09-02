// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// The account controller is responsible for
// 1. checking if the account exists
// 2. register the device

mod command_sender;
mod commands;
mod config;
mod controller;
mod deeplink;
mod error;
mod event_sender;
mod nyxd_client;
mod shared_state;
mod signer_discovery;
mod state_machine;
mod state_receiver;
mod storage;

pub(crate) use shared_state::SharedAccountState;

pub use command_sender::AccountCommandSender;
pub use config::AccountControllerConfig;
pub use controller::AccountController;
pub use deeplink::{CreateDeeplinkParams, Deeplink, DeeplinkError, DeeplinkMnemonic, Deeplinks};
pub use error::Error;
pub use event_sender::AccountControllerEventSender;
pub use nyxd_client::NyxdClient;
pub use signer_discovery::discover_ecash_signer_apis;
pub use state_receiver::AccountStateReceiver;
pub use storage::remove_files_for_account;
