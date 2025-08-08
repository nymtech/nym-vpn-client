// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Interface with macOS-specific bits.

#![cfg(target_os = "macos")]

pub mod net;
pub mod process;
