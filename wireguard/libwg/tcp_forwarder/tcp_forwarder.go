/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
 */

package tcp_forwarder

import (
	"net"
	"net/netip"
	"sync"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
	"gvisor.dev/gvisor/pkg/tcpip/adapters/gonet"
)

type TCPForwarder struct {
	// Logger.
	logger *device.Logger

	// UDP listener that receives inbound traffic destined to endpoint.
	listener *net.TCPListener

	// Outbound connection to the endpoint over the tunnel.
	outbound *gonet.TCPConn

	// Wait group used to signal when all goroutines have finished execution.
	waitGroup *sync.WaitGroup
}

const TCP_BUFFER_LEN = 65535
const TCP_WRITE_TIMEOUT = time.Duration(5) * time.Second

func New(endpoint netip.AddrPort, tnet *netstack.Net, logger *device.Logger) (*TCPForwarder, error) {
	var listenAddr *net.TCPAddr

	// Use the same ip protocol family as exit endpoint.
	if endpoint.Addr().Is4() {
		loopback := netip.AddrFrom4([4]byte{127, 0, 0, 1})
		listenAddr = net.TCPAddrFromAddrPort(netip.AddrPortFrom(loopback, 0))
	} else {
		listenAddr = net.TCPAddrFromAddrPort(netip.AddrPortFrom(netip.IPv6Loopback(), 0))
	}

	listener, err := net.ListenTCP("tcp", listenAddr)
	if err != nil {
		return nil, err
	}

	outbound, err := tnet.DialTCPAddrPort(endpoint)
	if err != nil {
		return nil, err
	}

	waitGroup := &sync.WaitGroup{}
	forwarder := &TCPForwarder{
		logger:    logger,
		listener:  listener,
		outbound:  outbound,
		waitGroup: waitGroup,
	}
	waitGroup.Add(1)
	go forwarder.RoutineListenTCP(listener)

	return forwarder, nil
}

// Get listener address that should be used to connect to the forwarder.
func (w *TCPForwarder) GetListenAddr() net.Addr {
	return w.listener.Addr()
}

func (w *TCPForwarder) Close() {
	// Close all connections. This should release any blocking ReadFromUDP() calls.
	w.listener.Close()
	w.outbound.Close()

	// Wait for all routines to complete.
	w.waitGroup.Wait()
}

func (w *TCPForwarder) Wait() {
	w.waitGroup.Wait()
}

func (w *TCPForwarder) RoutineListenTCP(listener *net.TCPListener) {
	defer w.waitGroup.Done()

	w.logger.Verbosef("tcpforwarder(listen): listening on %s", listener.Addr().String())
	defer w.logger.Verbosef("tcpforwarder(listen): closed")

	inbound, err := listener.AcceptTCP()
	if err != nil {
		w.logger.Errorf("tcpforwarder(listen): %s", err.Error())
		return
	}

	w.waitGroup.Add(2)
	go w.RoutineHandleInbound(inbound, w.outbound)
	go w.RoutineHandleOutbound(inbound, w.outbound)
}

func (w *TCPForwarder) RoutineHandleInbound(inbound *net.TCPConn, outbound *gonet.TCPConn) {
	defer w.waitGroup.Done()

	inboundBuffer := make([]byte, TCP_BUFFER_LEN)

	w.logger.Verbosef("tcpforwarder(inbound): accepted from %s", (*inbound).LocalAddr().String())
	defer w.logger.Verbosef("tcpforwarder(inbound): closed")

	for {
		// Receive bytes from local socket
		bytesRead, err := (*inbound).Read(inboundBuffer)
		if err != nil {
			w.logger.Errorf("tcpforwarder(inbound): %s", err.Error())
			return
		}

		// Set write timeout for outbound.
		deadline := time.Now().Add(TCP_WRITE_TIMEOUT)
		err = outbound.SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("tcpforwarder(inbound): %s", err.Error())
			// todo: handle error
			return
		}

		// Forward the packet over the outbound connection via another WireGuard tunnel.
		_, err = outbound.Write(inboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("tcpforwarder(inbound): %s", err.Error())
			// todo: handle error
			return
		}
	}
}

func (w *TCPForwarder) RoutineHandleOutbound(inbound *net.TCPConn, outbound *gonet.TCPConn) {
	defer w.waitGroup.Done()

	remoteAddr := outbound.RemoteAddr().(*net.TCPAddr)
	w.logger.Verbosef("tcpforwarder(outbound): dial %s", remoteAddr.String())
	defer w.logger.Verbosef("tcpforwarder(outbound): closed")

	outboundBuffer := make([]byte, TCP_BUFFER_LEN)

	for {
		// Receive packets from remote server.
		bytesRead, err := outbound.Read(outboundBuffer)
		if err != nil {
			w.logger.Errorf("tcpforwarder(outbound): %s", err.Error())
			// todo: handle error
			return
		}

		// Set write timeout for inbound.
		deadline := time.Now().Add(TCP_WRITE_TIMEOUT)
		err = (*inbound).SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("tcpforwarder(outbound): %s", err.Error())
			return
		}

		// Forward packet from remote to local client.
		_, err = (*inbound).Write(outboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("tcpforwarder(outbound): %s", err.Error())
			return
		}
	}
}
