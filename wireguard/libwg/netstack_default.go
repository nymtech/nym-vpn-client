//go:build (darwin || linux || windows) && !android && !ios

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

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun/netstack"
)

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

	err = dev.Up()
	if err != nil {
		logger.Errorf("Failed to set device state to Up: %v", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
	}

	logger.Verbosef("Net device started")

	i, err := netTunnelHandles.Insert(NetTunnelHandle{dev, tnet, logger})
	if err != nil {
		logger.Errorf("Failed to store tunnel: %v", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
	}

	return i
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

	return 0
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
