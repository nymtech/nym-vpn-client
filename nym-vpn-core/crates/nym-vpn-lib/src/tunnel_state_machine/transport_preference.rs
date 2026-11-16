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

#[cfg(test)]
mod test {
    use super::*;

    fn quic() -> ClientConfig {
        ClientConfig::QuicPlain(nym_bridges::types::quic::ClientOptions {
            addresses: vec![],
            host: None,
            id_pubkey: String::new(),
        })
    }

    fn tls() -> ClientConfig {
        ClientConfig::TlsPlain(nym_bridges::types::tls::ClientOptions {
            addresses: vec![],
            host: None,
            id_pubkey: String::new(),
        })
    }

    fn ssh() -> ClientConfig {
        ClientConfig::SshPlain(nym_bridges::types::ssh::ClientOptions {
            addresses: vec![],
            id_pubkey: String::new(),
            username: None,
            client_auth_key: String::new(),
            client_banner: None,
        })
    }

    #[test]
    fn filters_by_preference() {
        let input = || (vec![quic()], vec![tls(), ssh()]);

        let (q, f) = filter_by_preference(input().0, input().1, None);
        assert_eq!((q.len(), f.len()), (1, 2));

        let (q, f) = filter_by_preference(input().0, input().1, Some(BridgeTransport::Quic));
        assert_eq!((q.len(), f.len()), (1, 0));

        let (q, f) = filter_by_preference(input().0, input().1, Some(BridgeTransport::Tls));
        assert!(q.is_empty() && matches!(f.as_slice(), [ClientConfig::TlsPlain(_)]));

        let (q, f) = filter_by_preference(input().0, input().1, Some(BridgeTransport::Ssh));
        assert!(q.is_empty() && matches!(f.as_slice(), [ClientConfig::SshPlain(_)]));
    }
}
