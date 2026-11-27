// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::BridgeTransport;

use crate::tunnel_state_machine::tunnel::transports::ClientConfig;

/// Narrows candidates to `preference`'s transport; `None` leaves both lists untouched.
pub(super) fn filter_by_preference(
    quic: Vec<ClientConfig>,
    fallback: Vec<ClientConfig>,
    preference: Option<BridgeTransport>,
) -> (Vec<ClientConfig>, Vec<ClientConfig>) {
    match preference {
        None => (quic, fallback),
        Some(BridgeTransport::Quic) => (quic, Vec::new()),
        Some(BridgeTransport::Tls) => (
            Vec::new(),
            fallback
                .into_iter()
                .filter(|c| matches!(c, ClientConfig::TlsPlain(_)))
                .collect(),
        ),
        Some(BridgeTransport::Ssh) => (
            Vec::new(),
            fallback
                .into_iter()
                .filter(|c| matches!(c, ClientConfig::SshPlain(_)))
                .collect(),
        ),
    }
}
