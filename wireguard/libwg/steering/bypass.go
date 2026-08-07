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

// Config configures the bypass netstack: which app UIDs get their flows
// dialed directly instead of through the tunnel, which underlying-network
// DNS resolvers to redirect bypassed DNS flows to, and the netstack MTU.
type Config struct {
	ExcludedUIDs  []uint32
	UnderlyingDNS []netip.Addr
	MTU           int
}

// Callbacks lets the bypass stack reach back into the platform layer.
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

// bypassStack is a gVisor netstack that terminates flows injected via
// InjectInbound and bridges them to real sockets dialed on the underlying
// network (protected from the VPN via Callbacks.Protect), pumping any
// netstack-emitted packets (SYN-ACKs, ACKs, DNS responses, ...) back to the
// TUN via writeToTun.
//
// Fail-closed: this stack never originates a route decision. It only ever
// serves flows the engine (Task 4) explicitly injects here; everything else
// stays on the tunnel.
type bypassStack struct {
	stack     *stack.Stack
	endpoint  *channel.Endpoint
	dialer    net.Dialer
	cfg       Config
	logger    *device.Logger
	ctx       context.Context
	cancel    context.CancelFunc
	waitGroup sync.WaitGroup

	mu     sync.Mutex
	closed bool
}

// tryAdd registers a goroutine (handler, pump, or dial) as tracked work
// against b.waitGroup, unless Close has already begun. It must be used
// instead of a bare waitGroup.Add anywhere that can race Close's Wait:
// gVisor's tcp.Forwarder invokes handleTCP/handleUDP on fresh, unmanaged
// goroutines, so an Add there can otherwise land concurrently with a Wait
// that just observed the counter hit zero, which panics ("WaitGroup misuse:
// Add called concurrently with Wait"). Serializing Add against the
// closed flag under mu removes that race, and rejecting new work once
// closed keeps Close's "waits for every goroutine started by this stack"
// contract true even for handlers that haven't reached CreateEndpoint yet.
func (b *bypassStack) tryAdd() bool {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.closed {
		return false
	}
	b.waitGroup.Add(1)
	return true
}

func newBypassStack(cfg Config, cb Callbacks, writeToTun func([]byte), logger *device.Logger) (*bypassStack, error) {
	if cb.Protect == nil {
		return nil, fmt.Errorf("bypass stack requires a non-nil Protect callback: every socket dialed for a bypassed flow must be protected")
	}
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
			// cb.Protect is guaranteed non-nil (checked above), so this hook
			// unconditionally protects every socket this stack dials.
			Control: func(network, address string, c syscall.RawConn) error {
				return c.Control(func(fd uintptr) {
					cb.Protect(int32(fd))
				})
			},
		},
	}

	tcpFwd := tcp.NewForwarder(s, 0, tcpForwarderInFlight, b.handleTCP)
	s.SetTransportProtocolHandler(tcp.ProtocolNumber, tcpFwd.HandlePacket)
	udpFwd := udp.NewForwarder(s, b.handleUDP)
	s.SetTransportProtocolHandler(udp.ProtocolNumber, udpFwd.HandlePacket)

	// Pump packets the netstack emits (responses to the apps) back to the TUN.
	// This Add happens before newBypassStack returns and before Close can
	// possibly be called (the caller doesn't have b yet), so it cannot race
	// Close's Wait; tryAdd is not needed here.
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

// InjectInbound feeds a raw IP packet read from the TUN into the netstack,
// as if it arrived on the wire.
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

// Close tears down the netstack and blocks until the packet pump and every
// flow-bridging goroutine started by this stack have exited. Setting closed
// (under mu, before cancel/endpoint-close/stack-destroy) makes every
// in-flight handleTCP/handleUDP invocation observe the shutdown via tryAdd
// and bail out instead of touching a torn-down stack; the Wait at the end
// then blocks until all goroutines that did win a tryAdd race have
// finished.
func (b *bypassStack) Close() {
	b.mu.Lock()
	b.closed = true
	b.mu.Unlock()

	b.cancel()
	b.endpoint.Close()
	b.stack.Destroy()
	b.waitGroup.Wait()
}

func (b *bypassStack) handleTCP(r *tcp.ForwarderRequest) {
	// gVisor's tcp.Forwarder invokes handleTCP on a fresh goroutine per
	// flow, so this Add must be serialized against Close's closed flag
	// (tryAdd) rather than called bare: otherwise it can race a Wait that
	// just observed the counter reach zero and panic, and an untracked
	// handler could still be between here and CreateEndpoint when Close
	// returns and the stack is destroyed.
	if !b.tryAdd() {
		r.Complete(true) // free the forwarder's in-flight slot; we're shutting down
		return
	}
	defer b.waitGroup.Done()

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

	// Run the pump synchronously so the flow's lifetime stays covered by
	// the Add above for as long as data is flowing, instead of handing off
	// to a second, separately tracked goroutine.
	pump(b.ctx, inbound, outbound)
}

func (b *bypassStack) handleUDP(r *udp.ForwarderRequest) {
	// See handleTCP: gVisor's udp.Forwarder also dispatches on a fresh,
	// unmanaged goroutine, so this must go through tryAdd, not a bare Add.
	if !b.tryAdd() {
		return
	}
	defer b.waitGroup.Done()

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

	// Run synchronously (see handleTCP) so the whole flow's lifetime is
	// covered by the Add above.
	pumpUDP(b.ctx, inbound, outbound)
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
