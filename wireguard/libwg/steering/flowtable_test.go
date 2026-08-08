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

// TestFlowTablePerEntryTTL covers the shorter lifetime the engine gives UDP
// entries: UDP has no SYN to re-classify on, so a cached decision must expire
// with the bypass relay's idle timeout rather than linger for the table's
// default TTL, after which another app could reuse the same 5-tuple.
func TestFlowTablePerEntryTTL(t *testing.T) {
	now := time.Unix(1000, 0)
	clock := func() time.Time { return now }
	ft := NewFlowTable(10, 5*time.Minute, clock)
	ft.InsertWithTTL(key(1000), DecisionBypass, 60*time.Second)
	ft.Insert(key(2000), DecisionBypass) // table default TTL

	now = now.Add(61 * time.Second)
	if _, ok := ft.Lookup(key(1000)); ok {
		t.Fatal("expected short-TTL entry to expire after its own ttl, not the table's")
	}
	if _, ok := ft.Lookup(key(2000)); !ok {
		t.Fatal("expected default-TTL entry to still be live")
	}
}
