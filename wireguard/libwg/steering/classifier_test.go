package steering

import (
	"net/netip"
	"testing"
)

func TestClassifierExcludedUidBypasses(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return 10123 }, false)
	if got := c.Decide(key(1)); got != DecisionBypass {
		t.Fatalf("expected bypass, got %v", got)
	}
}

func TestClassifierNonExcludedUidTunnels(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return 10999 }, false)
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel, got %v", got)
	}
}

func TestClassifierInvalidUidFailsClosed(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 }, false)
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel on INVALID_UID, got %v", got)
	}
}

func TestClassifierNilCallbackFailsClosed(t *testing.T) {
	c := NewClassifier([]uint32{10123}, nil, false)
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel with nil callback, got %v", got)
	}
}

func lanKey(dst string) FlowKey {
	return FlowKey{
		Proto: ProtoTCP,
		Src:   netip.AddrPortFrom(netip.MustParseAddr("10.0.0.2"), 5000),
		Dst:   netip.MustParseAddrPort(dst),
	}
}

func TestIsLanDestBypassWhenEnabled(t *testing.T) {
	c := NewClassifier(nil, nil, true)
	for _, dst := range []string{"192.168.1.1:80", "10.5.6.7:443", "172.16.0.9:53", "169.254.1.1:80", "[fe80::1]:80", "[fc00::1]:443"} {
		if !c.IsLanDestBypass(lanKey(dst)) {
			t.Fatalf("expected LAN-dest bypass for %s", dst)
		}
	}
}

func TestIsLanDestBypassIgnoresNonLan(t *testing.T) {
	c := NewClassifier(nil, nil, true)
	for _, dst := range []string{"1.2.3.4:443", "8.8.8.8:53", "[2001:4860:4860::8888]:53"} {
		if c.IsLanDestBypass(lanKey(dst)) {
			t.Fatalf("expected NO LAN-dest bypass for public %s", dst)
		}
	}
}

func TestIsLanDestBypassDisabled(t *testing.T) {
	// LAN bypass off: even a LAN destination is not bypassed on this criterion.
	c := NewClassifier(nil, nil, false)
	if c.IsLanDestBypass(lanKey("192.168.1.1:80")) {
		t.Fatal("expected no LAN-dest bypass when bypassLan is disabled")
	}
}
