// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
#[cfg(target_os = "linux")]
pub const MARK: i32 = 0xf42;
