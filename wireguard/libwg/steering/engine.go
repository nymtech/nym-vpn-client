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

// Start takes ownership of both tunFd and innerFd on success AND on failure:
// on any error return, both fds have already been closed (either directly,
// or via the os.File wrapping them), so the caller must never close them
// itself in either case. This avoids leaving the fds to an os.File
// finalizer, which would run at an arbitrary later time and could end up
// closing an unrelated fd the caller has since reused for something else.
func Start(tunFd int, innerFd int, cfg Config, cb Callbacks, logger *device.Logger) (*Engine, error) {
	tunFile := os.NewFile(uintptr(tunFd), "steering-tun")
	innerFile := os.NewFile(uintptr(innerFd), "steering-inner")

	// Non-blocking so os.File uses the runtime poller and Close() unblocks
	// pending reads (same as newSocketTunFromFD in libwg_android.go).
	for _, fd := range []int{tunFd, innerFd} {
		if err := unix.SetNonblock(fd, true); err != nil {
			tunFile.Close()
			innerFile.Close()
			return nil, err
		}
	}
	e := &Engine{
		tunFile:   tunFile,
		innerFile: innerFile,
		flows:     NewFlowTable(flowTableSize, flowTTL, time.Now),
		classify:  NewClassifier(cfg.ExcludedUIDs, cb.OwnerUID),
		hasBypass: len(cfg.ExcludedUIDs) > 0,
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
