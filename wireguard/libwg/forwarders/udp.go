/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
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

const UDP_WRITE_TIMEOUT = time.Duration(5) * time.Second
const MAX_UDP_DATAGRAM_LEN = 65535

type UDPForwarderConfig struct {
	// Listen port for incoming UDP traffic.
	// For IPv4 endpoint, the listening port is bound to 127.0.0.1, for IPv6 it's ::1.
	ListenPort uint16

	// Client port on loopback from which the incoming connection will be received.
	// Only packets from this port will be passed through to the endpoint.
	ClientPort uint16

	// Endpoint to connect to over netstack
	Endpoint netip.AddrPort
}

// UDP forwarder that creates a bidirectional in-tunnel connection between a local and remote UDP endpoints
type UDPForwarder struct {
	logger *device.Logger

	// Netstack tunnel
	tnet *netstack.Net

	// UDP listener that receives inbound traffic piped to the remote endpoint
	listener *net.UDPConn

	// Outbound connection to the remote endpoint over the entry tunnel
	outbound *gonet.UDPConn

	// Wait group used to signal when all goroutines have finished execution
	waitGroup *sync.WaitGroup
}

func NewUDPForwarder(config UDPForwarderConfig, tnet *netstack.Net, logger *device.Logger) (*UDPForwarder, error) {
	var listenAddr *net.UDPAddr
	var clientAddr *net.UDPAddr

	// Use the same ip protocol family as endpoint
	if config.Endpoint.Addr().Is4() {
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

	outbound, err := tnet.DialUDPAddrPort(netip.AddrPort{}, config.Endpoint)
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
	go wrapper.routineHandleInbound(listener, outbound, clientAddr)
	go wrapper.routineHandleOutbound(listener, outbound, clientAddr)

	return wrapper, nil
}

func (w *UDPForwarder) GetListenAddr() net.Addr {
	return w.listener.LocalAddr()
}

func (w *UDPForwarder) Close() {
	// Close all connections. This should release any blocking ReadFromUDP() calls
	w.listener.Close()
	w.outbound.Close()

	// Wait for all routines to complete
	w.waitGroup.Wait()
}

func (w *UDPForwarder) Wait() {
	w.waitGroup.Wait()
}

func (w *UDPForwarder) routineHandleInbound(inbound *net.UDPConn, outbound *gonet.UDPConn, clientAddr *net.UDPAddr) {
	defer w.waitGroup.Done()
	defer outbound.Close()

	inboundBuffer := make([]byte, MAX_UDP_DATAGRAM_LEN)

	w.logger.Verbosef("udpforwarder(inbound): listening on %s (proxy to %s)", inbound.LocalAddr().String(), outbound.RemoteAddr().String())
	defer w.logger.Verbosef("udpforwarder(inbound): closed")

	for {
		// Receive the WireGuard packet from local port
		bytesRead, senderAddr, err := inbound.ReadFromUDP(inboundBuffer)
		if err != nil {
			w.logger.Errorf("udpforwarder(inbound): %s", err.Error())
			return
		}

		// Drop packet from unknown sender
		if !senderAddr.IP.IsLoopback() || senderAddr.Port != clientAddr.Port {
			w.logger.Verbosef("udpforwarder(inbound): drop packet from unknown sender: %s, expected: %s.", senderAddr.String(), clientAddr.String())
			continue
		}

		// Set write timeout for outbound
		deadline := time.Now().Add(UDP_WRITE_TIMEOUT)
		err = outbound.SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("udpforwarder(inbound): %s", err.Error())
			return
		}

		// Forward the packet over the outbound connection via another WireGuard tunnel
		bytesWritten, err := outbound.Write(inboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("udpforwarder(inbound): %s", err.Error())
			return
		}

		// todo: is it possible?
		if bytesWritten != bytesRead {
			w.logger.Errorf("udpforwarder(inbound): wrote %d bytes, expected %d", bytesWritten, bytesRead)
		}
	}
}

func (w *UDPForwarder) routineHandleOutbound(inbound *net.UDPConn, outbound *gonet.UDPConn, clientAddr *net.UDPAddr) {
	defer w.waitGroup.Done()
	defer inbound.Close()

	remoteAddr := outbound.RemoteAddr().(*net.UDPAddr)
	w.logger.Verbosef("udpforwarder(outbound): dial %s", remoteAddr.String())
	defer w.logger.Verbosef("udpforwarder(outbound): closed")

	outboundBuffer := make([]byte, MAX_UDP_DATAGRAM_LEN)

	for {
		// Receive WireGuard packet from remote server
		bytesRead, senderAddr, err := outbound.ReadFrom(outboundBuffer)
		if err != nil {
			w.logger.Errorf("udpforwarder(outbound): %s", err.Error())
			return
		}
		// Cast net.Addr to net.UDPAddr
		senderUDPAddr := senderAddr.(*net.UDPAddr)

		// Drop packet from unknown sender.
		if !senderUDPAddr.IP.Equal(remoteAddr.IP) || senderUDPAddr.Port != remoteAddr.Port {
			w.logger.Verbosef("udpforwarder(outbound): drop packet from unknown sender: %s, expected: %s", senderUDPAddr.String(), remoteAddr.String())
			continue
		}

		// Set write timeout for inbound
		deadline := time.Now().Add(UDP_WRITE_TIMEOUT)
		err = inbound.SetWriteDeadline(deadline)
		if err != nil {
			w.logger.Errorf("udpforwarder(outbound): %s", err.Error())
			return
		}

		// Forward packet from remote to local client
		bytesWritten, err := inbound.WriteToUDP(outboundBuffer[:bytesRead], clientAddr)
		if err != nil {
			w.logger.Errorf("udpforwarder(outbound): %s", err.Error())
			return
		}

		// todo: is it possible?
		if bytesWritten != bytesRead {
			w.logger.Errorf("udpforwarder(outbound): wrote %d bytes, expected %d", bytesWritten, bytesRead)
		}
	}
}
