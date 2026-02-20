// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
pub mod session;
mod sys;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod tun;
