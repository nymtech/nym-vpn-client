/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import "net/netip"

// OwnerUidFunc resolves the UID owning a connection, or -1 (INVALID_UID)
// when the owner cannot be determined.
type OwnerUidFunc func(proto Proto, src, dst netip.AddrPort) int32

// SpecialLocalPrefixes are always-local ranges that "allow local network
// access" (LAN bypass) may always exempt from the tunnel: link-local,
// broadcast, and multicast. Crucially they do NOT include the RFC1918 unicast
// ranges (10/8, 172.16/12, 192.168/16) or the IPv6 ULA range (fc00::/7): the
// Nym multihop tunnel addresses its OWN in-tunnel infrastructure out of RFC1918
// (e.g. the exit gateway at 10.1.0.1), so blanket-bypassing all of RFC1918
// would divert the tunnel's own traffic onto the underlying network and break
// connect. The actual unicast local subnet(s) to bypass are instead supplied at
// runtime from the underlying network's link properties (see NewClassifier's
// lanPrefixes), so only the device's real local network is bypassed.
var SpecialLocalPrefixes = []netip.Prefix{
	netip.MustParsePrefix("169.254.0.0/16"),
	netip.MustParsePrefix("fe80::/10"),
	// Broadcast / multicast (needed for LAN service discovery: mDNS, SSDP, ...).
	netip.MustParsePrefix("255.255.255.255/32"),
	netip.MustParsePrefix("224.0.0.0/24"),
	netip.MustParsePrefix("239.0.0.0/8"),
	netip.MustParsePrefix("ff01::/16"),
	netip.MustParsePrefix("ff02::/16"),
	netip.MustParsePrefix("ff03::/16"),
	netip.MustParsePrefix("ff04::/16"),
	netip.MustParsePrefix("ff05::/16"),
}

// Classifier decides whether a new flow is routed through the tunnel or
// bypassed directly. Two independent bypass criteria: destination in a local
// network (LAN bypass) and ownership by an excluded app UID. Unattributable
// flows always go through the tunnel.
type Classifier struct {
	excludedUIDs map[uint32]struct{}
	ownerUID     OwnerUidFunc
	// lanPrefixes are the local-network ranges to bypass; empty disables LAN
	// bypass. The caller assembles this from the underlying network's real
	// subnet(s) plus SpecialLocalPrefixes (see engine.Start).
	lanPrefixes []netip.Prefix
}

func NewClassifier(excludedUIDs []uint32, ownerUID OwnerUidFunc, lanPrefixes []netip.Prefix) *Classifier {
	m := make(map[uint32]struct{}, len(excludedUIDs))
	for _, uid := range excludedUIDs {
		m[uid] = struct{}{}
	}
	return &Classifier{excludedUIDs: m, ownerUID: ownerUID, lanPrefixes: lanPrefixes}
}

// LanBypassPrefixes returns the effective set of local-network prefixes to
// bypass: the provided real subnet(s) (from the underlying network) plus the
// always-local SpecialLocalPrefixes when enabled, or nil when disabled.
func LanBypassPrefixes(enabled bool, provided []netip.Prefix) []netip.Prefix {
	if !enabled {
		return nil
	}
	out := make([]netip.Prefix, 0, len(provided)+len(SpecialLocalPrefixes))
	out = append(out, provided...)
	out = append(out, SpecialLocalPrefixes...)
	return out
}

// IsLanDestBypass reports whether the flow must be bypassed because it targets
// the local network and LAN bypass is enabled. This is a pure function of the
// destination address (no UID attribution) and takes precedence over UID-based
// classification and the DNS-to-tunnel fallback: local-network traffic must go
// direct so it survives Android's "block connections without VPN" kill switch,
// which otherwise blocks the route-based LAN exemption.
func (c *Classifier) IsLanDestBypass(key FlowKey) bool {
	if len(c.lanPrefixes) == 0 {
		return false
	}
	addr := key.Dst.Addr().Unmap()
	for _, p := range c.lanPrefixes {
		if p.Contains(addr) {
			return true
		}
	}
	return false
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
