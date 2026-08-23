/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import "net/netip"

// OwnerUidFunc resolves the UID owning a connection, or -1 (INVALID_UID)
// when the owner cannot be determined.
type OwnerUidFunc func(proto Proto, src, dst netip.AddrPort) int32

// lanPrefixes are the local-network ranges that "allow local network access"
// (LAN bypass) exempts from the tunnel. They mirror ALLOWED_LAN_NETS and
// ALLOWED_LAN_MULTICAST_NETS in the Rust `nym-firewall-config` crate — keep the
// two lists in sync.
var lanPrefixes = []netip.Prefix{
	netip.MustParsePrefix("10.0.0.0/8"),
	netip.MustParsePrefix("172.16.0.0/12"),
	netip.MustParsePrefix("192.168.0.0/16"),
	netip.MustParsePrefix("169.254.0.0/16"),
	netip.MustParsePrefix("fe80::/10"),
	netip.MustParsePrefix("fc00::/7"),
	// Multicast / broadcast.
	netip.MustParsePrefix("255.255.255.255/32"),
	netip.MustParsePrefix("224.0.0.0/24"),
	netip.MustParsePrefix("239.0.0.0/8"),
	netip.MustParsePrefix("ff01::/16"),
	netip.MustParsePrefix("ff02::/16"),
	netip.MustParsePrefix("ff03::/16"),
	netip.MustParsePrefix("ff04::/16"),
	netip.MustParsePrefix("ff05::/16"),
}

// isLanAddr reports whether addr falls in one of the local-network ranges.
func isLanAddr(addr netip.Addr) bool {
	addr = addr.Unmap()
	for _, p := range lanPrefixes {
		if p.Contains(addr) {
			return true
		}
	}
	return false
}

// Classifier decides whether a new flow is routed through the tunnel or
// bypassed directly. Two independent bypass criteria: destination in a local
// network (LAN bypass) and ownership by an excluded app UID. Unattributable
// flows always go through the tunnel.
type Classifier struct {
	excludedUIDs map[uint32]struct{}
	ownerUID     OwnerUidFunc
	bypassLan    bool
}

func NewClassifier(excludedUIDs []uint32, ownerUID OwnerUidFunc, bypassLan bool) *Classifier {
	m := make(map[uint32]struct{}, len(excludedUIDs))
	for _, uid := range excludedUIDs {
		m[uid] = struct{}{}
	}
	return &Classifier{excludedUIDs: m, ownerUID: ownerUID, bypassLan: bypassLan}
}

// IsLanDestBypass reports whether the flow must be bypassed because it targets
// the local network and LAN bypass is enabled. This is a pure function of the
// destination address (no UID attribution) and takes precedence over UID-based
// classification and the DNS-to-tunnel fallback: local-network traffic must go
// direct so it survives Android's "block connections without VPN" kill switch,
// which otherwise blocks the route-based LAN exemption.
func (c *Classifier) IsLanDestBypass(key FlowKey) bool {
	return c.bypassLan && isLanAddr(key.Dst.Addr())
}

// Decide classifies a flow by owning UID (LAN-destination bypass is handled
// separately, ahead of this, by IsLanDestBypass).
func (c *Classifier) Decide(key FlowKey) Decision {
	if c.ownerUID == nil {
		return DecisionTunnel
	}
	uid := c.ownerUID(key.Proto, key.Src, key.Dst)
	if uid < 0 {
		return DecisionTunnel
	}
	if _, excluded := c.excludedUIDs[uint32(uid)]; excluded {
		return DecisionBypass
	}
	return DecisionTunnel
}
