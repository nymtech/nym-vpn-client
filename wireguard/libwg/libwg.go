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
	"runtime"
	"unsafe"

	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
)

const (
	ERROR_GENERAL_FAILURE      = -1
	ERROR_INTERMITTENT_FAILURE = -2
)

type TunnelContext struct {
	Device *device.Device
	Logger *device.Logger
}

// goStringFixed converts a NUL-terminated C string to a Go string without
// reading outside the string's allocation. The standard C.GoString locates
// the terminator with vectorized scans that read up to 31 bytes beyond it and
// before the start of the string; under ARM MTE (16-byte tag granules) those
// out-of-bounds reads hit differently-tagged granules and kill the process
// with SEGV_MTESERR. See https://github.com/mullvad/mullvadvpn-app/pull/6727.
func goStringFixed(cString *C.char) string {
	if cString == nil {
		return ""
	}
	ptr := unsafe.Pointer(cString)
	length := 0
	for *(*byte)(unsafe.Pointer(uintptr(ptr) + uintptr(length))) != 0 {
		length++
	}
	// C.GoStringN copies exactly length bytes and never reads past them.
	return C.GoStringN(cString, C.int(length))
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
