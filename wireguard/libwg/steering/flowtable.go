/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

// Package steering routes packets between the Android TUN device, the VPN
// tunnel, and a direct-bypass netstack, so that split-tunnel-excluded apps
// keep connectivity under Android VPN lockdown.
package steering

import (
	"container/list"
	"net/netip"
	"sync"
	"time"
)

type Proto uint8

const (
	ProtoTCP Proto = 6
	ProtoUDP Proto = 17
)

type FlowKey struct {
	Proto Proto
	Src   netip.AddrPort
	Dst   netip.AddrPort
}

type Decision uint8

const (
	DecisionTunnel Decision = 0
	DecisionBypass Decision = 1
)

type flowEntry struct {
	key      FlowKey
	decision Decision
	seen     time.Time
}

// FlowTable is a bounded LRU cache of per-flow routing decisions.
type FlowTable struct {
	mu         sync.Mutex
	maxEntries int
	ttl        time.Duration
	now        func() time.Time
	entries    map[FlowKey]*list.Element
	order      *list.List // front = most recently used
}

func NewFlowTable(maxEntries int, ttl time.Duration, now func() time.Time) *FlowTable {
	return &FlowTable{
		maxEntries: maxEntries,
		ttl:        ttl,
		now:        now,
		entries:    make(map[FlowKey]*list.Element),
		order:      list.New(),
	}
}

func (t *FlowTable) Lookup(key FlowKey) (Decision, bool) {
	t.mu.Lock()
	defer t.mu.Unlock()
	el, ok := t.entries[key]
	if !ok {
		return DecisionTunnel, false
	}
	entry := el.Value.(*flowEntry)
	if t.now().Sub(entry.seen) > t.ttl {
		t.order.Remove(el)
		delete(t.entries, key)
		return DecisionTunnel, false
	}
	entry.seen = t.now()
	t.order.MoveToFront(el)
	return entry.decision, true
}

func (t *FlowTable) Insert(key FlowKey, d Decision) {
	t.mu.Lock()
	defer t.mu.Unlock()
	if el, ok := t.entries[key]; ok {
		el.Value.(*flowEntry).decision = d
		el.Value.(*flowEntry).seen = t.now()
		t.order.MoveToFront(el)
		return
	}
	for len(t.entries) >= t.maxEntries {
		oldest := t.order.Back()
		if oldest == nil {
			break
		}
		t.order.Remove(oldest)
		delete(t.entries, oldest.Value.(*flowEntry).key)
	}
	el := t.order.PushFront(&flowEntry{key: key, decision: d, seen: t.now()})
	t.entries[key] = el
}
