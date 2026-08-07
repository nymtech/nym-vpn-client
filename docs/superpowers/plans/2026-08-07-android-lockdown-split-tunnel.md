# Android Lockdown-Compatible Split Tunneling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make split-tunnel-excluded apps keep direct internet access when Android's "Block connections without VPN" (lockdown) is enabled, by routing all apps into the TUN and forwarding excluded apps' flows over `VpnService.protect()`-ed sockets.

**Architecture:** A new Go "steering" engine in `wireguard/libwg` owns the real Android TUN fd. Tunneled traffic passes through raw to a socketpair whose other end becomes the `AsyncDevice` the existing Rust/Go tunnel code consumes (the `dns_filter_proxy.rs` / `socketTun` pattern, already shipping). Excluded apps' TCP/UDP flows are terminated in a gVisor netstack (already a libwg dependency) and dialed out via protected sockets. Flow→app attribution happens via a callback chain Go → Rust → Kotlin `ConnectivityManager.getConnectionOwnerUid()`. Kotlin decides per-connect whether steering is active (lockdown detected + non-empty exclusion list) and, if so, skips `addDisallowedApplication()`.

**Tech Stack:** Go (gVisor `pkg/tcpip`, cgo), Rust (nym-wg-go FFI, uniffi), Kotlin (VpnService, Compose).

**Spec:** `docs/superpowers/specs/2026-08-07-android-lockdown-split-tunnel-design.md`

## Global Constraints

- **Git (per user ruling 2026-08-07):** work happens in an isolated git worktree on a dedicated branch; implementers commit per task there. Never push, never merge, never touch `develop` — the user reviews, squashes, merges, and pushes everything themselves. End each task with a commit on the worktree branch plus a list of changed files. Commit messages must not mention Claude or AI authorship — no Co-Authored-By trailer, plain conventional messages only (user request 2026-08-07).
- **Fail-closed:** any flow that cannot be attributed (`INVALID_UID` = -1, callback error, non-TCP/UDP protocol) goes **through the tunnel**, never direct.
- Steering activates only when: exclusion list non-empty AND `VpnService.isLockdownEnabled()` is true AND API ≥ 29. All other cases keep today's `addDisallowedApplication()` behavior byte-for-byte.
- Android minSdk is 24; `isLockdownEnabled` and `getConnectionOwnerUid` are API 29+ — every use must be guarded by `Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q`.
- Excluded apps' DNS is redirected to the underlying network's resolvers at dial time (no packet checksum rewriting); if none are known, DNS flows fall back to the tunnel.
- Kotlin uses tabs for indentation (match existing files). Go/Rust: run `gofmt` / `cargo fmt` on touched files.
- Build commands: Go tests `cd wireguard/libwg && go test ./steering/...`; Rust `cd nym-vpn-core && cargo check -p nym-wg-go -p nym-vpn-lib`; full Android native build `cd nym-vpn-core && make -f Android.mk` (needs NDK + `cargo-ndk`); uniffi Kotlin regen `make -f Android.mk uniffi`; Kotlin tests `cd nym-vpn-android && ./gradlew :core:testDebugUnitTest :app:testDebugUnitTest`.

---

### Task 1: Go flow table and classifier (`steering` package core)

**Files:**
- Create: `wireguard/libwg/steering/flowtable.go`
- Create: `wireguard/libwg/steering/classifier.go`
- Test: `wireguard/libwg/steering/flowtable_test.go`, `wireguard/libwg/steering/classifier_test.go`

**Interfaces:**
- Consumes: nothing (pure Go + stdlib `net/netip`).
- Produces: `type Proto uint8` (`ProtoTCP Proto = 6`, `ProtoUDP Proto = 17`); `type FlowKey struct { Proto Proto; Src netip.AddrPort; Dst netip.AddrPort }`; `type Decision uint8` (`DecisionTunnel Decision = 0`, `DecisionBypass Decision = 1`); `func NewFlowTable(maxEntries int, ttl time.Duration, now func() time.Time) *FlowTable` with methods `Lookup(FlowKey) (Decision, bool)` and `Insert(FlowKey, Decision)`; `type OwnerUidFunc func(proto Proto, src, dst netip.AddrPort) int32`; `func NewClassifier(excludedUIDs []uint32, ownerUID OwnerUidFunc) *Classifier` with method `Decide(FlowKey) Decision`. Task 4's engine consumes all of these.

- [ ] **Step 1: Write failing tests**

`wireguard/libwg/steering/flowtable_test.go`:

```go
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
```

`wireguard/libwg/steering/classifier_test.go`:

```go
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
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cd wireguard/libwg && go test ./steering/...`
Expected: compile errors (`undefined: NewFlowTable` etc.).

- [ ] **Step 3: Implement**

`wireguard/libwg/steering/flowtable.go` — map + doubly-linked LRU list (use `container/list` from stdlib):

```go
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
```

`wireguard/libwg/steering/classifier.go`:

```go
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
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cd wireguard/libwg && go test ./steering/... && gofmt -l steering/`
Expected: PASS, no gofmt output.

- [ ] **Step 5: Report changed files** (no commit — user handles git).

---

### Task 2: Go packet parser

**Files:**
- Create: `wireguard/libwg/steering/packet.go`
- Test: `wireguard/libwg/steering/packet_test.go`

**Interfaces:**
- Consumes: `FlowKey`, `Proto` from Task 1; gVisor `gvisor.dev/gvisor/pkg/tcpip/header` (already in `go.mod` transitively — add as direct require if `go mod tidy` asks).
- Produces: `type PacketInfo struct { Key FlowKey; IsIPv4 bool; IsTCPSyn bool }`; `func ParsePacket(pkt []byte) (PacketInfo, bool)` — returns `false` for anything that is not a well-formed IPv4/IPv6 TCP or UDP packet (callers then route it to the tunnel).

- [ ] **Step 1: Write failing tests** — build packets with gVisor `header` so the test is self-checking:

```go
package steering

import (
	"net/netip"
	"testing"

	"gvisor.dev/gvisor/pkg/tcpip"
	"gvisor.dev/gvisor/pkg/tcpip/header"
)

func buildIPv4UDP(src, dst netip.Addr, srcPort, dstPort uint16) []byte {
	payload := []byte("hi")
	length := header.IPv4MinimumSize + header.UDPMinimumSize + len(payload)
	buf := make([]byte, length)
	ip := header.IPv4(buf)
	ip.Encode(&header.IPv4Fields{
		TotalLength: uint16(length),
		TTL:         64,
		Protocol:    uint8(header.UDPProtocolNumber),
		SrcAddr:     tcpip.AddrFrom4(src.As4()),
		DstAddr:     tcpip.AddrFrom4(dst.As4()),
	})
	ip.SetChecksum(^ip.CalculateChecksum())
	udp := header.UDP(buf[header.IPv4MinimumSize:])
	udp.Encode(&header.UDPFields{
		SrcPort: srcPort,
		DstPort: dstPort,
		Length:  uint16(header.UDPMinimumSize + len(payload)),
	})
	copy(buf[header.IPv4MinimumSize+header.UDPMinimumSize:], payload)
	return buf
}

func TestParseIPv4UDP(t *testing.T) {
	src := netip.MustParseAddr("10.0.0.2")
	dst := netip.MustParseAddr("9.9.9.9")
	info, ok := ParsePacket(buildIPv4UDP(src, dst, 5353, 53))
	if !ok {
		t.Fatal("expected parse success")
	}
	if info.Key.Proto != ProtoUDP || !info.IsIPv4 {
		t.Fatalf("wrong proto/family: %+v", info)
	}
	if info.Key.Src != netip.AddrPortFrom(src, 5353) || info.Key.Dst != netip.AddrPortFrom(dst, 53) {
		t.Fatalf("wrong addrs: %+v", info.Key)
	}
}

func TestParseRejectsNonTcpUdp(t *testing.T) {
	// ICMP echo: minimal IPv4 header with protocol 1
	buf := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), netip.MustParseAddr("9.9.9.9"), 1, 1)
	header.IPv4(buf).Encode(&header.IPv4Fields{
		TotalLength: uint16(len(buf)),
		TTL:         64,
		Protocol:    1, // ICMP
		SrcAddr:     tcpip.AddrFrom4(netip.MustParseAddr("10.0.0.2").As4()),
		DstAddr:     tcpip.AddrFrom4(netip.MustParseAddr("9.9.9.9").As4()),
	})
	if _, ok := ParsePacket(buf); ok {
		t.Fatal("expected ICMP to be rejected")
	}
}

func TestParseRejectsTruncated(t *testing.T) {
	if _, ok := ParsePacket([]byte{0x45, 0x00}); ok {
		t.Fatal("expected truncated packet to be rejected")
	}
}
```

Also add an IPv6 TCP test mirroring `TestParseIPv4UDP` using `header.IPv6` / `header.TCP` with `IsTCPSyn` asserted true when the SYN flag is set.

- [ ] **Step 2: Run tests, verify fail** — `cd wireguard/libwg && go test ./steering/...` → `undefined: ParsePacket`.

- [ ] **Step 3: Implement** `wireguard/libwg/steering/packet.go`:

```go
/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net/netip"

	"gvisor.dev/gvisor/pkg/tcpip/header"
)

type PacketInfo struct {
	Key      FlowKey
	IsIPv4   bool
	IsTCPSyn bool
}

// ParsePacket extracts the flow key from a raw IP packet. It returns false
// for any packet that is not well-formed IPv4/IPv6 TCP or UDP; such packets
// must be routed through the tunnel.
func ParsePacket(pkt []byte) (PacketInfo, bool) {
	if len(pkt) == 0 {
		return PacketInfo{}, false
	}
	switch header.IPVersion(pkt) {
	case header.IPv4Version:
		ip := header.IPv4(pkt)
		if len(pkt) < header.IPv4MinimumSize || !ip.IsValid(len(pkt)) {
			return PacketInfo{}, false
		}
		src, _ := netip.AddrFromSlice(ip.SourceAddress().AsSlice())
		dst, _ := netip.AddrFromSlice(ip.DestinationAddress().AsSlice())
		return parseTransport(pkt[ip.HeaderLength():], uint8(ip.Protocol()), src, dst, true)
	case header.IPv6Version:
		if len(pkt) < header.IPv6MinimumSize {
			return PacketInfo{}, false
		}
		ip := header.IPv6(pkt)
		src, _ := netip.AddrFromSlice(ip.SourceAddress().AsSlice())
		dst, _ := netip.AddrFromSlice(ip.DestinationAddress().AsSlice())
		// NextHeader chains (extension headers) are rare on first hop; treat
		// anything other than a directly nested TCP/UDP as tunnel traffic.
		return parseTransport(pkt[header.IPv6MinimumSize:], uint8(ip.NextHeader()), src, dst, false)
	default:
		return PacketInfo{}, false
	}
}

func parseTransport(payload []byte, proto uint8, src, dst netip.Addr, isIPv4 bool) (PacketInfo, bool) {
	switch Proto(proto) {
	case ProtoTCP:
		if len(payload) < header.TCPMinimumSize {
			return PacketInfo{}, false
		}
		tcp := header.TCP(payload)
		return PacketInfo{
			Key: FlowKey{
				Proto: ProtoTCP,
				Src:   netip.AddrPortFrom(src, tcp.SourcePort()),
				Dst:   netip.AddrPortFrom(dst, tcp.DestinationPort()),
			},
			IsIPv4:   isIPv4,
			IsTCPSyn: tcp.Flags()&header.TCPFlagSyn != 0 && tcp.Flags()&header.TCPFlagAck == 0,
		}, true
	case ProtoUDP:
		if len(payload) < header.UDPMinimumSize {
			return PacketInfo{}, false
		}
		udp := header.UDP(payload)
		return PacketInfo{
			Key: FlowKey{
				Proto: ProtoUDP,
				Src:   netip.AddrPortFrom(src, udp.SourcePort()),
				Dst:   netip.AddrPortFrom(dst, udp.DestinationPort()),
			},
			IsIPv4: isIPv4,
		}, true
	default:
		return PacketInfo{}, false
	}
}
```

Run `go mod tidy` if gVisor needs promoting to a direct dependency.

- [ ] **Step 4: Run tests, verify pass** — `cd wireguard/libwg && go test ./steering/...`.

- [ ] **Step 5: Report changed files.**

---

### Task 3: Go bypass netstack (gVisor + protected dialer + DNS redirect)

**Files:**
- Create: `wireguard/libwg/steering/bypass.go`
- Test: `wireguard/libwg/steering/bypass_test.go`

**Interfaces:**
- Consumes: `Config`/`Callbacks` (defined here, used by Task 4), gVisor `stack`, `channel`, `tcp`, `udp`, `gonet` packages (same versions the amneziawg-go netstack uses — check `go.mod`).
- Produces:
  - `type Config struct { ExcludedUIDs []uint32; UnderlyingDNS []netip.Addr; MTU int }`
  - `type Callbacks struct { Protect func(fd int32); OwnerUID OwnerUidFunc }`
  - `func newBypassStack(cfg Config, cb Callbacks, writeToTun func([]byte), logger *device.Logger) (*bypassStack, error)` with methods `InjectInbound(pkt []byte, isIPv4 bool)` and `Close()`
  - `func resolveBypassDialAddr(dst netip.AddrPort, underlyingDNS []netip.Addr) netip.AddrPort` (exported for tests as `ResolveBypassDialAddr` if preferred; keep package-private + test in-package)

- [ ] **Step 1: Write failing test for the DNS redirect decision** (the pure-logic part):

```go
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
```

Note: the "no resolvers" case never reaches the bypass stack in practice — Task 4's engine routes DNS flows to the tunnel when `UnderlyingDNS` is empty — but `resolveBypassDialAddr` must still be total.

- [ ] **Step 2: Run tests, verify fail** — `undefined: resolveBypassDialAddr`.

- [ ] **Step 3: Implement** `wireguard/libwg/steering/bypass.go`. This is the tun2socks pattern: a `stack.Stack` fed by a `channel.Endpoint`; `tcp.NewForwarder`/`udp.NewForwarder` accept flows and bridge them to real sockets dialed with a `Control` hook that protects the fd.

```go
/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"context"
	"fmt"
	"io"
	"net"
	"net/netip"
	"sync"
	"syscall"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"gvisor.dev/gvisor/pkg/buffer"
	"gvisor.dev/gvisor/pkg/tcpip"
	"gvisor.dev/gvisor/pkg/tcpip/adapters/gonet"
	"gvisor.dev/gvisor/pkg/tcpip/header"
	"gvisor.dev/gvisor/pkg/tcpip/link/channel"
	"gvisor.dev/gvisor/pkg/tcpip/network/ipv4"
	"gvisor.dev/gvisor/pkg/tcpip/network/ipv6"
	"gvisor.dev/gvisor/pkg/tcpip/stack"
	"gvisor.dev/gvisor/pkg/tcpip/transport/tcp"
	"gvisor.dev/gvisor/pkg/tcpip/transport/udp"
	"gvisor.dev/gvisor/pkg/waiter"
)

const (
	bypassNICID          = tcpip.NICID(1)
	udpIdleTimeout       = 60 * time.Second
	dnsPort              = 53
	tcpForwarderInFlight = 1024
)

type Config struct {
	ExcludedUIDs  []uint32
	UnderlyingDNS []netip.Addr
	MTU           int
}

type Callbacks struct {
	// Protect marks a socket fd to bypass the VPN (VpnService.protect()).
	Protect func(fd int32)
	// OwnerUID resolves the app UID owning a connection, -1 if unknown.
	OwnerUID OwnerUidFunc
}

// resolveBypassDialAddr returns the address a bypassed flow should actually
// dial: DNS flows are redirected to an underlying-network resolver of the
// same address family; everything else dials its original destination.
func resolveBypassDialAddr(dst netip.AddrPort, underlyingDNS []netip.Addr) netip.AddrPort {
	if dst.Port() != dnsPort || len(underlyingDNS) == 0 {
		return dst
	}
	for _, r := range underlyingDNS {
		if r.Is4() == dst.Addr().Is4() {
			return netip.AddrPortFrom(r, dnsPort)
		}
	}
	return netip.AddrPortFrom(underlyingDNS[0], dnsPort)
}

type bypassStack struct {
	stack     *stack.Stack
	endpoint  *channel.Endpoint
	dialer    net.Dialer
	cfg       Config
	logger    *device.Logger
	ctx       context.Context
	cancel    context.CancelFunc
	waitGroup sync.WaitGroup
}

func newBypassStack(cfg Config, cb Callbacks, writeToTun func([]byte), logger *device.Logger) (*bypassStack, error) {
	s := stack.New(stack.Options{
		NetworkProtocols:   []stack.NetworkProtocolFactory{ipv4.NewProtocol, ipv6.NewProtocol},
		TransportProtocols: []stack.TransportProtocolFactory{tcp.NewProtocol, udp.NewProtocol},
	})
	ep := channel.New(512, uint32(cfg.MTU), "")
	if err := s.CreateNIC(bypassNICID, ep); err != nil {
		s.Destroy()
		return nil, fmt.Errorf("create NIC: %s", err)
	}
	// Accept any destination/source address arriving from the TUN.
	if err := s.SetPromiscuousMode(bypassNICID, true); err != nil {
		s.Destroy()
		return nil, fmt.Errorf("set promiscuous: %s", err)
	}
	if err := s.SetSpoofing(bypassNICID, true); err != nil {
		s.Destroy()
		return nil, fmt.Errorf("set spoofing: %s", err)
	}
	s.SetRouteTable([]tcpip.Route{
		{Destination: header.IPv4EmptySubnet, NIC: bypassNICID},
		{Destination: header.IPv6EmptySubnet, NIC: bypassNICID},
	})

	ctx, cancel := context.WithCancel(context.Background())
	b := &bypassStack{
		stack:    s,
		endpoint: ep,
		cfg:      cfg,
		logger:   logger,
		ctx:      ctx,
		cancel:   cancel,
		dialer: net.Dialer{
			Control: func(network, address string, c syscall.RawConn) error {
				return c.Control(func(fd uintptr) {
					if cb.Protect != nil {
						cb.Protect(int32(fd))
					}
				})
			},
		},
	}

	tcpFwd := tcp.NewForwarder(s, 0, tcpForwarderInFlight, b.handleTCP)
	s.SetTransportProtocolHandler(tcp.ProtocolNumber, tcpFwd.HandlePacket)
	udpFwd := udp.NewForwarder(s, b.handleUDP)
	s.SetTransportProtocolHandler(udp.ProtocolNumber, udpFwd.HandlePacket)

	// Pump packets the netstack emits (responses to the apps) back to the TUN.
	b.waitGroup.Add(1)
	go func() {
		defer b.waitGroup.Done()
		for {
			pkt := ep.ReadContext(ctx)
			if pkt == nil {
				return
			}
			view := pkt.ToView()
			writeToTun(view.AsSlice())
			view.Release()
			pkt.DecRef()
		}
	}()

	return b, nil
}

func (b *bypassStack) InjectInbound(pkt []byte, isIPv4 bool) {
	proto := header.IPv6ProtocolNumber
	if isIPv4 {
		proto = header.IPv4ProtocolNumber
	}
	pb := stack.NewPacketBuffer(stack.PacketBufferOptions{
		Payload: buffer.MakeWithData(append([]byte(nil), pkt...)),
	})
	b.endpoint.InjectInbound(proto, pb)
	pb.DecRef()
}

func (b *bypassStack) Close() {
	b.cancel()
	b.endpoint.Close()
	b.stack.Destroy()
	b.waitGroup.Wait()
}

func (b *bypassStack) handleTCP(r *tcp.ForwarderRequest) {
	id := r.ID()
	dst := forwarderDstAddr(id)
	dialAddr := resolveBypassDialAddr(dst, b.cfg.UnderlyingDNS)

	dialCtx, dialCancel := context.WithTimeout(b.ctx, 10*time.Second)
	outbound, err := b.dialer.DialContext(dialCtx, "tcp", dialAddr.String())
	dialCancel()
	if err != nil {
		b.logger.Errorf("steering: bypass tcp dial %s failed: %s", dialAddr, err)
		r.Complete(true) // send RST so the app fails fast
		return
	}

	var wq waiter.Queue
	ep, tcpErr := r.CreateEndpoint(&wq)
	if tcpErr != nil {
		b.logger.Errorf("steering: bypass tcp endpoint: %s", tcpErr)
		outbound.Close()
		r.Complete(true)
		return
	}
	r.Complete(false)
	inbound := gonet.NewTCPConn(&wq, ep)

	b.waitGroup.Add(1)
	go func() {
		defer b.waitGroup.Done()
		pump(b.ctx, inbound, outbound)
	}()
}

func (b *bypassStack) handleUDP(r *udp.ForwarderRequest) {
	id := r.ID()
	dst := forwarderDstAddr(id)
	dialAddr := resolveBypassDialAddr(dst, b.cfg.UnderlyingDNS)

	var wq waiter.Queue
	ep, tcpErr := r.CreateEndpoint(&wq)
	if tcpErr != nil {
		b.logger.Errorf("steering: bypass udp endpoint: %s", tcpErr)
		return
	}
	inbound := gonet.NewUDPConn(&wq, ep)

	outbound, err := b.dialer.DialContext(b.ctx, "udp", dialAddr.String())
	if err != nil {
		b.logger.Errorf("steering: bypass udp dial %s failed: %s", dialAddr, err)
		inbound.Close()
		return
	}

	b.waitGroup.Add(1)
	go func() {
		defer b.waitGroup.Done()
		pumpUDP(b.ctx, inbound, outbound)
	}()
}

func forwarderDstAddr(id stack.TransportEndpointID) netip.AddrPort {
	// LocalAddress is the destination the app targeted (we are "the internet"
	// from the netstack's point of view).
	addr, _ := netip.AddrFromSlice(id.LocalAddress.AsSlice())
	return netip.AddrPortFrom(addr, id.LocalPort)
}

func pump(ctx context.Context, a, b net.Conn) {
	defer a.Close()
	defer b.Close()
	done := make(chan struct{}, 2)
	go func() { io.Copy(a, b); done <- struct{}{} }()
	go func() { io.Copy(b, a); done <- struct{}{} }()
	select {
	case <-done:
	case <-ctx.Done():
	}
}

func pumpUDP(ctx context.Context, inbound, outbound net.Conn) {
	defer inbound.Close()
	defer outbound.Close()
	done := make(chan struct{}, 2)
	relay := func(dst, src net.Conn) {
		buf := make([]byte, 65535)
		for {
			src.SetReadDeadline(time.Now().Add(udpIdleTimeout))
			n, err := src.Read(buf)
			if err != nil {
				done <- struct{}{}
				return
			}
			if _, err := dst.Write(buf[:n]); err != nil {
				done <- struct{}{}
				return
			}
		}
	}
	go relay(outbound, inbound)
	go relay(inbound, outbound)
	select {
	case <-done:
	case <-ctx.Done():
	}
}
```

Adjust to the gVisor API version pinned in `go.mod` (e.g. `PacketBuffer` construction and `channel.Endpoint.ReadContext` signatures changed across gVisor releases — compile against the vendored version and fix signatures accordingly; the amneziawg-go `tun/netstack` package in the module cache is the local reference for the correct idioms).

- [ ] **Step 4: Run tests + vet** — `cd wireguard/libwg && go test ./steering/... && go vet ./steering/...`
Expected: PASS. The netstack data path itself is covered by the leak/manual matrix in Task 11 (it requires a live network).

- [ ] **Step 5: Report changed files.**

---

### Task 4: Go steering engine + cgo exports

**Files:**
- Create: `wireguard/libwg/steering/engine.go`
- Create: `wireguard/libwg/steering_android.go` (package `main`, `//go:build android`)
- Test: `wireguard/libwg/steering/engine_test.go`

**Interfaces:**
- Consumes: Tasks 1–3 (`FlowTable`, `Classifier`, `ParsePacket`, `bypassStack`, `Config`, `Callbacks`); `container.Container` (existing, see `netstack.go:28-36`); `logging.NewLogger` (existing).
- Produces:
  - Go: `func Start(tunFd int, innerFd int, cfg Config, cb Callbacks, logger *device.Logger) (*Engine, error)`; `func (e *Engine) Stop()`.
  - cgo (consumed by Task 5's Rust bindings):
    ```c
    typedef void (*steering_protect_fn)(void *ctx, int32_t fd);
    typedef int32_t (*steering_owner_uid_fn)(void *ctx, int32_t protocol, const char *src, const char *dst);
    int32_t steeringTurnOn(int32_t tunFd, int32_t innerFd, int32_t mtu,
                           const uint32_t *excludedUids, int32_t uidCount,
                           const char *dnsServers, /* comma-separated IPs, may be NULL */
                           steering_protect_fn protectCb, steering_owner_uid_fn ownerUidCb, void *cbCtx,
                           void *logSink, void *logContext); // returns handle >= 0 or negative error
    void steeringTurnOff(int32_t handle);
    ```
    Callback address strings are `"ip:port"` (IPv6 as `"[addr]:port"`), i.e. Go `netip.AddrPort.String()` format.

- [ ] **Step 1: Write failing engine test** — passthrough behavior over real socketpairs, no netstack needed (empty exclusion list short-circuits classification):

```go
package steering

import (
	"net/netip"
	"os"
	"syscall"
	"testing"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
)

func socketPair(t *testing.T) (*os.File, *os.File) {
	t.Helper()
	fds, err := syscall.Socketpair(syscall.AF_UNIX, syscall.SOCK_DGRAM, 0)
	if err != nil {
		t.Fatal(err)
	}
	return os.NewFile(uintptr(fds[0]), "a"), os.NewFile(uintptr(fds[1]), "b")
}

func TestEnginePassthroughTunnelTraffic(t *testing.T) {
	tunA, tunB := socketPair(t)   // tunA = fake TUN device side, tunB = engine's tun fd
	innerA, innerB := socketPair(t) // innerA = engine's inner fd, innerB = fake wireguard side
	defer tunA.Close()
	defer innerB.Close()

	logger := device.NewLogger(device.LogLevelError, "test")
	eng, err := Start(int(tunB.Fd()), int(innerA.Fd()),
		Config{MTU: 1500},
		Callbacks{OwnerUID: func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 }},
		logger)
	if err != nil {
		t.Fatal(err)
	}
	defer eng.Stop()

	pkt := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), netip.MustParseAddr("1.2.3.4"), 1234, 4321)

	// upstream: TUN → engine → inner (tunnel)
	if _, err := tunA.Write(pkt); err != nil {
		t.Fatal(err)
	}
	buf := make([]byte, 2048)
	innerB.SetReadDeadline(time.Now().Add(2 * time.Second))
	n, err := innerB.Read(buf)
	if err != nil || n != len(pkt) {
		t.Fatalf("upstream passthrough failed: n=%d err=%v", n, err)
	}

	// downstream: inner → engine → TUN
	if _, err := innerB.Write(pkt); err != nil {
		t.Fatal(err)
	}
	tunA.SetReadDeadline(time.Now().Add(2 * time.Second))
	n, err = tunA.Read(buf)
	if err != nil || n != len(pkt) {
		t.Fatalf("downstream passthrough failed: n=%d err=%v", n, err)
	}
}
```

Add a second test `TestEngineUnattributableFlowGoesToTunnel`: same setup but `ExcludedUIDs: []uint32{10123}` and `OwnerUID` returning `-1` — packet must still arrive on `innerB` (fail-closed).

- [ ] **Step 2: Run tests, verify fail** — `undefined: Start`.

- [ ] **Step 3: Implement** `wireguard/libwg/steering/engine.go`:

```go
/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"os"
	"sync"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"golang.org/x/sys/unix"
)

const (
	maxPacketSize = 65536
	flowTableSize = 4096
	flowTTL       = 5 * time.Minute
)

// Engine owns the real Android TUN fd. Tunneled traffic passes through to the
// inner fd (consumed by wireguard-go or the mixnet processor); excluded apps'
// flows are terminated in the bypass netstack and dialed out directly.
type Engine struct {
	tunFile   *os.File
	innerFile *os.File
	flows     *FlowTable
	classify  *Classifier
	bypass    *bypassStack
	hasBypass bool
	dnsDirect bool
	logger    *device.Logger
	writeMu   sync.Mutex
	closeOnce sync.Once
	waitGroup sync.WaitGroup
}

func Start(tunFd int, innerFd int, cfg Config, cb Callbacks, logger *device.Logger) (*Engine, error) {
	// Non-blocking so os.File uses the runtime poller and Close() unblocks
	// pending reads (same as newSocketTunFromFD in libwg_android.go).
	for _, fd := range []int{tunFd, innerFd} {
		if err := unix.SetNonblock(fd, true); err != nil {
			return nil, err
		}
	}
	e := &Engine{
		tunFile:   os.NewFile(uintptr(tunFd), "steering-tun"),
		innerFile: os.NewFile(uintptr(innerFd), "steering-inner"),
		flows:     NewFlowTable(flowTableSize, flowTTL, time.Now),
		classify:  NewClassifier(cfg.ExcludedUIDs, cb.OwnerUID),
		hasBypass: len(cfg.ExcludedUIDs) > 0,
		dnsDirect: len(cfg.UnderlyingDNS) > 0,
		logger:    logger,
	}
	if e.hasBypass {
		b, err := newBypassStack(cfg, cb, e.writeToTun, logger)
		if err != nil {
			return nil, err
		}
		e.bypass = b
	}
	e.waitGroup.Add(2)
	go e.runUpstream()
	go e.runDownstream()
	logger.Verbosef("steering: engine started (excluded UIDs: %d, direct DNS: %v)", len(cfg.ExcludedUIDs), e.dnsDirect)
	return e, nil
}

func (e *Engine) Stop() {
	e.closeOnce.Do(func() {
		e.tunFile.Close()
		e.innerFile.Close()
		if e.bypass != nil {
			e.bypass.Close()
		}
	})
	e.waitGroup.Wait()
}

func (e *Engine) writeToTun(pkt []byte) {
	e.writeMu.Lock()
	defer e.writeMu.Unlock()
	if _, err := e.tunFile.Write(pkt); err != nil {
		e.logger.Verbosef("steering: write to tun failed: %s", err)
	}
}

// runUpstream reads app traffic from the TUN and routes each packet to the
// tunnel (inner fd) or the bypass netstack.
func (e *Engine) runUpstream() {
	defer e.waitGroup.Done()
	buf := make([]byte, maxPacketSize)
	for {
		n, err := e.tunFile.Read(buf)
		if err != nil {
			e.logger.Verbosef("steering: tun read stopped: %s", err)
			return
		}
		pkt := buf[:n]
		if e.decide(pkt) == DecisionBypass {
			info, _ := ParsePacket(pkt)
			e.bypass.InjectInbound(pkt, info.IsIPv4)
		} else {
			if _, err := e.innerFile.Write(pkt); err != nil {
				e.logger.Verbosef("steering: inner write stopped: %s", err)
				return
			}
		}
	}
}

func (e *Engine) decide(pkt []byte) Decision {
	if !e.hasBypass {
		return DecisionTunnel
	}
	info, ok := ParsePacket(pkt)
	if !ok {
		return DecisionTunnel // non-TCP/UDP (e.g. ICMP): fail closed
	}
	// Without underlying resolvers, excluded DNS must use the tunnel resolver.
	if info.Key.Proto == ProtoUDP && info.Key.Dst.Port() == dnsPort && !e.dnsDirect {
		return DecisionTunnel
	}
	if d, ok := e.flows.Lookup(info.Key); ok {
		return d
	}
	d := e.classify.Decide(info.Key)
	e.flows.Insert(info.Key, d)
	return d
}

// runDownstream pumps tunnel responses back to the TUN.
func (e *Engine) runDownstream() {
	defer e.waitGroup.Done()
	buf := make([]byte, maxPacketSize)
	for {
		n, err := e.innerFile.Read(buf)
		if err != nil {
			e.logger.Verbosef("steering: inner read stopped: %s", err)
			return
		}
		e.writeToTun(buf[:n])
	}
}
```

- [ ] **Step 4: Run tests, verify pass** — `cd wireguard/libwg && go test ./steering/...`.

- [ ] **Step 5: Add cgo exports** in `wireguard/libwg/steering_android.go`:

```go
/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

//go:build android

package main

/*
#include <stdint.h>
#include <stdlib.h>

typedef void (*steering_protect_fn)(void *ctx, int32_t fd);
typedef int32_t (*steering_owner_uid_fn)(void *ctx, int32_t protocol, const char *src, const char *dst);

static void call_steering_protect(steering_protect_fn fn, void *ctx, int32_t fd) {
	fn(ctx, fd);
}
static int32_t call_steering_owner_uid(steering_owner_uid_fn fn, void *ctx, int32_t protocol, const char *src, const char *dst) {
	return fn(ctx, protocol, src, dst);
}
*/
import "C"

import (
	"net/netip"
	"strings"
	"unsafe"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/steering"
)

var steeringEngines = container.New[*steering.Engine]()

//export steeringTurnOn
func steeringTurnOn(tunFd int32, innerFd int32, mtu int32,
	excludedUids *C.uint32_t, uidCount int32,
	dnsServers *C.char,
	protectCb C.steering_protect_fn, ownerUidCb C.steering_owner_uid_fn, cbCtx unsafe.Pointer,
	logSink LogSink, logContext LogContext) int32 {

	logger := logging.NewLogger(logSink, logContext)

	var uids []uint32
	if excludedUids != nil && uidCount > 0 {
		uids = append(uids, unsafe.Slice((*uint32)(unsafe.Pointer(excludedUids)), int(uidCount))...)
	}

	var dns []netip.Addr
	if dnsServers != nil {
		for _, s := range strings.Split(C.GoString(dnsServers), ",") {
			if addr, err := netip.ParseAddr(strings.TrimSpace(s)); err == nil {
				dns = append(dns, addr)
			}
		}
	}

	cb := steering.Callbacks{
		Protect: func(fd int32) {
			C.call_steering_protect(protectCb, cbCtx, C.int32_t(fd))
		},
		OwnerUID: func(proto steering.Proto, src, dst netip.AddrPort) int32 {
			cSrc := C.CString(src.String())
			cDst := C.CString(dst.String())
			defer C.free(unsafe.Pointer(cSrc))
			defer C.free(unsafe.Pointer(cDst))
			return int32(C.call_steering_owner_uid(ownerUidCb, cbCtx, C.int32_t(proto), cSrc, cDst))
		},
	}

	engine, err := steering.Start(int(tunFd), int(innerFd), steering.Config{
		ExcludedUIDs:  uids,
		UnderlyingDNS: dns,
		MTU:           int(mtu),
	}, cb, logger)
	if err != nil {
		logger.Errorf("steeringTurnOn: %s", err)
		return ERROR_GENERAL_FAILURE
	}

	handle, err := steeringEngines.Insert(engine)
	if err != nil {
		logger.Errorf("steeringTurnOn: %s", err)
		engine.Stop()
		return ERROR_GENERAL_FAILURE
	}
	return handle
}

//export steeringTurnOff
func steeringTurnOff(handle int32) {
	engine, err := steeringEngines.Remove(handle)
	if err != nil {
		return
	}
	(*engine).Stop()
}
```

Check `container.Container`'s method set matches usage in `netstack.go:28-36` (`Insert`/`Get`/`Remove`); mirror exactly.

- [ ] **Step 6: Compile-check the android build** — `cd nym-vpn-core && make -f Android.mk libwg` (requires NDK env; if unavailable locally, `cd wireguard/libwg && GOOS=android GOARCH=arm64 CGO_ENABLED=1 go vet ./...` as a fallback syntax check and flag the gap in the task report).

- [ ] **Step 7: Report changed files.**

---

### Task 5: Rust bindings for the steering engine (`nym-wg-go`)

**Files:**
- Create: `nym-vpn-core/crates/nym-wg-go/src/steering.rs`
- Modify: `nym-vpn-core/crates/nym-wg-go/src/lib.rs` (add `#[cfg(target_os = "android")] pub mod steering;`)
- Test: inline `#[cfg(test)]` in `steering.rs` + a host-target socketpair/tun-crate spike test in `nym-vpn-core/crates/nym-vpn-lib/src/tunnel_state_machine/tunnel/wireguard/dns_filter_proxy.rs`'s sibling (see Step 1)

**Interfaces:**
- Consumes: cgo exports from Task 4 (`steeringTurnOn`, `steeringTurnOff` — extern "C" declarations mirror `wireguard_go.rs:526-533` style).
- Produces (consumed by Task 7):
  ```rust
  pub trait SteeringCallbacks: Send + Sync + 'static {
      fn protect(&self, fd: RawFd);
      /// protocol: 6 = TCP, 17 = UDP. Return the owning UID or -1.
      fn owner_uid(&self, protocol: i32, src: &str, dst: &str) -> i32;
  }
  pub struct SteeringConfig {
      pub mtu: u16,
      pub excluded_uids: Vec<u32>,
      pub underlying_dns: Vec<IpAddr>,
  }
  pub struct Steering { /* handle + leaked ctx */ }
  impl Steering {
      /// Consumes the real tun fd; returns the steering handle and the outer
      /// socketpair fd that replaces the tun device for downstream consumers.
      pub fn start(tun_fd: OwnedFd, config: SteeringConfig, callbacks: Arc<dyn SteeringCallbacks>) -> Result<(Self, OwnedFd), Error>;
      pub fn stop(self);
  }
  ```

- [ ] **Step 1: De-risk the fd substitution first.** Downstream (Task 7) wraps the outer socketpair end with `tun::create_as_async(raw_fd)` — same as `tunnel_monitor.rs:1974-1988` does with the real tun fd. Verify the `tun` crate accepts a `SOCK_DGRAM` socketpair fd on a host target before building everything on it. Write a host test (this one runs on Linux, not Android — the `tun` crate's fd-wrapping path is shared):

In `nym-vpn-core/crates/nym-vpn-lib/src/tunnel_state_machine/tunnel_monitor.rs`, add at the bottom:

```rust
#[cfg(all(test, target_os = "linux"))]
mod tun_over_socketpair_tests {
    use std::os::fd::{AsRawFd, IntoRawFd};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// The Android steering engine substitutes the TUN fd with one end of a
    /// SOCK_DGRAM socketpair (see steering.rs). This test pins the `tun`
    /// crate's ability to do raw I/O over such an fd.
    #[tokio::test]
    async fn tun_async_device_works_over_socketpair() {
        let (a, b) = std::os::unix::net::UnixDatagram::pair().unwrap();
        a.set_nonblocking(true).unwrap();
        b.set_nonblocking(true).unwrap();

        let mut config = tun::Configuration::default();
        config.raw_fd(a.as_raw_fd());
        let mut device = tun::create_as_async(&config).expect("tun crate must accept socketpair fd");
        let _ = a.into_raw_fd(); // device owns it now

        let b = tokio::net::UnixDatagram::from_std(b).unwrap();
        b.send(b"ping").await.unwrap();
        let mut buf = [0u8; 16];
        let n = device.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"ping");

        device.write_all(b"pong").await.unwrap();
        let mut buf2 = [0u8; 16];
        let n2 = b.recv(&mut buf2).await.unwrap();
        assert_eq!(&buf2[..n2], b"pong");
    }
}
```

Run: `cd nym-vpn-core && cargo test -p nym-vpn-lib tun_over_socketpair`. **If this fails** (the tun crate probes the fd), STOP and change the Task 7 approach: instead of returning an `AsyncDevice`, `create_tun_device` gains an enum return wrapping either an `AsyncDevice` or a `tokio::net::UnixDatagram` adapter — raise this to the user before continuing, since it widens Task 7's diff.

- [ ] **Step 2: Write the binding** in `nym-vpn-core/crates/nym-wg-go/src/steering.rs` (whole file):

```rust
// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Bindings for the libwg steering engine: routes split-tunnel-excluded apps'
//! traffic directly (bypassing the tunnel) so they keep connectivity under
//! Android VPN lockdown.

use std::{
    ffi::{CStr, CString, c_char, c_void},
    net::IpAddr,
    os::fd::{IntoRawFd, OwnedFd, RawFd},
    sync::Arc,
};

use crate::{Error, LoggingCallback, Result};

pub trait SteeringCallbacks: Send + Sync + 'static {
    fn protect(&self, fd: RawFd);
    /// protocol: 6 = TCP, 17 = UDP. src/dst are "ip:port" ("[ip]:port" for v6).
    /// Return the owning UID or -1 when unknown.
    fn owner_uid(&self, protocol: i32, src: &str, dst: &str) -> i32;
}

pub struct SteeringConfig {
    pub mtu: u16,
    pub excluded_uids: Vec<u32>,
    pub underlying_dns: Vec<IpAddr>,
}

struct CallbackCtx {
    callbacks: Arc<dyn SteeringCallbacks>,
}

unsafe extern "C" fn protect_trampoline(ctx: *mut c_void, fd: i32) {
    let ctx = unsafe { &*(ctx as *const CallbackCtx) };
    ctx.callbacks.protect(fd);
}

unsafe extern "C" fn owner_uid_trampoline(
    ctx: *mut c_void,
    protocol: i32,
    src: *const c_char,
    dst: *const c_char,
) -> i32 {
    let ctx = unsafe { &*(ctx as *const CallbackCtx) };
    let src = unsafe { CStr::from_ptr(src) }.to_string_lossy();
    let dst = unsafe { CStr::from_ptr(dst) }.to_string_lossy();
    ctx.callbacks.owner_uid(protocol, &src, &dst)
}

pub struct Steering {
    handle: i32,
    // Kept alive for the lifetime of the engine; freed on stop().
    ctx: *mut CallbackCtx,
}

// SAFETY: the ctx pointer is only dereferenced by the Go engine's callbacks,
// which are themselves Send + Sync.
unsafe impl Send for Steering {}
unsafe impl Sync for Steering {}

impl Steering {
    /// Start the steering engine. Consumes `tun_fd` (the real TUN device);
    /// returns the engine plus the outer socketpair fd that downstream code
    /// must use in place of the TUN device.
    pub fn start(
        tun_fd: OwnedFd,
        config: SteeringConfig,
        callbacks: Arc<dyn SteeringCallbacks>,
    ) -> Result<(Self, OwnedFd)> {
        let (outer, inner) = std::os::unix::net::UnixDatagram::pair()
            .map_err(|_| Error::FailedToStartTunnel(crate::ERROR_GENERAL_FAILURE))?;

        let dns_csv = config
            .underlying_dns
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let dns_cstr = CString::new(dns_csv).expect("no NUL in IP list");

        let ctx = Box::into_raw(Box::new(CallbackCtx { callbacks }));

        let handle = unsafe {
            steeringTurnOn(
                tun_fd.into_raw_fd(),
                inner.into_raw_fd(),
                i32::from(config.mtu),
                config.excluded_uids.as_ptr(),
                config.excluded_uids.len() as i32,
                dns_cstr.as_ptr(),
                protect_trampoline,
                owner_uid_trampoline,
                ctx as *mut c_void,
                crate::logging::wg_logger_callback as *const c_void,
                std::ptr::null_mut(),
            )
        };
        if handle < 0 {
            // Engine did not start: reclaim the ctx to avoid leaking it.
            drop(unsafe { Box::from_raw(ctx) });
            return Err(Error::FailedToStartTunnel(handle));
        }
        Ok((Self { handle, ctx }, OwnedFd::from(outer)))
    }

    pub fn stop(self) {
        unsafe {
            steeringTurnOff(self.handle);
            // Safe to free only after the engine (and all its callback users)
            // has fully stopped, which steeringTurnOff guarantees.
            drop(Box::from_raw(self.ctx));
        }
        std::mem::forget(self);
    }
}

impl Drop for Steering {
    fn drop(&mut self) {
        unsafe {
            steeringTurnOff(self.handle);
            drop(Box::from_raw(self.ctx));
        }
    }
}

unsafe extern "C" {
    unsafe fn steeringTurnOn(
        tun_fd: i32,
        inner_fd: i32,
        mtu: i32,
        excluded_uids: *const u32,
        uid_count: i32,
        dns_servers: *const c_char,
        protect_cb: unsafe extern "C" fn(*mut c_void, i32),
        owner_uid_cb: unsafe extern "C" fn(*mut c_void, i32, *const c_char, *const c_char) -> i32,
        cb_ctx: *mut c_void,
        log_sink: *const c_void,
        log_context: *mut c_void,
    ) -> i32;
    unsafe fn steeringTurnOff(handle: i32);
}
```

**Adapt the following to the crate's actual local idioms** (read them before writing): the error type/variants (`crate::Error` — reuse whatever `netstack.rs`/`wireguard_go.rs` use for negative Go return codes), and the logger plumbing (`wg_logger_callback` name and how `netstack.rs:95` passes `logSink`/`logContext` — copy that exact pattern, including any `LoggingContext` registration).

- [ ] **Step 3: Compile** — `cd nym-vpn-core && cargo check -p nym-wg-go --target aarch64-linux-android` (needs NDK toolchain configured as for `make -f Android.mk`; fallback: gate the module `#[cfg(target_os = "android")]` and run plain `cargo check -p nym-wg-go` to at least validate the non-android surface, flagging the gap).

- [ ] **Step 4: Report changed files.**

---

### Task 6: Extend `AndroidTunProvider` with connection-owner lookup

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib/src/tunnel_provider/mod.rs:21-25`
- Modify: `nym-vpn-core/crates/nym-vpn-lib-uniffi/src/tunnel_provider/android.rs`
- Modify: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/backend/service/VpnService.kt` (implements the uniffi interface)
- Create: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/util/ConnectionOwnerResolver.kt`
- Test: `nym-vpn-android/core/src/test/java/net/nymtech/vpn/util/ConnectionOwnerResolverTest.kt`

**Interfaces:**
- Produces (Rust trait additions, consumed by Task 7):
  - `nym-vpn-lib`: `fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32;`
  - uniffi trait (same signature) + adapter forwarding in `AndroidTunProviderImpl`.
- Produces (Kotlin): `object ConnectionOwnerResolver { fun parseAddrPort(s: String): InetSocketAddress?; fun lookup(connectivityManager: ConnectivityManager, protocol: Int, source: String, destination: String): Int }` returning `-1` on any failure.

- [ ] **Step 1: Add the method to both Rust traits.**

`nym-vpn-lib/src/tunnel_provider/mod.rs`:

```rust
#[cfg(target_os = "android")]
pub trait AndroidTunProvider: Send + Sync + std::fmt::Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(&self, config: TunnelSettings) -> std::io::Result<std::os::fd::RawFd>;
    /// Resolve the UID owning a connection. protocol: 6 = TCP, 17 = UDP;
    /// source/destination: "ip:port" ("[ip]:port" for IPv6). Returns -1 when
    /// the owner cannot be determined.
    fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32;
}
```

`nym-vpn-lib-uniffi/src/tunnel_provider/android.rs` — add to the `#[uniffi::export(with_foreign)]` trait:

```rust
    /// Resolve the UID owning a connection (ConnectivityManager.getConnectionOwnerUid).
    /// Returns -1 when unknown. protocol: 6 = TCP, 17 = UDP.
    fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32;
```

and forward it in `impl nym_vpn_lib::tunnel_provider::AndroidTunProvider for AndroidTunProviderImpl`:

```rust
    fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32 {
        self.inner.get_connection_owner_uid(protocol, source, destination)
    }
```

- [ ] **Step 2: Regenerate Kotlin bindings** — `cd nym-vpn-core && make -f Android.mk uniffi` (this also rebuilds the Rust libs; requires NDK). Verify the regenerated `nym_vpn_lib` Kotlin interface now declares `getConnectionOwnerUid`.

- [ ] **Step 3: Write failing Kotlin test** for the parser (pure function, no Android deps needed for `parseAddrPort` if implemented with manual parsing):

```kotlin
package net.nymtech.vpn.util

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ConnectionOwnerResolverTest {
	@Test
	fun `parses ipv4 addr port`() {
		val parsed = ConnectionOwnerResolver.parseAddrPort("10.0.0.2:443")!!
		assertEquals("10.0.0.2", parsed.hostString)
		assertEquals(443, parsed.port)
	}

	@Test
	fun `parses bracketed ipv6 addr port`() {
		val parsed = ConnectionOwnerResolver.parseAddrPort("[fd00::1]:53")!!
		assertEquals("fd00::1", parsed.hostString)
		assertEquals(53, parsed.port)
	}

	@Test
	fun `rejects garbage`() {
		assertNull(ConnectionOwnerResolver.parseAddrPort("not-an-address"))
		assertNull(ConnectionOwnerResolver.parseAddrPort("10.0.0.2"))
		assertNull(ConnectionOwnerResolver.parseAddrPort("[fd00::1]:notaport"))
	}
}
```

Run: `cd nym-vpn-android && ./gradlew :core:testDebugUnitTest --tests '*ConnectionOwnerResolverTest*'` → FAIL (class missing).

- [ ] **Step 4: Implement** `ConnectionOwnerResolver.kt`:

```kotlin
package net.nymtech.vpn.util

import android.net.ConnectivityManager
import android.os.Build
import java.net.InetAddress
import java.net.InetSocketAddress
import timber.log.Timber

object ConnectionOwnerResolver {
	private const val INVALID_UID = -1
	private const val TAG = "core-vpn"

	fun parseAddrPort(s: String): InetSocketAddress? {
		return try {
			val (host, port) = if (s.startsWith("[")) {
				val end = s.indexOf(']')
				if (end == -1 || s.getOrNull(end + 1) != ':') return null
				s.substring(1, end) to s.substring(end + 2)
			} else {
				val sep = s.lastIndexOf(':')
				if (sep == -1) return null
				s.substring(0, sep) to s.substring(sep + 1)
			}
			val portNum = port.toIntOrNull() ?: return null
			// createUnresolved avoids DNS; the string is always a literal IP.
			InetSocketAddress(InetAddress.getByName(host), portNum)
		} catch (_: Exception) {
			null
		}
	}

	fun lookup(connectivityManager: ConnectivityManager, protocol: Int, source: String, destination: String): Int {
		if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return INVALID_UID
		val src = parseAddrPort(source) ?: return INVALID_UID
		val dst = parseAddrPort(destination) ?: return INVALID_UID
		return try {
			connectivityManager.getConnectionOwnerUid(protocol, src, dst)
		} catch (e: SecurityException) {
			Timber.tag(TAG).w(e, "getConnectionOwnerUid denied")
			INVALID_UID
		} catch (e: Exception) {
			Timber.tag(TAG).w(e, "getConnectionOwnerUid failed")
			INVALID_UID
		}
	}
}
```

Note: `InetAddress.getByName` with a literal IP does not hit DNS, but to be belt-and-braces prefer `InetAddresses.parseNumericAddress(host)` (android.net, API 29 — fine because `lookup` already guards on Q) inside `lookup`, keeping `getByName` only in the JVM-unit-testable `parseAddrPort`.

In `VpnService.kt`, next to `override fun bypass(socket: Int)` at `:222`:

```kotlin
	override fun getConnectionOwnerUid(protocol: Int, source: String, destination: String): Int =
		ConnectionOwnerResolver.lookup(
			getSystemService(ConnectivityManager::class.java),
			protocol,
			source,
			destination,
		)
```

- [ ] **Step 5: Run tests + compile** — `./gradlew :core:testDebugUnitTest --tests '*ConnectionOwnerResolverTest*' :core:compileDebugKotlin`. Expected: PASS.

- [ ] **Step 6: Report changed files.**

---

### Task 7: Rust plumbing — `SetAppBypass` command → steering start in `create_tun_device`

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs` (command enum ~`:107`, dispatch ~`:1052`, handler ~`:1390`)
- Modify: the config manager the handlers use (follow `set_enable_two_hop` — same file/module)
- Modify: `nym-vpn-core/crates/nym-vpn-lib/src/tunnel_state_machine/mod.rs` + `tunnel_monitor.rs:1956-1994` (`create_tun_device`)
- Modify: `nym-vpn-core/crates/nym-vpn-lib-uniffi/src/vpn_service_command_sender.rs` (~`:128`)
- Modify: `nym-vpn-core/crates/nym-vpn-lib-uniffi/src/lib.rs` or its types module (uniffi record)

**Interfaces:**
- Consumes: `nym_wg_go::steering::{Steering, SteeringConfig, SteeringCallbacks}` (Task 5), `AndroidTunProvider::get_connection_owner_uid` + `bypass` (Task 6).
- Produces (consumed by Task 8 from Kotlin): `suspend fun setAppBypass(config: AppBypassConfig?)` on `NymVpnServiceCommandSender`, with uniffi record:
  ```rust
  #[derive(uniffi::Record, Clone, Debug)]
  pub struct AppBypassConfig {
      pub excluded_uids: Vec<u32>,
      pub underlying_dns: Vec<String>,
  }
  ```

- [ ] **Step 1: Define the core type** in `nym-vpn-lib` (put it next to `TunnelSettings` in `tunnel_provider/mod.rs`):

```rust
/// Per-connection app-bypass ("steering") configuration for Android lockdown
/// mode. `None`/absent means steering is off and classic per-app exclusion
/// (VpnService.Builder.addDisallowedApplication) is in effect.
#[cfg(target_os = "android")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppBypassConfig {
    pub excluded_uids: Vec<u32>,
    pub underlying_dns: Vec<std::net::IpAddr>,
}
```

- [ ] **Step 2: Add the command.** In `vpn_service.rs`, mirroring `SetEnableTwoHop` exactly:
  - enum variant: `SetAppBypass(oneshot::Sender<()>, Option<crate::tunnel_provider::AppBypassConfig>)` (android-gated with `#[cfg(target_os = "android")]`, or unconditional with the type moved out of the cfg if the enum is not cfg-friendly — match how other android-only commands are handled in this enum; if none exist, keep it unconditional and make `AppBypassConfig` unconditional too, unused off-android).
  - dispatch arm (next to `:1052`):
    ```rust
    VpnServiceCommand::SetAppBypass(tx, config) => {
        self.handle_set_app_bypass(config).await;
        let _ = tx.send(());
    }
    ```
  - handler (next to `:1390`):
    ```rust
    async fn handle_set_app_bypass(&mut self, config: Option<AppBypassConfig>) {
        self.config_manager.set_app_bypass(config).await;
        self.update_tunnel_settings_with_throttle();
    }
    ```
  - Add `set_app_bypass` to the config manager following `set_enable_two_hop`'s implementation (same file or `config_manager` module — find it via `grep -n "fn set_enable_two_hop" nym-vpn-core/crates/nym-vpn-lib/src/`), storing the value in the same config struct the tunnel state machine reads. This is runtime tunnel config, not user-persisted settings — if `set_enable_two_hop` persists to disk, store app-bypass in the in-memory portion only (Kotlin re-sends it on every connect); check how `update_tunnel_settings_with_throttle()` propagates values into `tunnel_state_machine` options and thread `app_bypass` the same way `enable_ad_blocking`/`dns_filter` reaches `tunnel_monitor` (anchor: `tunnel_state_machine/mod.rs:1200`).

- [ ] **Step 3: Start steering in `create_tun_device`** (`tunnel_monitor.rs:1956-1994`). The android branch becomes:

```rust
    #[cfg(target_os = "android")]
    let owned_tun_fd = {
        let raw_tun_fd = self
            .tun_provider
            .configure_tunnel(packet_tunnel_settings)
            .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?;
        let real_tun_fd = unsafe { OwnedFd::from_raw_fd(raw_tun_fd) };

        match self.app_bypass_config() {
            Some(app_bypass) => {
                let callbacks = Arc::new(TunProviderSteeringCallbacks {
                    tun_provider: self.tun_provider.clone(),
                });
                let (steering, outer_fd) = nym_wg_go::steering::Steering::start(
                    real_tun_fd,
                    nym_wg_go::steering::SteeringConfig {
                        mtu: packet_tunnel_settings_mtu,
                        excluded_uids: app_bypass.excluded_uids.clone(),
                        underlying_dns: app_bypass.underlying_dns.clone(),
                    },
                    callbacks,
                )
                .map_err(Error::StartSteering)?;
                self.store_steering_handle(steering);
                outer_fd
            }
            None => real_tun_fd,
        }
    };
```

with the callback adapter:

```rust
#[cfg(target_os = "android")]
#[derive(Debug)]
struct TunProviderSteeringCallbacks {
    tun_provider: std::sync::Arc<dyn crate::tunnel_provider::AndroidTunProvider>,
}

#[cfg(target_os = "android")]
impl nym_wg_go::steering::SteeringCallbacks for TunProviderSteeringCallbacks {
    fn protect(&self, fd: std::os::fd::RawFd) {
        self.tun_provider.bypass(fd);
    }
    fn owner_uid(&self, protocol: i32, src: &str, dst: &str) -> i32 {
        self.tun_provider
            .get_connection_owner_uid(protocol, src.to_owned(), dst.to_owned())
    }
}
```

Concrete sub-steps (resolve against real code while implementing, all in `tunnel_monitor.rs`):
  - `app_bypass_config()`: read from wherever the monitor already reads per-connection options (same source as the DNS/mtu settings used at `:1071-1092` and `:1843-1846`); capture `mtu` before `packet_tunnel_settings` is moved.
  - `store_steering_handle`: add `steering: Option<nym_wg_go::steering::Steering>` to the monitor struct; drop/`stop()` it in the same teardown path where the tunnel is shut down (mirror how `proxy_join_handle` from `connected_tunnel.rs:361-379` is awaited/cleaned at `:509-524` — steering out-lives the wg tunnel, so stop it after downstream consumers have stopped).
  - New `Error::StartSteering(nym_wg_go::Error)` variant in this module's error enum.
  - Add `nym-wg-go` steering import; it is already a dependency of `nym-vpn-lib`.

- [ ] **Step 4: uniffi surface.** In `vpn_service_command_sender.rs` (next to `:128`):

```rust
    pub async fn set_app_bypass(&self, config: Option<AppBypassConfig>) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::SetAppBypass, config.map(Into::into))
            .await
    }
```

Define the uniffi record + conversion where the other uniffi request types live (grep `uniffi::Record` in `nym-vpn-lib-uniffi/src/`):

```rust
#[derive(uniffi::Record, Clone, Debug)]
pub struct AppBypassConfig {
    pub excluded_uids: Vec<u32>,
    pub underlying_dns: Vec<String>,
}

impl From<AppBypassConfig> for nym_vpn_lib::tunnel_provider::AppBypassConfig {
    fn from(c: AppBypassConfig) -> Self {
        Self {
            excluded_uids: c.excluded_uids,
            underlying_dns: c
                .underlying_dns
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect(),
        }
    }
}
```

- [ ] **Step 5: Compile + regen** — `cd nym-vpn-core && cargo check -p nym-vpn-lib -p nym-vpn-lib-uniffi` (desktop check catches everything non-android-gated), then `make -f Android.mk` for the android build + `make -f Android.mk uniffi` to regenerate Kotlin bindings (Kotlin now sees `setAppBypass` and `AppBypassConfig`).

- [ ] **Step 6: Run the spike test still passing** — `cargo test -p nym-vpn-lib tun_over_socketpair`.

- [ ] **Step 7: Report changed files.**

---

### Task 8: Kotlin — lockdown detection, UID/DNS resolution, command wiring, builder switch

**Files:**
- Create: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/util/AppBypassResolver.kt`
- Modify: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/backend/controller/VpnTunController.kt`
- Modify: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/backend/controller/VpnCoreController.kt` (`applyConfigDiffToSender` `:403-427`, `syncLocalFieldsFromConfig` `:441-449`)
- Test: `nym-vpn-android/core/src/test/java/net/nymtech/vpn/util/AppBypassResolverTest.kt`

**Interfaces:**
- Consumes: `sender.setAppBypass(AppBypassConfig?)` + `nym_vpn_lib.AppBypassConfig` (Task 7 codegen).
- Produces:
  - `object AppBypassResolver`:
    - `fun shouldSteer(sdkInt: Int, lockdownEnabled: Boolean, restrictedApps: List<String>): Boolean` (pure, unit-tested)
    - `fun resolveUids(packageManager: PackageManager, packages: List<String>): List<UInt>`
    - `fun underlyingDnsServers(connectivityManager: ConnectivityManager): List<String>`
  - `VpnTunController.setAppBypassActive(active: Boolean)` — when true, `configureTunnel` skips the `addDisallowedApplication` loop.

- [ ] **Step 1: Write failing tests** for the decision matrix:

```kotlin
package net.nymtech.vpn.util

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class AppBypassResolverTest {
	private val apps = listOf("com.example.app")

	@Test
	fun `steers when lockdown and exclusions on Q+`() {
		assertTrue(AppBypassResolver.shouldSteer(sdkInt = 29, lockdownEnabled = true, restrictedApps = apps))
	}

	@Test
	fun `no steering without lockdown`() {
		assertFalse(AppBypassResolver.shouldSteer(sdkInt = 34, lockdownEnabled = false, restrictedApps = apps))
	}

	@Test
	fun `no steering with empty exclusion list`() {
		assertFalse(AppBypassResolver.shouldSteer(sdkInt = 34, lockdownEnabled = true, restrictedApps = emptyList()))
	}

	@Test
	fun `no steering below api 29`() {
		assertFalse(AppBypassResolver.shouldSteer(sdkInt = 28, lockdownEnabled = true, restrictedApps = apps))
	}
}
```

Run: `./gradlew :core:testDebugUnitTest --tests '*AppBypassResolverTest*'` → FAIL.

- [ ] **Step 2: Implement** `AppBypassResolver.kt`:

```kotlin
package net.nymtech.vpn.util

import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import timber.log.Timber

object AppBypassResolver {
	private const val TAG = "core-vpn"

	fun shouldSteer(sdkInt: Int, lockdownEnabled: Boolean, restrictedApps: List<String>): Boolean =
		sdkInt >= Build.VERSION_CODES.Q && lockdownEnabled && restrictedApps.isNotEmpty()

	fun resolveUids(packageManager: PackageManager, packages: List<String>): List<UInt> =
		packages.mapNotNull { pkg ->
			runCatching { packageManager.getApplicationInfo(pkg, 0).uid.toUInt() }
				.onFailure { Timber.tag(TAG).w("app-bypass: package not found: %s", pkg) }
				.getOrNull()
		}.distinct()

	/** DNS servers of a non-VPN network with validated internet, as IP strings. */
	fun underlyingDnsServers(connectivityManager: ConnectivityManager): List<String> {
		return connectivityManager.allNetworks.asSequence()
			.mapNotNull { network ->
				val caps = connectivityManager.getNetworkCapabilities(network) ?: return@mapNotNull null
				if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) return@mapNotNull null
				if (!caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)) return@mapNotNull null
				connectivityManager.getLinkProperties(network)?.dnsServers
			}
			.firstOrNull { it.isNotEmpty() }
			?.map { it.hostAddress ?: "" }
			?.filter { it.isNotEmpty() }
			.orEmpty()
	}
}
```

(`allNetworks` is deprecated but remains the correct enumeration for "some other network than my VPN"; suppress the deprecation warning.)

- [ ] **Step 3: Run tests, verify pass.**

- [ ] **Step 4: Builder switch.** In `VpnTunController.kt`:

```kotlin
	@Volatile private var appBypassActive: Boolean = false

	fun setAppBypassActive(active: Boolean) {
		appBypassActive = active
	}
```

and in `configureTunnel` replace the loop at `:37-39`:

```kotlin
			if (appBypassActive) {
				Timber.tag(TAG).i("App bypass active (lockdown): steering excluded apps in-tunnel")
			} else {
				disallowedApps.forEach { pkg ->
					runCatching { builder.addDisallowedApplication(pkg) }
				}
			}
```

- [ ] **Step 5: Command wiring.** In `VpnCoreController.kt`:

Add a helper (near `syncLocalFieldsFromConfig`):

```kotlin
	private fun computeAppBypass(cfg: CoreVpnConfig): nym_vpn_lib.AppBypassConfig? {
		val lockdown = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q && service.isLockdownEnabled
		if (!AppBypassResolver.shouldSteer(Build.VERSION.SDK_INT, lockdown, cfg.restrictedApps)) return null
		return nym_vpn_lib.AppBypassConfig(
			excludedUids = AppBypassResolver.resolveUids(service.packageManager, cfg.restrictedApps),
			underlyingDns = AppBypassResolver.underlyingDnsServers(service.getSystemService(ConnectivityManager::class.java)),
		)
	}
```

In `syncLocalFieldsFromConfig(cfg)` (`:441-449`), after `tun.setDisallowedApps(...)`:

```kotlin
		tun.setAppBypassActive(computeAppBypass(cfg) != null)
```

In `applyConfigDiffToSender` (`:403-427`), add (always re-send — lockdown state can change outside our process, and the call is cheap):

```kotlin
		sender.setAppBypass(computeAppBypass(cfg))
```

Note the field name Kotlin uses for `VpnService.isLockdownEnabled` (method `isLockdownEnabled()` surfaces as property `isLockdownEnabled`); it must be called on the `VpnService` instance — `service` here is exactly that (check the constructor property type of `VpnCoreController` and adjust the accessor if it's typed as the app's `VpnService` subclass, which inherits it).

Reconnect-on-lockdown-change is intentionally out of scope: steering config is captured per connect; the existing `tunSettingsChanged` reconnect at `:385-400` already fires when the app list changes. Document in the task report that toggling lockdown mid-connection requires a reconnect (Android itself restarts always-on VPNs when lockdown is toggled, which covers the common path).

- [ ] **Step 6: Compile + full unit tests** — `./gradlew :core:compileDebugKotlin :core:testDebugUnitTest`.

- [ ] **Step 7: Report changed files.**

---

### Task 9: UI — lockdown notes and warnings

**Files:**
- Modify: `nym-vpn-android/app/src/main/res/values/strings.xml`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/settings/tunneling/SplitTunnelingScreen.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/settings/tunneling/SplitTunnelingViewModel.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/settings/tunneling/components/SplitTunnelingInfoModal.kt`

**Interfaces:**
- Consumes: `AppBypassResolver.shouldSteer` semantics (display-only mirror; the app module can't call core's `isLockdownEnabled` — see Step 2).
- Produces: UI state field `lockdownState: LockdownState` (`enum class LockdownState { OFF, ACTIVE_STEERING, UNSUPPORTED_API }`) on `SplitTunnelingUiState`.

- [ ] **Step 1: Add strings** to `strings.xml`:

```xml
	<string name="split_tunnel_lockdown_active_note">"Block connections without VPN" is on. Excluded apps connect directly to the internet outside the VPN tunnel, from your real IP address. Ping (ICMP) does not work for excluded apps."</string>
	<string name="split_tunnel_lockdown_legacy_warning">Your Android version blocks excluded apps when "Block connections without VPN" is enabled in system settings. Disable it there or remove exclusions.</string>
	<string name="split_tunnel_open_vpn_settings">Open VPN settings</string>
```

- [ ] **Step 2: Detect lockdown in the app module.** The app process cannot call `VpnService.isLockdownEnabled` (it's a method on the running service instance, in the `:core` service process). Read it the same way `GeneralExtensions.kt:25-32` reads always-on state:

```kotlin
private const val ALWAYS_ON_VPN_LOCKDOWN = "always_on_vpn_lockdown"

fun isVpnLockdownEnabled(context: Context): Boolean = try {
	Settings.Secure.getInt(context.contentResolver, ALWAYS_ON_VPN_LOCKDOWN, 0) == 1 &&
		context.isVpnAlwaysOn()
} catch (_: Exception) {
	false
}
```

Add this next to `isVpnAlwaysOn` in `GeneralExtensions.kt`. (Reading `always_on_vpn_lockdown` may throw or return 0 on some OEMs — the catch keeps it display-safe; the authoritative gate for actual steering remains `isLockdownEnabled` in the service, Task 8.)

- [ ] **Step 3: Surface state.** In `SplitTunnelingViewModel`, compute on screen load:

```kotlin
	val lockdownState: LockdownState = when {
		!context.isVpnLockdownEnabled() -> LockdownState.OFF
		Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q -> LockdownState.ACTIVE_STEERING
		else -> LockdownState.UNSUPPORTED_API
	}
```

(inject `@ApplicationContext context` following the ViewModel's existing Hilt pattern), expose it on the UI state, and in `SplitTunnelingScreen.kt` render above the app list:
- `ACTIVE_STEERING` → informational card with `split_tunnel_lockdown_active_note`.
- `UNSUPPORTED_API` → warning card with `split_tunnel_lockdown_legacy_warning` + a button (`split_tunnel_open_vpn_settings`) launching `Intent(Settings.ACTION_VPN_SETTINGS)`.
- `OFF` → nothing.

Follow the screen's existing card/notification composable style (match whatever component `SplitTunnelingScreen.kt` already uses for info surfaces; reuse `components/StaticContent.kt` patterns).

- [ ] **Step 4: Privacy note.** Append a sentence to the existing info modal (`SplitTunnelingInfoModal.kt` / its strings): excluded apps' traffic leaves the device directly from the real IP and, under lockdown, is relayed by the NymVPN app process on-device without entering the tunnel.

- [ ] **Step 5: Compile + lint** — `./gradlew :app:compileDebugKotlin`; run existing Maestro flow file check (`nym-vpn-android/maestro/flows/settings_screen/split_tunelling.yaml`) mentally against the new cards — if the flow taps list items by index/position, update selectors accordingly.

- [ ] **Step 6: Report changed files.**

---

### Task 10: Fix stale exclusion list on the always-on boot path

**Files:**
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/manager/backend/ServiceBackedBackendManager.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/manager/backend/BackendManager.kt` (interface — add `suspend fun pushRestrictedApps()`)
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/settings/tunneling/SplitTunnelingViewModel.kt` (`saveChangesAndMaybeReconnect` `:110-139`)

**Why:** The core service's persisted copy (`KEY_RESTRICTED_APPS`) is only refreshed inside `startTunnel()`/`requestReconnect()` (`ServiceBackedBackendManager.kt:113-119,162-174`). The always-on boot path (`VpnService.kt:190-195` → `core.connectLocked()`) never passes through the app process, so it connects with whatever list was last pushed. Pushing on every save closes the gap.

- [ ] **Step 1: Add to `ServiceBackedBackendManager`:**

```kotlin
	override suspend fun pushRestrictedApps() {
		val restrictedApps = getRestrictedAppsPackages()
		serviceConnectionManager.withApi { api ->
			runCatching {
				api.applyUpdates(listOf(CoreVpnConfigUpdate.SetRestrictedApps(restrictedApps)))
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "push restricted apps failed")
			}
		}
	}
```

and the matching declaration in the `BackendManager` interface. (If `withApi` cannot bind because the service has never run, the push is a no-op — acceptable: the list is pushed again on first `startTunnel`.)

- [ ] **Step 2: Call it on save.** In `SplitTunnelingViewModel.saveChangesAndMaybeReconnect` (`:110-139`), after `splitTunnelingRepository.saveAppInfoList(toSave)`, add an unconditional `backendManager.pushRestrictedApps()` **before** the existing "reconnect if connected" logic (the reconnect path re-pushes anyway; the new call covers the disconnected case).

- [ ] **Step 3: Compile + tests** — `./gradlew :app:compileDebugKotlin :app:testDebugUnitTest`. If `BackendManager` has a fake/mock in tests, add the new member there.

- [ ] **Step 4: Report changed files.**

---

### Task 11: Full-build verification + manual test matrix

**Files:** none created; this task validates the whole feature.

- [ ] **Step 1: Full native build** — `cd nym-vpn-core && make -f Android.mk` (all three ABIs + uniffi bindings), then `cd ../nym-vpn-android && ./gradlew assembleDebug`.

- [ ] **Step 2: All unit tests** — `cd wireguard/libwg && go test ./steering/...`; `cd nym-vpn-core && cargo test -p nym-vpn-lib tun_over_socketpair && cargo check -p nym-wg-go -p nym-vpn-lib-uniffi`; `cd nym-vpn-android && ./gradlew :core:testDebugUnitTest :app:testDebugUnitTest`.

- [ ] **Step 3: Hand the manual matrix to the user** (requires physical devices; cannot be automated here). Print this checklist in the task report:

| # | Device | Lockdown | Mode | Check |
|---|--------|----------|------|-------|
| 1 | Stock Android 14+ | off | WG | Excluded app connects directly (unchanged behavior); tunneled app goes through VPN |
| 2 | Stock Android 14+ | on | WG | Excluded app browses (TCP), streams (QUIC/UDP), resolves DNS; `ping` from excluded app fails (expected) |
| 3 | Stock Android 14+ | on | Mixnet | Same as #2 |
| 4 | GrapheneOS (lockdown default) | on | WG + Mixnet | Same as #2/#3 |
| 5 | Any, API 29+ | on | any | Tunneled app's traffic still exits at the Nym exit (check IP) ; ad-block (WG mode) still works with steering active |
| 6 | Any | on | any | Disconnect tunnel → excluded app is blocked by OS (expected fail-closed) |
| 7 | Any | on | any | Split-tunneling screen shows the lockdown info card; info modal shows privacy note |
| 8 | API 24–28 device/emulator | on (system setting) | any | Warning card + working deep link to VPN settings; excluded apps blocked (documented) |
| 9 | Any, API 29+ | on | any | Leak check: `tcpdump` on the AP/router shows only tunnel-endpoint traffic + excluded app's flows; no other cleartext from the device |
| 10 | Any | on | any | Edit exclusion list while connected → reconnect fires (existing `:397-400` path) and new list takes effect |

- [ ] **Step 4: Report overall status and any deviations from the plan.**
