/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import "net/netip"

// OwnerUidFunc resolves the UID owning a connection, or -1 (INVALID_UID)
// when the owner cannot be determined.
type OwnerUidFunc func(proto Proto, src, dst netip.AddrPort) int32

// Classifier decides whether a new flow is routed through the tunnel or
// bypassed directly. Unattributable flows always go through the tunnel.
type Classifier struct {
	excludedUIDs map[uint32]struct{}
	ownerUID     OwnerUidFunc
}

func NewClassifier(excludedUIDs []uint32, ownerUID OwnerUidFunc) *Classifier {
	m := make(map[uint32]struct{}, len(excludedUIDs))
	for _, uid := range excludedUIDs {
		m[uid] = struct{}{}
	}
	return &Classifier{excludedUIDs: m, ownerUID: ownerUID}
}

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
