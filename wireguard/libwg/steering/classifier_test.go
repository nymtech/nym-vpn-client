package steering

import (
	"net/netip"
	"testing"
)

func TestClassifierExcludedUidBypasses(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return 10123 }, nil)
	if got := c.Decide(key(1)); got != DecisionBypass {
		t.Fatalf("expected bypass, got %v", got)
	}
}

func TestClassifierNonExcludedUidTunnels(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return 10999 }, nil)
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel, got %v", got)
	}
}

func TestClassifierInvalidUidFailsClosed(t *testing.T) {
	c := NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 }, nil)
	if got := c.Decide(key(1)); got != DecisionTunnel {
		t.Fatalf("expected tunnel on INVALID_UID, got %v", got)
	}
}

func TestClassifierNilCallbackFailsClosed(t *testing.T) {
	c := NewClassifier([]uint32{10123}, nil, nil)
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

func prefixes(cidrs ...string) []netip.Prefix {
	out := make([]netip.Prefix, 0, len(cidrs))
	for _, c := range cidrs {
		out = append(out, netip.MustParsePrefix(c))
	}
	return out
}

func TestIsLanDestBypassWhenEnabled(t *testing.T) {
	// The device's real local subnet(s) plus the always-local special ranges
	// (link-local, broadcast, multicast) that LanBypassPrefixes appends.
	c := NewClassifier(nil, nil, LanBypassPrefixes(true, prefixes("10.5.6.0/24", "192.168.1.0/24")))
	for _, dst := range []string{
		"192.168.1.1:80", "10.5.6.7:443", // on a real local subnet
		"169.254.1.1:80", "[fe80::1]:80", "255.255.255.255:9", "224.0.0.251:5353", // always-local
	} {
		if !c.IsLanDestBypass(lanKey(dst)) {
			t.Fatalf("expected LAN-dest bypass for %s", dst)
		}
	}
}

// TestIsLanDestBypassScopedToProvidedSubnet pins the fix: an RFC1918 address
// that is the tunnel's own in-tunnel gateway (10.1.0.1) but NOT on the device's
// real local subnet must stay tunneled, or the tunnel diverts its own traffic
// onto the underlying network and connect never completes.
func TestIsLanDestBypassScopedToProvidedSubnet(t *testing.T) {
	c := NewClassifier(nil, nil, LanBypassPrefixes(true, prefixes("10.223.228.0/24")))
	if c.IsLanDestBypass(lanKey("10.1.0.1:51830")) {
		t.Fatal("in-tunnel gateway 10.1.0.1 must NOT be LAN-bypassed: it is not on the real local subnet")
	}
	if !c.IsLanDestBypass(lanKey("10.223.228.5:80")) {
		t.Fatal("an address on the real local subnet must be LAN-bypassed")
	}
}

func TestIsLanDestBypassIgnoresNonLan(t *testing.T) {
	c := NewClassifier(nil, nil, LanBypassPrefixes(true, prefixes("10.5.6.0/24")))
	for _, dst := range []string{"1.2.3.4:443", "8.8.8.8:53", "[2001:4860:4860::8888]:53"} {
		if c.IsLanDestBypass(lanKey(dst)) {
			t.Fatalf("expected NO LAN-dest bypass for public %s", dst)
		}
	}
}

func TestIsLanDestBypassDisabled(t *testing.T) {
	// LAN bypass off: LanBypassPrefixes returns nil, so even a would-be-local
	// destination is not bypassed on this criterion.
	c := NewClassifier(nil, nil, LanBypassPrefixes(false, prefixes("192.168.1.0/24")))
	if c.IsLanDestBypass(lanKey("192.168.1.1:80")) {
		t.Fatal("expected no LAN-dest bypass when LAN bypass is disabled")
	}
}
