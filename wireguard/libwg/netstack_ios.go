/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"
import "time"

//export wgNetBumpSockets
func wgNetBumpSockets(tunnelHandle int32) {
	tunnel, err := netTunnelHandles.Get(tunnelHandle)
	if err != nil {
		return
	}
	go func() {
		for i := 0; i < 10; i++ {
			err := tunnel.Device.BindUpdate()
			if err == nil {
				tunnel.Device.SendKeepalivesToPeersWithCurrentKeypair()
				return
			}
			tunnel.Logger.Errorf("Unable to update bind, try %d: %v", i+1, err)
			time.Sleep(time.Second / 2)
		}
		tunnel.Logger.Errorf("Gave up trying to update bind; tunnel is likely dysfunctional")
	}()
}
