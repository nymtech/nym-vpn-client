/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
 */

package forwarders

import (
	"context"
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

	// TCP listener accepting connections on local address and establishing a bidirectional connection to the endpoint over netstack tunnel
	listener *net.TCPListener

	// Endpoint to connect to over netstack
	endpoint netip.AddrPort

	// Cancellation context
	ctx context.Context

	// Cancellation func
	cancel context.CancelFunc

	// Wait group used to signal when all goroutines have finished execution.
	waitGroup *sync.WaitGroup
}

const TCP_BUFFER_LEN = 65535
const TCP_WRITE_TIMEOUT = time.Duration(5) * time.Second

func NewTCPForwarder(endpoint netip.AddrPort, tnet *netstack.Net, logger *device.Logger) (*TCPForwarder, error) {
	var listenAddr *net.TCPAddr

	// Use the same ip protocol family as endpoint
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

	ctx, cancel := context.WithCancel(context.Background())

	waitGroup := &sync.WaitGroup{}
	forwarder := &TCPForwarder{
		logger:    logger,
		tnet:      tnet,
		listener:  listener,
		endpoint:  endpoint,
		ctx:       ctx,
		cancel:    cancel,
		waitGroup: waitGroup,
	}
	waitGroup.Add(1)
	go forwarder.routineListenTCP()

	return forwarder, nil
}

func (w *TCPForwarder) GetListenAddr() net.Addr {
	return w.listener.Addr()
}

func (w *TCPForwarder) Close() {
	// Close TCP listener connection
	w.listener.Close()

	// Cancel all active connections
	w.cancel()

	// Wait for all routines to complete
	w.waitGroup.Wait()
}

func (w *TCPForwarder) Wait() {
	w.waitGroup.Wait()
}

func (w *TCPForwarder) routineListenTCP() {
	defer w.waitGroup.Done()

	w.logger.Verbosef("tcpforwarder(listen): listening on %s (proxy to %s)", w.listener.Addr().String(), w.endpoint.String())
	defer w.logger.Verbosef("tcpforwarder(listen): closed")

	// Cancel pending connections when TCP listener is closed
	defer w.cancel()

	newConns := make(chan *net.TCPConn)

	go func() {
		for {
			inbound, err := w.listener.AcceptTCP()
			if err != nil {
				w.logger.Errorf("tcpforwarder(listen): failed to accept connection: %s", err.Error())
				w.cancel()
				return
			}

			newConns <- inbound
		}
	}()

	for {
		select {
		case inbound := <-newConns:
			w.waitGroup.Add(1)
			go w.routineHandleNewConnection(inbound)
		case <-w.ctx.Done():
			return
		}
	}
}

func (w *TCPForwarder) routineHandleNewConnection(inbound *net.TCPConn) {
	defer w.waitGroup.Done()

	w.logger.Verbosef("tcpforwarder(listen): accepted from %s", (*inbound).RemoteAddr().String())

	ctx, cancel := context.WithCancel(w.ctx)
	defer cancel()

	go func() {
		<-ctx.Done()
		inbound.Close()
	}()

	w.logger.Verbosef("tcpforwarder(listen): dial %s", w.endpoint.String())
	outbound, err := w.tnet.DialContextTCPAddrPort(ctx, w.endpoint)
	if err != nil {
		w.logger.Errorf("tcpforwarder(listen): failed to connect to %s: %s", w.endpoint.String(), err.Error())
		return
	}

	go func() {
		<-ctx.Done()
		outbound.Close()
	}()

	waitGroup := &sync.WaitGroup{}
	waitGroup.Add(2)

	go w.routineHandleInbound(inbound, outbound, waitGroup)
	go w.routineHandleOutbound(inbound, outbound, waitGroup)

	waitGroup.Wait()
	w.logger.Verbosef("tcpforwarder(listen): connection closed")
}

func (w *TCPForwarder) routineHandleInbound(inbound *net.TCPConn, outbound *gonet.TCPConn, waitGroup *sync.WaitGroup) {
	defer waitGroup.Done()
	defer w.logger.Verbosef("tcpforwarder(inbound): closed")
	defer outbound.Close()

	inboundBuffer := make([]byte, TCP_BUFFER_LEN)

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

func (w *TCPForwarder) routineHandleOutbound(inbound *net.TCPConn, outbound *gonet.TCPConn, waitGroup *sync.WaitGroup) {
	defer waitGroup.Done()
	defer w.logger.Verbosef("tcpforwarder(outbound): closed")
	defer inbound.Close()

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
