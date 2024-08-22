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
	"bytes"
	"net"
	"runtime"
	"unsafe"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
	"golang.zx2c4.com/wireguard/device"
)

const (
	ERROR_GENERAL_FAILURE      = -1
	ERROR_INTERMITTENT_FAILURE = -2
)

type TunnelContext struct {
	Device *device.Device
	Uapi   net.Listener
	Logger *device.Logger
}

var tunnels container.Container[TunnelContext]

func init() {
	tunnels = container.New[TunnelContext]()
}

//export wgTurnOff
func wgTurnOff(tunnelHandle int32) {
	{
		tunnel, err := tunnels.Remove(tunnelHandle)
		if err != nil {
			return
		}
		tunnel.Device.Close()
	}
	// Calling twice convinces the GC to release NOW.
	runtime.GC()
	runtime.GC()
}

//export wgGetConfig
func wgGetConfig(tunnelHandle int32) *C.char {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return nil
	}
	settings := new(bytes.Buffer)
	writer := bufio.NewWriter(settings)
	if err := tunnel.Device.IpcGetOperation(writer); err != nil {
		tunnel.Logger.Errorf("Failed to get config for tunnel: %s\n", err)
		return nil
	}
	writer.Flush()
	return C.CString(settings.String())
}

//export wgFreePtr
func wgFreePtr(ptr unsafe.Pointer) {
	C.free(ptr)
}

func main() {}
