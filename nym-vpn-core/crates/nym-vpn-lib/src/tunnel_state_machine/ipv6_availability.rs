// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(windows)]
pub async fn is_ipv6_enabled_in_os() -> bool {
    use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

    const IPV6_DISABLED_ON_TUNNELS_MASK: u32 = 0x01;

    // Check registry if IPv6 is disabled on tunnel interfaces, as documented in
    // https://support.microsoft.com/en-us/help/929852/guidance-for-configuring-ipv6-in-windows-for-advanced-users
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters")
        .and_then(|ipv6_config| ipv6_config.get_value("DisabledComponents"))
        .map(|ipv6_disabled_bits: u32| (ipv6_disabled_bits & IPV6_DISABLED_ON_TUNNELS_MASK) == 0)
        .unwrap_or(true)
}

#[cfg(not(windows))]
pub async fn is_ipv6_enabled_in_os() -> bool {
    #[cfg(target_os = "linux")]
    {
        // - Use `/proc/sys/net/ipv6/conf/default` instead of `/proc/sys/net/ipv6/conf/default/all`,
        //   because when configuring the kernel with `ipv6.disable_ipv6=1`, `all` would be `0`, however `default` would be `1`.
        //   In such configuration manipulating routing table is not possible anyway due to permission error.
        //   See: https://wiki.archlinux.org/title/IPv6 (paragraph 10.1)
        //
        // - If kernel is configured with `ipv6.disable=1` then `/proc/sys/net/ipv6/*` does not even exist.
        //
        // - When setting `net.ipv6.conf.all.disable_ipv6=1` at runtime, `net.ipv6.conf.default`
        //   is automatically updated to `1` too.
        tokio::fs::read_to_string("/proc/sys/net/ipv6/conf/default/disable_ipv6")
            .await
            .map(|disable_ipv6| disable_ipv6.trim() == "0")
            .unwrap_or(false)
    }
    #[cfg(any(target_os = "macos", target_os = "android", target_os = "ios"))]
    {
        true
    }
}
