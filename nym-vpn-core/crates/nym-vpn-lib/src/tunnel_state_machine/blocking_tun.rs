// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shared blocking / placeholder TUN settings used on iOS and Android while the real tunnel is down.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::IpNetwork;

use crate::tunnel_provider::TunnelSettings;

/// Placeholder interface addresses used while the real tunnel is down (Error / reconnect gap).
pub const BLOCKING_INTERFACE_ADDRS: [IpAddr; 2] = [
    IpAddr::V4(Ipv4Addr::new(169, 254, 0, 10)),
    IpAddr::V6(Ipv6Addr::new(
        0xfdcc, 0x9fc0, 0xe75a, 0x53c3, 0xfa25, 0x241f, 0x21c0, 0x70d0,
    )),
];

/// Minimum IPv6 MTU; used for the blocking placeholder interface.
pub const BLOCKING_TUN_MTU: u16 = 1280;

/// Tunnel settings that keep a VPN interface up without forwarding traffic (blackhole).
pub fn blocking_tunnel_settings(dns_server: IpAddr) -> TunnelSettings {
    TunnelSettings {
        remote_addresses: vec![],
        interface_addresses: BLOCKING_INTERFACE_ADDRS.map(IpNetwork::from).to_vec(),
        dns_servers: vec![dns_server],
        mtu: BLOCKING_TUN_MTU,
    }
}

/// Default DNS for Android blocking TUN (no local filtering resolver on Android).
#[cfg(any(target_os = "android", test))]
pub fn android_blocking_dns() -> IpAddr {
    BLOCKING_INTERFACE_ADDRS[0]
}

/// Install blocking cover before releasing the previous TUN so reconnect cannot open an ISP window.
#[cfg(any(target_os = "android", test))]
pub fn with_blocking_before_tun_release<E>(
    install_blocking: impl FnOnce() -> Result<(), E>,
    release_previous_tun: impl FnOnce(),
) -> Result<(), E> {
    install_blocking()?;
    release_previous_tun();
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn blocking_tunnel_settings_uses_shared_addrs_and_mtu() {
        let dns = IpAddr::V4(Ipv4Addr::new(169, 254, 0, 10));
        let settings = blocking_tunnel_settings(dns);
        assert_eq!(settings.mtu, BLOCKING_TUN_MTU);
        assert_eq!(settings.dns_servers, vec![dns]);
        assert!(settings.remote_addresses.is_empty());
        assert_eq!(
            settings.interface_addresses,
            BLOCKING_INTERFACE_ADDRS.map(IpNetwork::from).to_vec()
        );
    }

    #[test]
    fn android_blocking_dns_is_link_local_v4() {
        assert_eq!(
            android_blocking_dns(),
            IpAddr::V4(Ipv4Addr::new(169, 254, 0, 10))
        );
    }

    #[test]
    fn with_blocking_before_tun_release_installs_before_drop() {
        let steps = RefCell::new(Vec::new());
        with_blocking_before_tun_release::<()>(
            || {
                steps.borrow_mut().push("install");
                Ok(())
            },
            || steps.borrow_mut().push("drop"),
        )
        .expect("install ok");
        assert_eq!(steps.into_inner(), ["install", "drop"]);
    }

    #[test]
    fn with_blocking_before_tun_release_skips_drop_on_install_error() {
        let steps = RefCell::new(Vec::new());
        let err = with_blocking_before_tun_release(
            || {
                steps.borrow_mut().push("install");
                Err("fail")
            },
            || steps.borrow_mut().push("drop"),
        );
        assert_eq!(err, Err("fail"));
        assert_eq!(steps.into_inner(), ["install"]);
    }
}
