package steering

import (
	"net/netip"
	"testing"
	"time"
)

func key(port uint16) FlowKey {
	return FlowKey{
		Proto: ProtoTCP,
		Src:   netip.AddrPortFrom(netip.MustParseAddr("10.0.0.2"), port),
		Dst:   netip.AddrPortFrom(netip.MustParseAddr("1.2.3.4"), 443),
	}
}

func TestFlowTableInsertLookup(t *testing.T) {
	ft := NewFlowTable(10, time.Minute, time.Now)
	if _, ok := ft.Lookup(key(1000)); ok {
		t.Fatal("expected miss on empty table")
	}
	ft.Insert(key(1000), DecisionBypass)
	d, ok := ft.Lookup(key(1000))
	if !ok || d != DecisionBypass {
		t.Fatalf("expected bypass hit, got %v %v", d, ok)
	}
}

func TestFlowTableTTLExpiry(t *testing.T) {
	now := time.Unix(1000, 0)
	clock := func() time.Time { return now }
	ft := NewFlowTable(10, 60*time.Second, clock)
	ft.Insert(key(1000), DecisionTunnel)
	now = now.Add(61 * time.Second)
	if _, ok := ft.Lookup(key(1000)); ok {
		t.Fatal("expected expired entry to miss")
	}
}

func TestFlowTableEvictsOldestWhenFull(t *testing.T) {
	ft := NewFlowTable(2, time.Minute, time.Now)
	ft.Insert(key(1), DecisionTunnel)
	ft.Insert(key(2), DecisionTunnel)
	ft.Insert(key(3), DecisionTunnel) // evicts key(1)
	if _, ok := ft.Lookup(key(1)); ok {
		t.Fatal("expected oldest entry evicted")
	}
	if _, ok := ft.Lookup(key(3)); !ok {
		t.Fatal("expected newest entry present")
	}
}
