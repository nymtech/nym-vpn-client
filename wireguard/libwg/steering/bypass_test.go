/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net/netip"
	"testing"

	"github.com/amnezia-vpn/amneziawg-go/device"
)

func TestDialAddrRedirectsDNSToUnderlyingResolver(t *testing.T) {
	dns := []netip.Addr{netip.MustParseAddr("192.168.1.1"), netip.MustParseAddr("fd00::1")}
	got := resolveBypassDialAddr(netip.AddrPortFrom(netip.MustParseAddr("10.64.0.1"), 53), dns)
	want := netip.AddrPortFrom(netip.MustParseAddr("192.168.1.1"), 53)
	if got != want {
		t.Fatalf("got %v want %v", got, want)
	}
}

func TestDialAddrMatchesAddressFamilyForDNS(t *testing.T) {
	dns := []netip.Addr{netip.MustParseAddr("192.168.1.1"), netip.MustParseAddr("fd00::1")}
	got := resolveBypassDialAddr(netip.AddrPortFrom(netip.MustParseAddr("fd00:aaaa::53"), 53), dns)
	want := netip.AddrPortFrom(netip.MustParseAddr("fd00::1"), 53)
	if got != want {
		t.Fatalf("got %v want %v", got, want)
	}
}

func TestDialAddrPassthroughForNonDNS(t *testing.T) {
	dns := []netip.Addr{netip.MustParseAddr("192.168.1.1")}
	orig := netip.AddrPortFrom(netip.MustParseAddr("1.2.3.4"), 443)
	if got := resolveBypassDialAddr(orig, dns); got != orig {
		t.Fatalf("got %v want %v", got, orig)
	}
}

func TestDialAddrPassthroughDNSWhenNoResolvers(t *testing.T) {
	orig := netip.AddrPortFrom(netip.MustParseAddr("10.64.0.1"), 53)
	if got := resolveBypassDialAddr(orig, nil); got != orig {
		t.Fatalf("got %v want %v", got, orig)
	}
}

// TestNewBypassStackRejectsNilProtect asserts the nil-Protect invariant
// structurally: without a Protect callback, a dialed socket for a bypassed
// flow would go out unprotected, re-entering the VPN routing loop the
// design forbids. newBypassStack must refuse to construct in that case
// rather than silently producing a fail-open dialer.
func TestNewBypassStackRejectsNilProtect(t *testing.T) {
	cfg := Config{MTU: 1500}
	cb := Callbacks{
		Protect:  nil,
		OwnerUID: func(proto Proto, src, dst netip.AddrPort) int32 { return -1 },
	}
	logger := device.NewLogger(device.LogLevelError, "test")
	b, err := newBypassStack(cfg, cb, func([]byte) {}, logger)
	if err == nil {
		if b != nil {
			b.Close()
		}
		t.Fatal("newBypassStack with nil Protect: got nil error, want error")
	}
	if b != nil {
		t.Fatal("newBypassStack with nil Protect: got non-nil stack, want nil")
	}
}
