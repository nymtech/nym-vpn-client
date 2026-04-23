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
	"os"
	"strings"
	"sync"

	"github.com/amnezia-vpn/amneziawg-go/conn"
	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"golang.org/x/sys/unix"
)

// socketTun is a tun.Device backed by a raw file descriptor (e.g. a Unix-domain socket).
// Unlike NativeTun it does NOT call TUNGETIFF, so it is safe to use with non-TUN fds on Android.
type socketTun struct {
	file      *os.File
	mtu       int
	events    chan tun.Event
	closeOnce sync.Once
}

func newSocketTunFromFD(fd int, mtu int) (*socketTun, error) {
	if err := unix.SetNonblock(fd, true); err != nil {
		return nil, err
	}
	file := os.NewFile(uintptr(fd), "wg-proxy-socket")
	st := &socketTun{
		file:   file,
		mtu:    mtu,
		events: make(chan tun.Event, 5),
	}
	st.events <- tun.EventUp
	return st, nil
}

func (st *socketTun) File() *os.File { return st.file }

func (st *socketTun) Name() (string, error) { return "wg-proxy", nil }

func (st *socketTun) MTU() (int, error) { return st.mtu, nil }

func (st *socketTun) Events() <-chan tun.Event { return st.events }

func (st *socketTun) BatchSize() int { return 1 }

func (st *socketTun) Read(bufs [][]byte, sizes []int, offset int) (int, error) {
	n, err := st.file.Read(bufs[0][offset:])
	if err != nil {
		return 0, err
	}
	sizes[0] = n
	return 1, nil
}

func (st *socketTun) Write(bufs [][]byte, offset int) (int, error) {
	for i, buf := range bufs {
		if _, err := st.file.Write(buf[offset:]); err != nil {
			return i, err
		}
	}
	return len(bufs), nil
}

func (st *socketTun) Close() error {
	var err error
	st.closeOnce.Do(func() {
		close(st.events)
		err = st.file.Close()
	})
	return err
}

// wgTurnOnWithProxyFd starts a WireGuard tunnel backed by a raw socket fd (e.g. the wg end of a
// DNS-filter proxy socket pair). Unlike wgTurnOn it does NOT call TUNGETIFF, so it works on
// Android with non-TUN file descriptors. mtu must be the MTU configured for the exit tunnel.
//
//export wgTurnOnWithProxyFd
func wgTurnOnWithProxyFd(cSettings *C.char, fd int, mtu int32, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	if cSettings == nil {
		logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}
	settings := C.GoString(cSettings)

	tunDevice, err := newSocketTunFromFD(fd, int(mtu))
	if err != nil {
		logger.Errorf("wgTurnOnWithProxyFd: failed to create socket tun: %s\n", err)
		unix.Close(fd)
		return ERROR_GENERAL_FAILURE
	}

	dev := device.NewDevice(tunDevice, conn.NewStdNetBind(), logger)

	setErr := dev.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setErr != nil {
		logger.Errorf("%s\n", setErr)
		dev.Close()
		return ERROR_INTERMITTENT_FAILURE
	}

	dev.DisableSomeRoamingForBrokenMobileSemantics()
	dev.Up()

	context := TunnelContext{
		Device: dev,
		Logger: logger,
	}

	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Errorf("%s\n", err)
		dev.Close()
		return ERROR_GENERAL_FAILURE
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

	err = tunnel.Device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if err != nil {
		tunnel.Logger.Errorf("Failed to set device configuration\n")
		tunnel.Logger.Errorf("%s\n", err)
		return ERROR_GENERAL_FAILURE
	}

	tunnel.Device.DisableSomeRoamingForBrokenMobileSemantics()

	return 0
}
