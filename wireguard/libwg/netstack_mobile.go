//go:build ios || android

/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"

import (
	"net/netip"
	"strings"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/udp_forwarder"
	"github.com/amnezia-vpn/amneziawg-go/conn"
	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
)

type netTunnelHandle struct {
	*device.Device
	*netstack.Net
	*device.Logger
}

var netTunnelHandles container.Container[netTunnelHandle]
var udpForwarders container.Container[*udp_forwarder.UDPForwarder]

func init() {
	netTunnelHandles = container.New[netTunnelHandle]()
	udpForwarders = container.New[*udp_forwarder.UDPForwarder]()
}

//export wgNetTurnOn
func wgNetTurnOn(localAddresses *C.char, dnsAddresses *C.char, mtu int, settings *C.char, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	// Parse comma separated list of IP addresses
	tunAddrs, err := parseIPAddrs(C.GoString(localAddresses))
	if err != nil {
		logger.Errorf("Failed to parse local addresses: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	// Parse comma separated list of DNS addresses
	dnsAddrs, err := parseIPAddrs(C.GoString(dnsAddresses))
	if err != nil {
		logger.Errorf("Failed to parse dns addresses: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	tun, tnet, err := netstack.CreateNetTUN(tunAddrs, dnsAddrs, mtu)
	if err != nil {
		logger.Errorf("Failed to create net tun: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	dev := device.NewDevice(
		tun,
		conn.NewDefaultBind(),
		logger,
	)
	if dev == nil {
		logger.Errorf("Failed to create device")
		return ERROR_GENERAL_FAILURE
	}

	err = dev.IpcSet(C.GoString(settings))
	if err != nil {
		logger.Errorf("Unable to set IPC settings: %v", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
	}

	dev.DisableSomeRoamingForBrokenMobileSemantics()
	err = dev.Up()
	if err != nil {
		logger.Errorf("Failed to set device state to Up: %v", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
	}

	logger.Verbosef("Net device started")

	i, err := netTunnelHandles.Insert(netTunnelHandle{dev, tnet, logger})
	if err != nil {
		logger.Errorf("Failed to store tunnel: %v", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
	}

	return i
}

//export wgNetTurnOff
func wgNetTurnOff(tunnelHandle int32) {
	dev, err := netTunnelHandles.Remove(tunnelHandle)
	if err != nil {
		return
	}
	dev.Close()
}

//export wgNetSetConfig
func wgNetSetConfig(tunnelHandle int32, settings *C.char) int64 {
	dev, err := netTunnelHandles.Get(tunnelHandle)
	if err != nil {
		return 0
	}
	err = dev.IpcSet(C.GoString(settings))
	if err != nil {
		dev.Errorf("Unable to set IPC settings: %v", err)
		if ipcErr, ok := err.(*device.IPCError); ok {
			return ipcErr.ErrorCode()
		}
		return ERROR_GENERAL_FAILURE
	}

	dev.DisableSomeRoamingForBrokenMobileSemantics()

	return 0
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

// Parse a list of comma-separated IP addresses into array of netip.Addr structs.
func parseIPAddrs(input string) ([]netip.Addr, error) {
	addrs := []netip.Addr{}
	for _, s := range strings.Split(input, ",") {
		addr, err := netip.ParseAddr(strings.TrimSpace(s))
		if err != nil {
			return addrs, err
		}
		addrs = append(addrs, addr)
	}
	return addrs, nil
}
