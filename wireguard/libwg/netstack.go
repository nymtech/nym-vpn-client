/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"

import (
	"net/netip"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/forwarders"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
)

type NetTunnelHandle struct {
	*device.Device
	*netstack.Net
	*device.Logger
}

var netTunnelHandles container.Container[NetTunnelHandle]
var udpForwarders container.Container[*forwarders.UDPForwarder]
var tcpForwarders container.Container[*forwarders.TCPForwarder]

func init() {
	netTunnelHandles = container.New[NetTunnelHandle]()
	udpForwarders = container.New[*forwarders.UDPForwarder]()
	tcpForwarders = container.New[*forwarders.TCPForwarder]()
}

//export wgNetTurnOff
func wgNetTurnOff(netTunnelHandle int32) {
	dev, err := netTunnelHandles.Remove(netTunnelHandle)
	if err != nil {
		return
	}
	dev.Close()
}

//export wgNetGetConfig
func wgNetGetConfig(netTunnelHandle int32) *C.char {
	device, err := netTunnelHandles.Get(netTunnelHandle)
	if err != nil {
		return nil
	}
	settings, err := device.IpcGet()
	if err != nil {
		return nil
	}
	return C.CString(settings)
}

//export wgNetStartUDPConnectionProxy
func wgNetStartUDPConnectionProxy(netTunnelHandle int32, listenPort uint16, clientPort uint16, endpoint *C.char, outListenAddr **C.char, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	if outListenAddr == nil {
		logger.Errorf("outListenAddr is null")
		return ERROR_GENERAL_FAILURE
	}

	dev, err := netTunnelHandles.Get(netTunnelHandle)
	if err != nil {
		dev.Errorf("Invalid tunnel handle: %d", netTunnelHandle)
		return ERROR_GENERAL_FAILURE
	}

	addr, err := netip.ParseAddrPort(goStringFixed(endpoint))
	if err != nil {
		dev.Errorf("Failed to parse endpoint: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	forwarderConfig := forwarders.UDPForwarderConfig{
		ListenPort: listenPort,
		ClientPort: clientPort,
		Endpoint:   addr,
	}

	forwarder, err := forwarders.NewUDPForwarder(forwarderConfig, dev.Net, logger)
	if err != nil {
		dev.Errorf("Failed to create udp forwarder: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	index, err := udpForwarders.Insert(forwarder)
	if err != nil {
		dev.Errorf("Failed to store udp forwarder: %v", err)
		forwarder.Close()
		return ERROR_GENERAL_FAILURE
	}

	*outListenAddr = C.CString(forwarder.GetListenAddr().String())

	return index
}

//export wgNetStopUDPConnectionProxy
func wgNetStopUDPConnectionProxy(udpProxyHandle int32) {
	udpForwarder, err := udpForwarders.Remove(udpProxyHandle)
	if err != nil {
		return
	}
	(*udpForwarder).Close()
}

//export wgNetStartTCPConnectionProxy
func wgNetStartTCPConnectionProxy(netTunnelHandle int32, endpoint *C.char, outListenAddr **C.char, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	if outListenAddr == nil {
		logger.Errorf("outListenAddr is null")
		return ERROR_GENERAL_FAILURE
	}

	dev, err := netTunnelHandles.Get(netTunnelHandle)
	if err != nil {
		dev.Errorf("Invalid tunnel handle: %d", netTunnelHandle)
		return ERROR_GENERAL_FAILURE
	}

	addr, err := netip.ParseAddrPort(goStringFixed(endpoint))
	if err != nil {
		dev.Errorf("Failed to parse endpoint: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	forwarder, err := forwarders.NewTCPForwarder(addr, dev.Net, logger)
	if err != nil {
		dev.Errorf("Failed to create tcp forwarder: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	index, err := tcpForwarders.Insert(forwarder)
	if err != nil {
		dev.Errorf("Failed to store tcp forwarder: %v", err)
		forwarder.Close()
		return ERROR_GENERAL_FAILURE
	}

	*outListenAddr = C.CString(forwarder.GetListenAddr().String())

	return index
}

//export wgNetStopTCPConnectionProxy
func wgNetStopTCPConnectionProxy(tcpProxyHandle int32) {
	forwarder, err := tcpForwarders.Remove(tcpProxyHandle)
	if err != nil {
		return
	}
	(*forwarder).Close()
}
