/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"

import "golang.zx2c4.com/wireguard/conn"

//export wgNetGetSocketV4
func wgNetGetSocketV4(tunnelHandle int32) int32 {
	tunnel, err := netTunnelHandles.Get(tunnelHandle)
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	peek := tunnel.Device.Bind().(conn.PeekLookAtSocketFd)
	fd, err := peek.PeekLookAtSocketFd4()
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	return int32(fd)
}

//export wgNetGetSocketV6
func wgNetGetSocketV6(tunnelHandle int32) int32 {
	tunnel, err := netTunnelHandles.Get(tunnelHandle)
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	peek := tunnel.Device.Bind().(conn.PeekLookAtSocketFd)
	fd, err := peek.PeekLookAtSocketFd6()
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	return int32(fd)
}
