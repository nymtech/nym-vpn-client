/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net/netip"
	"testing"
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
