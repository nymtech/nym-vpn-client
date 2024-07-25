//go:build ios

/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2018-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import "C"
import (
	"os"
	"time"
	"unsafe"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"golang.org/x/sys/unix"
	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

//export wgTurnOn
func wgTurnOn(settings *C.char, tunFd int32, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	dupTunFd, err := unix.Dup(int(tunFd))
	if err != nil {
		logger.Errorf("Unable to dup tun fd: %v", err)
		return ERROR_GENERAL_FAILURE
	}

	err = unix.SetNonblock(dupTunFd, true)
	if err != nil {
		logger.Errorf("Unable to set tun fd as non blocking: %v", err)
		unix.Close(dupTunFd)
		return ERROR_GENERAL_FAILURE
	}
	tun, err := tun.CreateTUNFromFile(os.NewFile(uintptr(dupTunFd), "/dev/tun"), 0)
	if err != nil {
		logger.Errorf("Unable to create new tun device from fd: %v", err)
		unix.Close(dupTunFd)
		return ERROR_INTERMITTENT_FAILURE
	}
	logger.Verbosef("Attaching to interface")
	dev := device.NewDevice(tun, conn.NewStdNetBind(), logger)

	err = dev.IpcSet(C.GoString(settings))
	if err != nil {
		logger.Errorf("Unable to set IPC settings: %v", err)
		unix.Close(dupTunFd)
		return ERROR_GENERAL_FAILURE
	}

	dev.DisableSomeRoamingForBrokenMobileSemantics()
	dev.Up()

	logger.Verbosef("Device started")

	context := TunnelContext{
		Device: dev,
		Logger: logger,
	}

	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Errorf("%s", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
	}

	return handle
}

//export wgBumpSockets
func wgBumpSockets(tunnelHandle int32) {
	tunnel, err := tunnels.Get(tunnelHandle)
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

//export wgDisableSomeRoamingForBrokenMobileSemantics
func wgDisableSomeRoamingForBrokenMobileSemantics(tunnelHandle int32) {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return
	}
	tunnel.Device.DisableSomeRoamingForBrokenMobileSemantics()
}
