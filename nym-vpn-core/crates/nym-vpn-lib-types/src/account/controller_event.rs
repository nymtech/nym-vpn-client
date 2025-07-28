// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use super::controller_state::AccountControllerState;

#[derive(Debug, Clone)]
pub enum AccountControllerEvent {
    NewState(AccountControllerState),
}

impl fmt::Display for AccountControllerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewState(new_state) => new_state.fmt(f),
        }
    }
}
