/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
 */

package udp_forwarder

import (
	"net"
	"net/netip"
	"sync"
	"time"

	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun/netstack"
	"gvisor.dev/gvisor/pkg/tcpip/adapters/gonet"
)

const UDP_WRITE_TIMEOUT = time.Duration(5) * time.Second
const MAX_UDP_DATAGRAM_LEN = 65535

type UDPForwarderConfig struct {
	// Listen port for incoming WireGuard traffic.
	// For IPv4 exit endpoint, the listening port is bound to 127.0.0.1, for IPv6 it's ::1.
	ListenPort uint16

	// Client port on loopback from which the incoming WireGuard connection will be received.
	// Only packets from this port will be passed through to the exit endpoint.
	ClientPort uint16

	// Exit endpoint which will receive the raw WireGuard packets received on the listen port.
	// The connection to exit endpoint is established over the entry tunnel, thus it creates
	// a tunnel inside of tunnel.
	ExitEndpoint netip.AddrPort
}

// UDP forwarder that creates a bidirectional connection between a local and exit UDP endpoints
// over the netstack-based WireGuard tunnel.
type UDPForwarder struct {
	// Logger.
	logger *device.Logger

	// Netstack tunnel wrapping the inbound WireGuard traffic.
	tnet *netstack.Net

	// UDP listener that receives inbound WireGuard traffic destined to exit endpoint.
	listener *net.UDPConn

	// Outbound connection to the exit endpoint over the entry tunnel.
	outbound *gonet.UDPConn

	// Wait group used to signal when all goroutines have finished execution.
	waitGroup *sync.WaitGroup
}

func New(config UDPForwarderConfig, tnet *netstack.Net, logger *device.Logger) (*UDPForwarder, error) {
	var listenAddr *net.UDPAddr
	var clientAddr *net.UDPAddr

	// Use the same ip protocol family as exit endpoint.
	if config.ExitEndpoint.Addr().Is4() {
		loopback := netip.AddrFrom4([4]byte{127, 0, 0, 1})
		listenAddr = net.UDPAddrFromAddrPort(netip.AddrPortFrom(loopback, config.ListenPort))
		clientAddr = net.UDPAddrFromAddrPort(netip.AddrPortFrom(loopback, config.ClientPort))
	} else {
		listenAddr = net.UDPAddrFromAddrPort(netip.AddrPortFrom(netip.IPv6Loopback(), config.ListenPort))
		clientAddr = net.UDPAddrFromAddrPort(netip.AddrPortFrom(netip.IPv6Loopback(), config.ClientPort))
	}

	listener, err := net.ListenUDP("udp", listenAddr)
	if err != nil {
		return nil, err
	}

	outbound, err := tnet.DialUDPAddrPort(netip.AddrPort{}, config.ExitEndpoint)
	if err != nil {
		return nil, err
	}

	waitGroup := &sync.WaitGroup{}
	wrapper := &UDPForwarder{
		logger,
		tnet,
		listener,
		outbound,
		waitGroup,
	}

	waitGroup.Add(2)
	go wrapper.RoutineHandleInbound(listener, outbound, clientAddr)
	go wrapper.RoutineHandleOutbound(listener, outbound, clientAddr)

	return wrapper, nil
}

func (w *UDPForwarder) Close() {
	// Close all connections. This should release any blocking ReadFromUDP() calls.
	w.listener.Close()
	w.outbound.Close()

	// Wait for all routines to complete.
	w.waitGroup.Wait()
}

func (w *UDPForwarder) Wait() {
	w.waitGroup.Wait()
}

func (w *UDPForwarder) RoutineHandleInbound(inbound *net.UDPConn, outbound *gonet.UDPConn, clientAddr *net.UDPAddr) {
	defer w.waitGroup.Done()

	inboundBuffer := make([]byte, MAX_UDP_DATAGRAM_LEN)

	w.logger.Verbosef("udpforwarder(inbound): listening on %s", inbound.LocalAddr().String())
	defer w.logger.Verbosef("udpforwarder(inbound): closed")

	for {
		// Receive the WireGuard packet from local port
		bytesRead, senderAddr, err := inbound.ReadFromUDP(inboundBuffer)
		if err != nil {
			w.logger.Errorf("udpforwarder(inbound): %s", err.Error())
			// todo: handle error
			return
		}

		w.logger.Verbosef("udpforwarder(inbound): received %d bytes <- %s", bytesRead, senderAddr.String())

		// Drop packet from unknown sender.
		if !senderAddr.IP.IsLoopback() || senderAddr.Port != clientAddr.Port {
			w.logger.Verbosef("udpforwarder(inbound): drop packet from unknown sender: %s, expected: %s.", senderAddr.String(), clientAddr.String())
			continue
		}

		// Set write timeout for outbound.
		deadline := time.Now().Add(UDP_WRITE_TIMEOUT)
		err = outbound.SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("udpforwarder(inbound): %s", err.Error())
			// todo: handle error
			continue
		}

		// Forward the packet over the outbound connection via another WireGuard tunnel.
		bytesWritten, err := outbound.Write(inboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("udpforwarder(inbound): %s", err.Error())
			// todo: handle error
			continue
		}
		w.logger.Verbosef("udpforwarder(inbound): sent %d bytes -> %s", bytesWritten, outbound.RemoteAddr().String())
	}
}

func (w *UDPForwarder) RoutineHandleOutbound(inbound *net.UDPConn, outbound *gonet.UDPConn, clientAddr *net.UDPAddr) {
	defer w.waitGroup.Done()

	remoteAddr := outbound.RemoteAddr().(*net.UDPAddr)
	w.logger.Verbosef("udpforwarder(outbound): dial %s", remoteAddr.String())
	defer w.logger.Verbosef("udpforwarder(outbound): closed")

	outboundBuffer := make([]byte, MAX_UDP_DATAGRAM_LEN)

	for {
		// Receive WireGuard packet from remote server.
		bytesRead, senderAddr, err := outbound.ReadFrom(outboundBuffer)
		if err != nil {
			w.logger.Errorf("udpforwarder(outbound): %s", err.Error())
			// todo: handle error
			return
		}
		// Cast net.Addr to net.UDPAddr
		senderUDPAddr := senderAddr.(*net.UDPAddr)
		w.logger.Verbosef("udpforwarder(outbound): received %d bytes <- %s", bytesRead, senderUDPAddr.String())

		// Drop packet from unknown sender.
		if !senderUDPAddr.IP.Equal(remoteAddr.IP) || senderUDPAddr.Port != remoteAddr.Port {
			w.logger.Verbosef("udpforwarder(outbound): drop packet from unknown sender: %s, expected: %s", senderUDPAddr.String(), remoteAddr.String())
			continue
		}

		// Set write timeout for inbound.
		deadline := time.Now().Add(UDP_WRITE_TIMEOUT)
		err = inbound.SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("udpforwarder(outbound): %s", err.Error())
			// todo: handle error
			continue
		}

		// Forward packet from remote to local client.
		bytesWritten, err := inbound.WriteToUDP(outboundBuffer[:bytesRead], clientAddr)
		if err != nil {
			w.logger.Errorf("udpforwarder(outbound): %s", err.Error())
			// todo: handle error
			continue
		}
		w.logger.Verbosef("udpforwarder(outbound): sent %d bytes -> %s", bytesWritten, clientAddr.String())
	}
}
