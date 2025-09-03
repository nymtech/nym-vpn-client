/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"
import (
	"net/netip"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/tcp_forwarder"
)

//export wgNetOpenTCPSocketThroughTunnel
func wgNetOpenTCPSocketThroughTunnel(entryTunnelHandle int32, endpoint *C.char, logSink LogSink, logContext LogContext) int {
	logger := logging.NewLogger(logSink, logContext)

	dev, err := netTunnelHandles.Get(entryTunnelHandle)
	if err != nil {
		dev.Errorf("Invalid tunnel handle: %d", entryTunnelHandle)
		return ERROR_GENERAL_FAILURE
	}

	addr, err := netip.ParseAddrPort(C.GoString(endpoint))
	if err != nil {
		dev.Errorf("Failed to parse endpoint: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	vnetConn, err := dev.DialTCPAddrPort(addr)
	if err != nil {
		dev.Errorf("Failed to connect to %s: %v", addr, err)
		return ERROR_GENERAL_FAILURE
	}

	forwarder, err := tcp_forwarder.New(logger, vnetConn)
	if err != nil {
		dev.Errorf("Failed to create tcp forwarder: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	return forwarder.GetSocketFd()
}
