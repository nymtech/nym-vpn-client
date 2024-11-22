/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"

import (
	"net/netip"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/udp_forwarder"
)

type NetTunnelHandle struct {
	*device.Device
	*netstack.Net
	*device.Logger
}

var netTunnelHandles container.Container[NetTunnelHandle]
var udpForwarders container.Container[*udp_forwarder.UDPForwarder]

func init() {
	netTunnelHandles = container.New[NetTunnelHandle]()
	udpForwarders = container.New[*udp_forwarder.UDPForwarder]()
}

//export wgNetTurnOff
func wgNetTurnOff(tunnelHandle int32) {
	dev, err := netTunnelHandles.Remove(tunnelHandle)
	if err != nil {
		return
	}
	dev.Close()
}

//export wgNetGetConfig
func wgNetGetConfig(tunnelHandle int32) *C.char {
	device, err := netTunnelHandles.Get(tunnelHandle)
	if err != nil {
		return nil
	}
	settings, err := device.IpcGet()
	if err != nil {
		return nil
	}
	return C.CString(settings)
}

//export wgNetOpenConnectionThroughTunnel
func wgNetOpenConnectionThroughTunnel(entryTunnelHandle int32, listenPort uint16, clientPort uint16, exitEndpointStr *C.char, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	dev, err := netTunnelHandles.Get(entryTunnelHandle)
	if err != nil {
		dev.Errorf("Invalid tunnel handle: %d", entryTunnelHandle)
		return ERROR_GENERAL_FAILURE
	}

	exitEndpoint, err := netip.ParseAddrPort(C.GoString(exitEndpointStr))
	if err != nil {
		dev.Errorf("Failed to parse endpoint: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	forwarderConfig := udp_forwarder.UDPForwarderConfig{
		ListenPort:   listenPort,
		ClientPort:   clientPort,
		ExitEndpoint: exitEndpoint,
	}

	udpForwarder, err := udp_forwarder.New(forwarderConfig, dev.Net, logger)
	if err != nil {
		dev.Errorf("Failed to create udp forwarder: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	forwarderHandle, err := udpForwarders.Insert(udpForwarder)
	if err != nil {
		dev.Errorf("Failed to store udp forwarder: %v", err)
		udpForwarder.Close()
		return ERROR_GENERAL_FAILURE
	}

	return forwarderHandle
}

//export wgNetCloseConnectionThroughTunnel
func wgNetCloseConnectionThroughTunnel(udpForwarderHandle int32) {
	udpForwarder, err := udpForwarders.Remove(udpForwarderHandle)
	if err != nil {
		return
	}
	(*udpForwarder).Close()
}
