/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net"
	"net/netip"
	"os"
	"syscall"
	"testing"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"golang.org/x/sys/unix"
	"gvisor.dev/gvisor/pkg/tcpip"
	"gvisor.dev/gvisor/pkg/tcpip/header"
)

// nonLoopbackIPv4 returns a local, non-loopback IPv4 address usable as the
// "internet" side of a bypassed flow.
func nonLoopbackIPv4(t *testing.T) netip.Addr {
	t.Helper()
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		t.Fatal(err)
	}
	for _, a := range addrs {
		ipNet, ok := a.(*net.IPNet)
		if !ok {
			continue
		}
		ip, ok := netip.AddrFromSlice(ipNet.IP)
		if !ok {
			continue
		}
		ip = ip.Unmap()
		if ip.Is4() && !ip.IsLoopback() && !ip.IsLinkLocalUnicast() {
			return ip
		}
	}
	t.Skip("no non-loopback IPv4 address available to stand in for the internet")
	return netip.Addr{}
}

func buildIPv4TCP(src, dst netip.Addr, srcPort, dstPort uint16, flags header.TCPFlags) []byte {
	length := header.IPv4MinimumSize + header.TCPMinimumSize
	buf := make([]byte, length)
	ip := header.IPv4(buf)
	ip.Encode(&header.IPv4Fields{
		TotalLength: uint16(length),
		TTL:         64,
		Protocol:    uint8(header.TCPProtocolNumber),
		SrcAddr:     tcpip.AddrFrom4(src.As4()),
		DstAddr:     tcpip.AddrFrom4(dst.As4()),
	})
	ip.SetChecksum(^ip.CalculateChecksum())
	tcp := header.TCP(buf[header.IPv4MinimumSize:])
	tcp.Encode(&header.TCPFields{
		SrcPort:    srcPort,
		DstPort:    dstPort,
		SeqNum:     1000,
		DataOffset: header.TCPMinimumSize,
		Flags:      flags,
		WindowSize: 65535,
	})
	return buf
}

func socketPair(t *testing.T) (*os.File, *os.File) {
	t.Helper()
	fds, err := rawSocketPair(t)
	if err != nil {
		t.Fatal(err)
	}
	// Deliberately wrapped as-is (blocking, hence not poller-registered): these
	// tests hand the wrapped fd straight to Start via File.Fd(), and registering
	// the same fd with the poller twice makes the second os.File fall back to
	// unpollable mode. Tests that need working deadlines use pollableFile on a
	// dedicated fd instead.
	return os.NewFile(uintptr(fds[0]), "a"), os.NewFile(uintptr(fds[1]), "b")
}

func rawSocketPair(t *testing.T) ([2]int, error) {
	t.Helper()
	fds, err := syscall.Socketpair(syscall.AF_UNIX, syscall.SOCK_DGRAM, 0)
	if err != nil {
		return [2]int{}, err
	}
	return [2]int{fds[0], fds[1]}, nil
}

// pollableFile wraps fd in an os.File that is registered with the runtime
// poller, so SetReadDeadline actually works on it (os.NewFile only polls fds
// that already carry O_NONBLOCK). Used for the *test's* end of each socket
// pair; the engine's end is deliberately handed over raw and blocking.
func pollableFile(t *testing.T, fd int, name string) *os.File {
	t.Helper()
	if err := unix.SetNonblock(fd, true); err != nil {
		t.Fatal(err)
	}
	return os.NewFile(uintptr(fd), name)
}

func TestEnginePassthroughTunnelTraffic(t *testing.T) {
	tunA, tunB := socketPair(t)     // tunA = fake TUN device side, tunB = engine's tun fd
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

// TestEngineUnattributableFlowGoesToTunnel exercises the fail-closed path: an
// excluded UID list is configured (so the engine builds a bypass stack), but
// OwnerUID cannot attribute the flow to any UID (-1). The packet must still
// arrive on the tunnel side (innerB), never routed to the bypass netstack.
func TestEngineUnattributableFlowGoesToTunnel(t *testing.T) {
	tunA, tunB := socketPair(t)
	innerA, innerB := socketPair(t)
	defer tunA.Close()
	defer innerB.Close()

	logger := device.NewLogger(device.LogLevelError, "test")
	eng, err := Start(int(tunB.Fd()), int(innerA.Fd()),
		Config{MTU: 1500, ExcludedUIDs: []uint32{10123}},
		Callbacks{
			Protect:  func(int32) {},
			OwnerUID: func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 },
		},
		logger)
	if err != nil {
		t.Fatal(err)
	}
	defer eng.Stop()

	pkt := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), netip.MustParseAddr("1.2.3.4"), 1234, 4321)

	if _, err := tunA.Write(pkt); err != nil {
		t.Fatal(err)
	}
	buf := make([]byte, 2048)
	innerB.SetReadDeadline(time.Now().Add(2 * time.Second))
	n, err := innerB.Read(buf)
	if err != nil || n != len(pkt) {
		t.Fatalf("unattributable flow did not reach tunnel: n=%d err=%v", n, err)
	}
}

// TestEngineForwardsAfterIdleWithRawBlockingFds is the regression test for the
// "engine dies milliseconds after Start" blackhole.
//
// It differs from the other engine tests in two ways that both matter:
//
//  1. It hands Start the RAW socketpair fds, in their default BLOCKING state,
//     rather than fds obtained from an os.File. That is what the Rust/cgo
//     caller does in production, and it is the state os.NewFile cares about:
//     an fd that is not already O_NONBLOCK when wrapped never gets registered
//     with the runtime poller. If Start wraps first and calls SetNonblock
//     after, both pumps end up doing raw non-blocking reads on non-pollable
//     files, so the very first read of an empty TUN returns EAGAIN as a hard
//     error and both pumps exit.
//
//  2. It leaves the TUN idle for a moment before sending the first packet.
//     The idle gap is what forces that first would-block read to happen; with
//     traffic already queued the read could succeed by luck and hide the bug.
//
// After the idle gap, both directions must still forward.
func TestEngineForwardsAfterIdleWithRawBlockingFds(t *testing.T) {
	tunFds, err := rawSocketPair(t)
	if err != nil {
		t.Fatal(err)
	}
	innerFds, err := rawSocketPair(t)
	if err != nil {
		t.Fatal(err)
	}
	// Test-side ends only; the engine-side fds (tunFds[1], innerFds[0]) are
	// passed to Start untouched, i.e. blocking, and owned by it from then on.
	tunA := pollableFile(t, tunFds[0], "tun-peer")
	innerB := pollableFile(t, innerFds[1], "inner-peer")
	defer tunA.Close()
	defer innerB.Close()

	logger := device.NewLogger(device.LogLevelError, "test")
	eng, err := Start(tunFds[1], innerFds[0],
		Config{MTU: 1500},
		Callbacks{OwnerUID: func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 }},
		logger)
	if err != nil {
		t.Fatal(err)
	}
	defer eng.Stop()

	// Idle gap: both pumps block on an empty socket here.
	time.Sleep(200 * time.Millisecond)

	if eng.Failed() {
		t.Fatal("engine reported failure while idle: a pump died on a would-block read")
	}

	pkt := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), netip.MustParseAddr("1.2.3.4"), 1234, 4321)
	if _, err := tunA.Write(pkt); err != nil {
		t.Fatal(err)
	}
	buf := make([]byte, 2048)
	innerB.SetReadDeadline(time.Now().Add(2 * time.Second))
	if n, err := innerB.Read(buf); err != nil || n != len(pkt) {
		t.Fatalf("upstream forwarding died during the idle gap: n=%d err=%v", n, err)
	}

	if _, err := innerB.Write(pkt); err != nil {
		t.Fatal(err)
	}
	tunA.SetReadDeadline(time.Now().Add(2 * time.Second))
	if n, err := tunA.Read(buf); err != nil || n != len(pkt) {
		t.Fatalf("downstream forwarding died during the idle gap: n=%d err=%v", n, err)
	}
}

// TestEngineKeepsForwardingDuringBypassedUDPFlow is the regression test for
// "the first bypassed UDP flow stalls all device traffic".
//
// gVisor's udp.Forwarder, unlike tcp.Forwarder, calls its handler
// synchronously, and the handler ends in a relay pump that only returns on
// error or after a 60s idle timeout. Since the engine injects into the
// netstack from its single TUN-reading goroutine, a synchronous handler pins
// that goroutine for the whole life of the flow: no further packet, tunneled
// or not, is ever read from the TUN.
//
// So: send one packet belonging to an excluded UID towards a real local UDP
// listener (standing in for "the internet"), which is bypassed and leaves an
// idle relay behind, then send a normal tunneled packet and require it to
// reach the inner fd promptly. Pre-fix it only arrives ~60s later, if at all.
func TestEngineKeepsForwardingDuringBypassedUDPFlow(t *testing.T) {
	const excludedUID = 10123

	// The "internet" end of the bypassed flow. It never answers, so the relay
	// stays alive and idle, which is exactly the stalling case.
	//
	// It cannot live on 127.0.0.1: gVisor drops packets with a loopback
	// destination as martians on a non-loopback NIC, so the injected packet
	// would never reach the forwarder at all. Bind a real, non-loopback local
	// address instead.
	host := nonLoopbackIPv4(t)
	listener, err := net.ListenPacket("udp4", net.JoinHostPort(host.String(), "0"))
	if err != nil {
		t.Fatal(err)
	}
	defer listener.Close()
	listenAddr := netip.MustParseAddrPort(listener.LocalAddr().String())

	tunFds, err := rawSocketPair(t)
	if err != nil {
		t.Fatal(err)
	}
	innerFds, err := rawSocketPair(t)
	if err != nil {
		t.Fatal(err)
	}
	tunA := pollableFile(t, tunFds[0], "tun-peer")
	innerB := pollableFile(t, innerFds[1], "inner-peer")
	defer tunA.Close()
	defer innerB.Close()

	logger := device.NewLogger(device.LogLevelError, "test")
	eng, err := Start(tunFds[1], innerFds[0],
		Config{MTU: 1500, ExcludedUIDs: []uint32{excludedUID}},
		Callbacks{
			// A real (no-op) Protect: the dialer's Control hook runs for every
			// bypassed socket, so it must be exercised, not stubbed out.
			Protect: func(int32) {},
			// Only the flow aimed at the listener belongs to the excluded app;
			// everything else is unattributable and must take the tunnel.
			OwnerUID: func(_ Proto, _ netip.AddrPort, dst netip.AddrPort) int32 {
				if dst == listenAddr {
					return excludedUID
				}
				return -1
			},
		},
		logger)
	if err != nil {
		t.Fatal(err)
	}
	defer eng.Stop()

	bypassed := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), listenAddr.Addr(), 1234, listenAddr.Port())
	if _, err := tunA.Write(bypassed); err != nil {
		t.Fatal(err)
	}

	// The bypassed datagram must actually reach the outside world, i.e. the
	// bypass path really ran (and left its relay behind) before we check that
	// the engine is still alive.
	buf := make([]byte, 2048)
	listener.SetReadDeadline(time.Now().Add(5 * time.Second))
	if _, _, err := listener.ReadFrom(buf); err != nil {
		t.Fatalf("bypassed udp datagram never reached the listener: %v", err)
	}

	// The engine must still be pumping the TUN.
	tunneled := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), netip.MustParseAddr("1.2.3.4"), 1234, 4321)
	if _, err := tunA.Write(tunneled); err != nil {
		t.Fatal(err)
	}
	innerB.SetReadDeadline(time.Now().Add(3 * time.Second))
	n, err := innerB.Read(buf)
	if err != nil || n != len(tunneled) {
		t.Fatalf("engine stalled behind the bypassed udp flow: n=%d err=%v", n, err)
	}
}

// TestEngineTCPSynForcesReclassification pins the 5-tuple-reuse leak: once an
// excluded app's connection is gone, a different, NON-excluded app can draw
// the same local port towards the same destination. A SYN means a brand new
// connection, so the cached decision for that 5-tuple is stale and must not be
// reused -- otherwise the new app inherits BYPASS and leaks around the tunnel.
func TestEngineTCPSynForcesReclassification(t *testing.T) {
	var owner int32 = 10123
	e := &Engine{
		flows:     NewFlowTable(flowTableSize, flowTTL, time.Now),
		classify:  NewClassifier([]uint32{10123}, func(Proto, netip.AddrPort, netip.AddrPort) int32 { return owner }, false),
		hasBypass: true,
		logger:    device.NewLogger(device.LogLevelError, "test"),
	}

	src := netip.MustParseAddr("10.0.0.2")
	dst := netip.MustParseAddr("1.2.3.4")
	syn := buildIPv4TCP(src, dst, 40000, 443, header.TCPFlagSyn)
	data := buildIPv4TCP(src, dst, 40000, 443, header.TCPFlagAck)

	if d := e.decide(syn); d != DecisionBypass {
		t.Fatalf("excluded app's SYN: got %v want bypass", d)
	}
	// Mid-connection packets keep using the cached decision (one classify call
	// per connection is the whole point of the flow table).
	owner = -1
	if d := e.decide(data); d != DecisionBypass {
		t.Fatalf("same connection after the SYN: got %v want bypass (cached)", d)
	}
	// New connection, same 5-tuple, different (non-excluded) app.
	if d := e.decide(syn); d != DecisionTunnel {
		t.Fatalf("reused 5-tuple SYN from a non-excluded app: got %v want tunnel", d)
	}
}

// TestEngineStartClosesFdsOnError asserts Start's fd-ownership contract: it
// takes ownership of both fds on success AND failure. If a fallible step
// after the fds are wrapped (here, newBypassStack rejecting a nil Protect)
// causes Start to return an error, both original fd numbers must already be
// closed — otherwise they leak into os.File finalizers, which run at an
// arbitrary later time and can close an unrelated fd the caller/OS has since
// reused for something else.
func TestEngineStartClosesFdsOnError(t *testing.T) {
	tunA, tunB := socketPair(t)
	innerA, innerB := socketPair(t)
	defer tunA.Close()
	defer innerB.Close()

	tunFd := int(tunB.Fd())
	innerFd := int(innerA.Fd())

	logger := device.NewLogger(device.LogLevelError, "test")
	eng, err := Start(tunFd, innerFd,
		Config{MTU: 1500, ExcludedUIDs: []uint32{10123}},
		Callbacks{
			Protect:  nil, // forces newBypassStack to fail
			OwnerUID: func(Proto, netip.AddrPort, netip.AddrPort) int32 { return -1 },
		},
		logger)
	if err == nil {
		eng.Stop()
		t.Fatal("expected Start to fail when bypass stack construction fails (nil Protect)")
	}

	for _, fd := range []int{tunFd, innerFd} {
		if _, fcntlErr := unix.FcntlInt(uintptr(fd), unix.F_GETFD, 0); fcntlErr != unix.EBADF {
			t.Fatalf("fd %d not closed on Start error: fcntl err=%v", fd, fcntlErr)
		}
	}
}
