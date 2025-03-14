// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Interface with low-level Windows-specific bits.

#![deny(missing_docs)]
#![cfg(windows)]

/// I/O
pub mod io;

/// Networking
pub mod net;

/// Synchronization
pub mod sync;

/// Processes
pub mod process;

/// Window event handling
pub mod window;

/// Security and identity
pub mod security;
