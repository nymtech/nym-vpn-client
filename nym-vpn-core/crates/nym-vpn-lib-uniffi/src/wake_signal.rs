// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

/// Call from the platform's own sleep/wake callback (`PacketTunnelProvider.wake()` on Apple) when
/// the device has just woken up. Does not go through the tunnel state machine or any command
/// queue - see `nym_vpn_lib::wake_signal` for what reacts to it.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn notifyDeviceWoke() {
    nym_vpn_lib::wake_signal::notify_woke();
}
