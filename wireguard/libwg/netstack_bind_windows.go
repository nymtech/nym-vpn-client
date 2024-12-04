/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"
import (
	"golang.org/x/sys/windows"
	"golang.zx2c4.com/wireguard/conn"
)

//export wgNetRebindTunnelSocket
func wgNetRebindTunnelSocket(family uint16, interfaceIndex uint32) {
	netTunnelHandles.ForEach(func(tunnel NetTunnelHandle) {
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
