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
	"sync"
	"unsafe"

	"golang.org/x/sys/windows"

	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

//export wgTurnOn
func wgTurnOn(cIfaceName *C.char, cRequestedGUID *C.char, cWintunTunnelType *C.char, mtu int, cSettings *C.char, cIfaceNameOut **C.char, cLuidOut *uint64, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)
	if cIfaceNameOut != nil {
		*cIfaceNameOut = nil
	}

	if cIfaceName == nil {
		logger.Errorf("cIfaceName is null\n")
		return ERROR_GENERAL_FAILURE
	}

	if cRequestedGUID == nil {
		logger.Errorf("cRequestedGUID is null\n")
		return ERROR_GENERAL_FAILURE
	}

	if cWintunTunnelType == nil {
		logger.Errorf("cWintunTunnelType is null\n")
		return ERROR_GENERAL_FAILURE
	}

	if cSettings == nil {
		logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}

	settings := C.GoString(cSettings)
	ifaceName := C.GoString(cIfaceName)
	requestedGUID := C.GoString(cRequestedGUID)
	wintunTunnelType := C.GoString(cWintunTunnelType)

	networkId, err := windows.GUIDFromString(requestedGUID)
	if err != nil {
		logger.Errorf("Failed to parse guid: %s\n", err)
		return ERROR_GENERAL_FAILURE
	}

	tun.WintunTunnelType = wintunTunnelType

	logger.Verbosef("Creating interface %s (guid: %s, tunnel-type: %s) and mtu %d", ifaceName, networkId, wintunTunnelType, mtu)

	// WORKAROUND: wrap CreateTUNWithRequestedGUID() into go routine to avoid a panic saying: "runtime: stack split at bad time"
	// See: https://github.com/golang/go/issues/68285
	var wintun tun.Device
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		wintun, err = tun.CreateTUNWithRequestedGUID(ifaceName, &networkId, mtu)
	}()
	wg.Wait()

	if err != nil {
		logger.Errorf("Failed to create tunnel: %s\n", err)
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
