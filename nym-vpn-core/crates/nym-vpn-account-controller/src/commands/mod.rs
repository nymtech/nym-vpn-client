// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod common_handler;
pub(crate) mod handler;

pub(crate) mod decentralised_zknym_handler;
mod dispatch;

pub(crate) use dispatch::{AccountCommand, CommonCommand, ReturnSender};
