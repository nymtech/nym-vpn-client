//! Best-effort detection of another active VPN tunnel on the host so we can
//! warn the user before they try to connect. Necessary because nym-vpnd's
//! firewall rules don't compose with a pre-existing default-route tunnel —
//! every outbound connection (including to gateways and to Nym's own CDN)
//! gets RST'd as the daemon's allow-rules are scoped to the physical
//! interface that's no longer the egress.
//!
//! Read-only: enumerates network interfaces and the default route. Never
//! mutates state.

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts", rename = "TActiveVpn")]
pub struct ActiveVpn {
    pub interface: String,
    pub kind: VpnKind,
    pub is_default_route: bool,
}

#[derive(Debug, Clone, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts", rename = "TVpnKind")]
pub enum VpnKind {
    Wireguard,
    Tun,
    Mullvad,
    NordLynx,
    Proton,
}

pub fn detect_active_vpns() -> Vec<ActiveVpn> {
    #[cfg(target_os = "linux")]
    {
        linux::detect()
    }
    #[cfg(target_os = "windows")]
    {
        windows::detect()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{ActiveVpn, VpnKind};
    use std::fs;

    // Interface names that belong to nym-vpnd itself. If the daemon is mid- or
    // post-connect, these would otherwise be reported as "another VPN". The
    // canonical names depend on the wireguard tunnel options the daemon was
    // configured with; these prefixes cover the common ones.
    const OWN_PREFIXES: &[&str] = &["nymtun", "nymvpn", "nymwg"];

    const NON_VPN_PREFIXES: &[&str] = &[
        "lo",       // loopback
        "docker",   // docker bridges
        "br-",      // docker custom bridges
        "veth",     // container veth pairs
        "virbr",    // libvirt bridges
        "vmnet",    // VMware
        "vboxnet",  // VirtualBox
        "wlx",      // some wifi naming on systemd-networkd
    ];

    pub fn detect() -> Vec<ActiveVpn> {
        let default_iface = default_route_iface();
        let Ok(entries) = fs::read_dir("/sys/class/net") else {
            return Vec::new();
        };

        entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if is_skipped(&name) {
                    return None;
                }
                let kind = classify(&name)?;
                Some(ActiveVpn {
                    is_default_route: default_iface.as_deref() == Some(&name),
                    interface: name,
                    kind,
                })
            })
            .collect()
    }

    fn is_skipped(name: &str) -> bool {
        OWN_PREFIXES.iter().any(|p| name.starts_with(p))
            || NON_VPN_PREFIXES.iter().any(|p| name == *p || name.starts_with(p))
    }

    /// Parse `/proc/net/route` for the IPv4 default route's output interface.
    /// The file is well-defined: the first column is the iface name, the
    /// second column is the destination in hex big-endian; "00000000" means
    /// the default route.
    fn default_route_iface() -> Option<String> {
        let routes = fs::read_to_string("/proc/net/route").ok()?;
        for line in routes.lines().skip(1) {
            let mut parts = line.split_whitespace();
            let iface = parts.next()?;
            let dest = parts.next()?;
            if dest == "00000000" {
                return Some(iface.to_string());
            }
        }
        None
    }

    /// Classify an interface as a VPN. Prefer the kernel-reported link kind
    /// from `/sys/class/net/<iface>/uevent`; fall back to name patterns for
    /// userspace tun devices (OpenVPN, WireGuard userspace, branded clients).
    fn classify(name: &str) -> Option<VpnKind> {
        if let Ok(uevent) = fs::read_to_string(format!("/sys/class/net/{name}/uevent"))
            && uevent.contains("DEVTYPE=wireguard")
        {
            return Some(VpnKind::Wireguard);
        }

        // Branded clients first (more specific than generic wg/tun).
        match name {
            n if n.starts_with("nordlynx") => Some(VpnKind::NordLynx),
            n if n.starts_with("mullvad") => Some(VpnKind::Mullvad),
            n if n.starts_with("proton") => Some(VpnKind::Proton),
            n if n.starts_with("wg") => Some(VpnKind::Wireguard),
            n if n.starts_with("tun") || n.starts_with("tap") => Some(VpnKind::Tun),
            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn skip_loopback_and_bridges() {
            for n in [
                "lo", "docker0", "br-abc123", "veth0", "virbr0", "vmnet1", "vboxnet0",
            ] {
                assert!(is_skipped(n), "{n} should be skipped");
            }
        }

        #[test]
        fn skip_own_tunnel() {
            for n in ["nymtun0", "nymtun1", "nymvpn0", "nymvpn", "nymwg0"] {
                assert!(is_skipped(n), "{n} should be skipped (own tunnel)");
            }
        }

        #[test]
        fn classify_known_names() {
            assert_eq!(classify("wg0"), Some(VpnKind::Wireguard));
            assert_eq!(classify("wg42"), Some(VpnKind::Wireguard));
            assert_eq!(classify("nordlynx"), Some(VpnKind::NordLynx));
            assert_eq!(classify("mullvad-tun"), Some(VpnKind::Mullvad));
            assert_eq!(classify("proton0"), Some(VpnKind::Proton));
            assert_eq!(classify("tun0"), Some(VpnKind::Tun));
            assert_eq!(classify("tap0"), Some(VpnKind::Tun));
            assert_eq!(classify("eth0"), None);
            assert_eq!(classify("wlan0"), None);
            assert_eq!(classify("enp0s3"), None);
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::ActiveVpn;

    // TODO(windows): use `GetIpForwardTable2` + `GetAdaptersAddresses` to
    // find the default route's interface and inspect its `IfType` /
    // description for known VPN clients (WireGuard service, OpenVPN TAP,
    // Mullvad, NordLynx, Proton). The `windows` crate is already in deps.
    // Returning empty for now keeps the call safe.
    pub fn detect() -> Vec<ActiveVpn> {
        Vec::new()
    }
}
