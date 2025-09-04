/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
 */

package forwarders

import (
	"net"
	"net/netip"
	"sync"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
	"gvisor.dev/gvisor/pkg/tcpip/adapters/gonet"
)

// TCP forwarder that creates a bidirectional in-tunnel connection between a local and remote TCP endpoints
type TCPForwarder struct {
	logger *device.Logger

	// Netstack tunnel
	tnet *netstack.Net

	// Endpoint to connect to over netstack
	endpoint netip.AddrPort

	// TCP listener accepting connections on local address and establishing a bidirectional connection to the endpoint over netstack tunnel
	listener *net.TCPListener

	// Wait group used to signal when all goroutines have finished execution.
	waitGroup *sync.WaitGroup
}

const TCP_BUFFER_LEN = 65535
const TCP_WRITE_TIMEOUT = time.Duration(5) * time.Second

func NewTCPForwarder(endpoint netip.AddrPort, tnet *netstack.Net, logger *device.Logger) (*TCPForwarder, error) {
	var listenAddr *net.TCPAddr

	// Use the same ip protocol family as exit endpoint
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

	waitGroup := &sync.WaitGroup{}
	forwarder := &TCPForwarder{
		logger:    logger,
		listener:  listener,
		endpoint:  endpoint,
		waitGroup: waitGroup,
	}
	waitGroup.Add(1)
	go forwarder.routineListenTCP(listener)

	return forwarder, nil
}

// Get listener address that should be used to connect to the forwarder.
func (w *TCPForwarder) GetListenAddr() net.Addr {
	return w.listener.Addr()
}

func (w *TCPForwarder) Close() {
	// Close TCP listener connection
	// Active connections will be closed shortly after
	w.listener.Close()

	// Wait for all routines to complete
	w.waitGroup.Wait()
}

func (w *TCPForwarder) Wait() {
	w.waitGroup.Wait()
}

func (w *TCPForwarder) routineListenTCP(listener *net.TCPListener) {
	defer w.waitGroup.Done()

	w.logger.Verbosef("tcpforwarder(listen): listening on %s", listener.Addr().String())
	defer w.logger.Verbosef("tcpforwarder(listen): closed")

	outbounds := []*gonet.TCPConn{}
	inbounds := []*net.TCPConn{}
	defer func() {
		w.logger.Verbosef("tcpforwarder(listen): closing connections")
		for _, outbound := range outbounds {
			outbound.Close()
		}
		for _, inbound := range inbounds {
			inbound.Close()
		}
	}()

	for {
		inbound, err := listener.AcceptTCP()
		if err != nil {
			w.logger.Errorf("tcpforwarder(listen): failed to accept connection: %s", err.Error())
			return
		}

		outbound, err := w.tnet.DialTCPAddrPort(w.endpoint)
		if err != nil {
			w.logger.Errorf("tcpforwarder(listen): failed to connect to %s: %s", w.endpoint.String(), err.Error())
			inbound.Close()
			continue
		}

		inbounds = append(inbounds, inbound)
		outbounds = append(outbounds, outbound)

		w.waitGroup.Add(2)
		go w.routineHandleInbound(inbound, outbound)
		go w.routineHandleOutbound(inbound, outbound)
	}
}

func (w *TCPForwarder) routineHandleInbound(inbound *net.TCPConn, outbound *gonet.TCPConn) {
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

		// Set write timeout for outbound
		deadline := time.Now().Add(TCP_WRITE_TIMEOUT)
		err = outbound.SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("tcpforwarder(inbound): %s", err.Error())
			return
		}

		// Forward the packet over the outbound connection via another WireGuard tunnel
		bytesWritten, err := outbound.Write(inboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("tcpforwarder(inbound): %s", err.Error())
			return
		}

		// todo: is it possible?
		if bytesWritten != bytesRead {
			w.logger.Errorf("tcpforwarder(inbound): wrote %d bytes, expected %d", bytesWritten, bytesRead)
		}
	}
}

func (w *TCPForwarder) routineHandleOutbound(inbound *net.TCPConn, outbound *gonet.TCPConn) {
	defer w.waitGroup.Done()

	remoteAddr := outbound.RemoteAddr().(*net.TCPAddr)
	w.logger.Verbosef("tcpforwarder(outbound): dial %s", remoteAddr.String())
	defer w.logger.Verbosef("tcpforwarder(outbound): closed")

	outboundBuffer := make([]byte, TCP_BUFFER_LEN)

	for {
		// Receive packets from remote server
		bytesRead, err := outbound.Read(outboundBuffer)
		if err != nil {
			w.logger.Errorf("tcpforwarder(outbound): %s", err.Error())
			return
		}

		// Set write timeout for inbound
		deadline := time.Now().Add(TCP_WRITE_TIMEOUT)
		err = (*inbound).SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("tcpforwarder(outbound): %s", err.Error())
			return
		}

		// Forward packet from remote to local client
		bytesWritten, err := (*inbound).Write(outboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("tcpforwarder(outbound): %s", err.Error())
			return
		}

		// todo: is it possible?
		if bytesWritten != bytesRead {
			w.logger.Errorf("tcpforwarder(outbound): wrote %d bytes, expected %d", bytesWritten, bytesRead)
		}
	}
}
