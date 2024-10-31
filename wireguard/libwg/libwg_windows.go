/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

// #include <stdlib.h>
import "C"

import (
	"bufio"
	"strings"
	"unsafe"

	"golang.org/x/sys/windows"

	"github.com/amnezia-vpn/amneziawg-go/conn"
	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

//export wgTurnOn
func wgTurnOn(cIfaceName *C.char, mtu int, cSettings *C.char, cIfaceNameOut **C.char, cLuidOut *uint64, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)
	if cIfaceNameOut != nil {
		*cIfaceNameOut = nil
	}

	if cIfaceName == nil {
		logger.Errorf("cIfaceName is null\n")
		return ERROR_GENERAL_FAILURE
	}

	if cSettings == nil {
		logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}

	settings := C.GoString(cSettings)
	ifaceName := C.GoString(cIfaceName)

	// {AFE43773-E1F8-4EBB-8536-576AB86AFE9A}
	networkId := windows.GUID{0xafe43773, 0xe1f8, 0x4ebb, [8]byte{0x85, 0x36, 0x57, 0x6a, 0xb8, 0x6a, 0xfe, 0x9a}}

	tun.WintunTunnelType = "Mullvad"

	wintun, err := tun.CreateTUNWithRequestedGUID(ifaceName, &networkId, mtu)
	if err != nil {
		logger.Errorf("Failed to create tunnel\n")
		logger.Errorf("%s\n", err)
		return ERROR_INTERMITTENT_FAILURE
	}

	nativeTun := wintun.(*tun.NativeTun)

	actualInterfaceName, err := nativeTun.Name()
	if err != nil {
		nativeTun.Close()
		logger.Errorf("Failed to determine name of wintun adapter\n")
		return ERROR_GENERAL_FAILURE
	}
	if actualInterfaceName != ifaceName {
		// WireGuard picked a different name for the adapter than the one we expected.
		// This indicates there is already an adapter with the name we intended to use.
		logger.Verbosef("Failed to create adapter with specific name\n")
	}

	device := device.NewDevice(wintun, conn.NewDefaultBind(), logger)

	setError := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		logger.Errorf("Failed to set device configuration\n")
		logger.Errorf("%s\n", setError)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	device.Up()

	context := TunnelContext{
		Device: device,
		Logger: logger,
	}

	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Errorf("%s\n", err)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	if cIfaceNameOut != nil {
		*cIfaceNameOut = C.CString(actualInterfaceName)
	}
	if cLuidOut != nil {
		*cLuidOut = nativeTun.LUID()
	}

	return handle
}

//export wgSetConfig
func wgSetConfig(tunnelHandle int32, cSettings *C.char) int32 {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	if cSettings == nil {
		tunnel.Logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}
	settings := C.GoString(cSettings)

	setError := tunnel.Device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		tunnel.Logger.Errorf("Failed to set device configuration\n")
		tunnel.Logger.Errorf("%s\n", setError)
		return ERROR_GENERAL_FAILURE
	}
	return 0
}

//export wgRebindTunnelSocket
func wgRebindTunnelSocket(family uint16, interfaceIndex uint32) {
	tunnels.ForEach(func(tunnel TunnelContext) {
		blackhole := (interfaceIndex == 0)
		bind := tunnel.Device.Bind().(conn.BindSocketToInterface)

		if family == windows.AF_INET {
			tunnel.Logger.Verbosef("Binding v4 socket to interface %d (blackhole=%v)\n", interfaceIndex, blackhole)
			err := bind.BindSocketToInterface4(interfaceIndex, blackhole)
			if err != nil {
				tunnel.Logger.Verbosef("%s\n", err)
			}
		} else if family == windows.AF_INET6 {
			tunnel.Logger.Verbosef("Binding v6 socket to interface %d (blackhole=%v)\n", interfaceIndex, blackhole)
			err := bind.BindSocketToInterface6(interfaceIndex, blackhole)
			if err != nil {
				tunnel.Logger.Verbosef("%s\n", err)
			}
		}
	})
}
