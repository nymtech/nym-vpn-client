//go:build ios || android

/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"

import (
	"bufio"
	"strings"
)

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

	err = tunnel.Device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if err != nil {
		tunnel.Logger.Errorf("Failed to set device configuration\n")
		tunnel.Logger.Errorf("%s\n", err)
		return ERROR_GENERAL_FAILURE
	}

	tunnel.Device.DisableSomeRoamingForBrokenMobileSemantics()

	return 0
}
