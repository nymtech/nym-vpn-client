/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
 */

package tcp_forwarder

import (
	"net"
	"sync"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/higebu/netfd"
	"github.com/prep/socketpair"
	"gvisor.dev/gvisor/pkg/tcpip/adapters/gonet"
)

type TCPForwarder struct {
	// Logger.
	logger *device.Logger

	// Consumer socket
	consumerSocket *net.Conn

	// Local socket representing one end of socket pair.
	localSocket *net.Conn

	// Virtual over the tunnel connection
	remoteSocket *gonet.TCPConn

	// Wait group used to signal when all goroutines have finished execution.
	waitGroup *sync.WaitGroup
}

const TCP_BUFFER_LEN = 65535
const TCP_WRITE_TIMEOUT = time.Duration(5) * time.Second

func New(logger *device.Logger, remoteSocket *gonet.TCPConn) (*TCPForwarder, error) {
	waitGroup := &sync.WaitGroup{}

	// Create a socket pair, one end of which will be returned to the caller, the other will be used for I/O
	consumerSocket, localSocket, err := socketpair.New("unix")
	if err != nil {
		return nil, err
	}

	forwarder := &TCPForwarder{
		logger:         logger,
		consumerSocket: &consumerSocket,
		localSocket:    &localSocket,
		remoteSocket:   remoteSocket,
		waitGroup:      waitGroup,
	}
	waitGroup.Add(2)
	go forwarder.RoutineHandleInbound(&localSocket, remoteSocket)
	go forwarder.RoutineHandleOutbound(&localSocket, remoteSocket)

	return forwarder, nil
}

// Get socket fd that can be used to read or write data into netstack connection.
func (w *TCPForwarder) GetSocketFd() int {
	return netfd.GetFdFromConn(*w.consumerSocket)
}

func (w *TCPForwarder) Close() {
	// Close all connections. This should release any blocking ReadFromUDP() calls.
	(*w.localSocket).Close()
	w.remoteSocket.Close()

	// Wait for all routines to complete.
	w.waitGroup.Wait()
}

func (w *TCPForwarder) Wait() {
	w.waitGroup.Wait()
}

func (w *TCPForwarder) RoutineHandleInbound(inbound *net.Conn, outbound *gonet.TCPConn) {
	defer w.waitGroup.Done()

	inboundBuffer := make([]byte, TCP_BUFFER_LEN)

	w.logger.Verbosef("tcpforwarder(inbound): listening on %s", (*inbound).LocalAddr().String())
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
			continue
		}

		// Forward the packet over the outbound connection via another WireGuard tunnel.
		_, err = outbound.Write(inboundBuffer[:bytesRead])
		if err != nil {
			w.logger.Errorf("tcpforwarder(inbound): %s", err.Error())
			// todo: handle error
			continue
		}
	}
}

func (w *TCPForwarder) RoutineHandleOutbound(inbound *net.Conn, outbound *gonet.TCPConn) {
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
