// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, Copy)]
pub(crate) struct AvailableBandwidth {
    pub(crate) bandwidth_bytes: i64,
    pub(crate) upgrade_mode: Option<bool>,
}
