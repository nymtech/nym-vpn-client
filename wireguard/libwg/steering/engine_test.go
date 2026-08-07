/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net/netip"
	"os"
	"syscall"
	"testing"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"golang.org/x/sys/unix"
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
