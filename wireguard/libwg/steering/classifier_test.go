package steering

import (
	"net/netip"
	"testing"
)

func TestClassifierExcludedUidBypasses(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return 10123 })
	if got := c.Decide(key(1)); got != DecisionBypass {
		t.Fatalf("expected bypass, got %v", got)
	}
}

func TestClassifierNonExcludedUidTunnels(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return 10999 })
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel, got %v", got)
	}
}

func TestClassifierInvalidUidFailsClosed(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 })
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel on INVALID_UID, got %v", got)
	}
}

func TestClassifierNilCallbackFailsClosed(t *testing.T) {
	c := NewClassifier([]uint32{10123}, nil)
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel with nil callback, got %v", got)
	}
}
