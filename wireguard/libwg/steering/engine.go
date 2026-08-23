/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"errors"
	"os"
	"sync"
	"sync/atomic"
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

	// stopping is set by Stop before the fds are closed, so the pumps can
	// tell an orderly shutdown apart from a genuine I/O failure.
	stopping atomic.Bool
	// failed is set when a pump dies for a reason that is NOT an orderly
	// shutdown. Once set, packets are no longer moving in at least one
	// direction and the tunnel must be torn down: see Failed.
	failed atomic.Bool
}

// Start takes ownership of both tunFd and innerFd on success AND on failure:
// on any error return, both fds have already been closed (either directly,
// or via the os.File wrapping them), so the caller must never close them
// itself in either case. This avoids leaving the fds to an os.File
// finalizer, which would run at an arbitrary later time and could end up
// closing an unrelated fd the caller has since reused for something else.
func Start(tunFd int, innerFd int, cfg Config, cb Callbacks, logger *device.Logger) (*Engine, error) {
	// Set O_NONBLOCK on the RAW fds BEFORE wrapping them, not after: os.NewFile
	// only registers an fd with the runtime network poller if the fd ALREADY
	// carries O_NONBLOCK at wrap time. Wrapping a blocking fd and then flipping
	// it non-blocking behind the poller's back yields a non-pollable os.File on
	// a non-blocking fd, so every would-be-blocking read returns EAGAIN as a
	// hard error instead of parking the goroutine -- both pumps then exit within
	// milliseconds of Start and all traffic is blackholed while the tunnel still
	// reports Connected. Mirrors newSocketTunFromFD in libwg_android.go.
	for _, fd := range []int{tunFd, innerFd} {
		if err := unix.SetNonblock(fd, true); err != nil {
			// No os.File owns these yet, so close the raw fds directly to keep
			// Start's "owns both fds on every return path" contract.
			unix.Close(tunFd)
			unix.Close(innerFd)
			return nil, err
		}
	}
	tunFile := os.NewFile(uintptr(tunFd), "steering-tun")
	innerFile := os.NewFile(uintptr(innerFd), "steering-inner")

	e := &Engine{
		tunFile:   tunFile,
		innerFile: innerFile,
		flows:     NewFlowTable(flowTableSize, flowTTL, time.Now),
		classify:  NewClassifier(cfg.ExcludedUIDs, cb.OwnerUID, LanBypassPrefixes(cfg.BypassLan, cfg.LanPrefixes)),
		hasBypass: len(cfg.ExcludedUIDs) > 0 || cfg.BypassLan,
		dnsDirect: len(cfg.UnderlyingDNS) > 0,
		logger:    logger,
	}
	if e.hasBypass {
		b, err := newBypassStack(cfg, cb, e.writeToTun, logger)
		if err != nil {
			tunFile.Close()
			innerFile.Close()
			return nil, err
		}
		e.bypass = b
	}
	e.waitGroup.Add(2)
	go e.runUpstream()
	go e.runDownstream()
	logger.Verbosef("steering: engine started (excluded UIDs: %d, LAN bypass: %v, direct DNS: %v)", len(cfg.ExcludedUIDs), cfg.BypassLan, e.dnsDirect)
	return e, nil
}

// Failed reports whether a packet pump has died for a reason other than an
// orderly Stop. When it returns true the engine is permanently degraded (at
// least one direction no longer moves packets, and excluded apps may have lost
// connectivity entirely), so the platform layer must tear the tunnel down
// rather than leave a silent blackhole behind a "Connected" UI.
func (e *Engine) Failed() bool {
	return e.failed.Load()
}

// pumpDied records the death of a packet pump. Shutdown-induced deaths are
// expected and only traced; anything else flips the failed flag (once) and is
// logged at error level so it is visible in logcat.
func (e *Engine) pumpDied(what string, err error) {
	if e.stopping.Load() {
		e.logger.Verbosef("steering: %s stopped during shutdown: %s", what, err)
		return
	}
	if e.failed.CompareAndSwap(false, true) {
		e.logger.Errorf("steering: %s died unrecoverably (%s); traffic is no longer being forwarded, the tunnel must be torn down", what, err)
	}
}

// isFatalIOError distinguishes "this fd is gone" from a transient, per-packet
// failure. Transient errors (ENOBUFS, EMSGSIZE, EINTR, ...) mean one packet was
// lost, which is normal at the IP layer; tearing the pump down for one of those
// would turn a dropped packet into a permanent blackhole.
func isFatalIOError(err error) bool {
	if errors.Is(err, os.ErrClosed) {
		return true
	}
	for _, fatal := range []unix.Errno{unix.EBADF, unix.EPIPE, unix.ECONNRESET, unix.ENOTCONN, unix.ESHUTDOWN, unix.ENODEV, unix.ENXIO} {
		if errors.Is(err, fatal) {
			return true
		}
	}
	return false
}

func (e *Engine) Stop() {
	e.closeOnce.Do(func() {
		e.stopping.Store(true)
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
		// Log and continue: a transient write failure only costs one packet.
		if isFatalIOError(err) {
			e.pumpDied("tun write", err)
			return
		}
		e.logger.Verbosef("steering: write to tun failed (packet dropped): %s", err)
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
			e.pumpDied("tun read", err)
			return
		}
		pkt := buf[:n]
		if e.decide(pkt) == DecisionBypass {
			info, _ := ParsePacket(pkt)
			e.bypass.InjectInbound(pkt, info.IsIPv4)
		} else {
			if _, err := e.innerFile.Write(pkt); err != nil {
				// Only a dead fd stops the pump; anything else (e.g. a
				// transient ENOBUFS) drops this packet and carries on.
				if isFatalIOError(err) {
					e.pumpDied("inner write", err)
					return
				}
				e.logger.Verbosef("steering: inner write failed (packet dropped): %s", err)
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
	// LAN-destination bypass is dest-based and takes precedence over everything
	// below, including the DNS-to-tunnel fallback: a DNS query to a LAN resolver
	// must also go direct. It's a pure function of the destination, so it's not
	// cached in the flow table.
	if e.classify.IsLanDestBypass(info.Key) {
		return DecisionBypass
	}
	// Without underlying resolvers, excluded DNS must use the tunnel resolver.
	if info.Key.Proto == ProtoUDP && info.Key.Dst.Port() == dnsPort && !e.dnsDirect {
		return DecisionTunnel
	}
	// A TCP SYN starts a NEW connection, so any cached decision for this
	// 5-tuple belongs to a previous, already-dead connection. Reusing it would
	// leak: once an excluded app frees its ephemeral port, a non-excluded app
	// that happens to draw the same local port towards the same destination
	// within the entry's lifetime would inherit the cached BYPASS and go out
	// directly. Force re-classification and overwrite the stale entry.
	if !info.IsTCPSyn {
		if d, ok := e.flows.Lookup(info.Key); ok {
			return d
		}
	}
	d := e.classify.Decide(info.Key)
	// UDP has no SYN to hang re-classification off, so bound its entries by the
	// same idle timeout the bypass relay uses: once that relay has given up on
	// a flow, the 5-tuple is free for another app to reuse and the cached
	// decision must no longer apply. (Lookup refreshes `seen`, so this is an
	// idle timeout, not a hard lifetime.)
	ttl := flowTTL
	if info.Key.Proto == ProtoUDP {
		ttl = udpIdleTimeout
	}
	e.flows.InsertWithTTL(info.Key, d, ttl)
	return d
}

// runDownstream pumps tunnel responses back to the TUN.
func (e *Engine) runDownstream() {
	defer e.waitGroup.Done()
	buf := make([]byte, maxPacketSize)
	for {
		n, err := e.innerFile.Read(buf)
		if err != nil {
			e.pumpDied("inner read", err)
			return
		}
		e.writeToTun(buf[:n])
	}
}
