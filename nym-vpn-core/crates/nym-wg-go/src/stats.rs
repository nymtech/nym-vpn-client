// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Result, wireguard_go::TunnelStats};

pub trait StatsReader {
    fn get_stats(&self) -> Result<TunnelStats>;
}
