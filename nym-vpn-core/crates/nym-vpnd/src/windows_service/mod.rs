// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod install;
mod service;

pub(crate) use service::{start, SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};
