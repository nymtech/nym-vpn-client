/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

import (
	"C"
	"bufio"
	"os"
	"strings"
	"sync"
	"unsafe"

	"golang.org/x/sys/unix"

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
func wgTurnOn(cSettings *C.char, fd int, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	if cSettings == nil {
		logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}
	settings := goStringFixed(cSettings)

	tunDevice, _, err := tun.CreateUnmonitoredTUNFromFD(fd)
	if err != nil {
		logger.Errorf("%s\n", err)
		unix.Close(fd)
		if err.Error() == "bad file descriptor" {
			return ERROR_INTERMITTENT_FAILURE
		}
		return ERROR_GENERAL_FAILURE
	}

	device := device.NewDevice(tunDevice, conn.NewStdNetBind(), logger)

	setErr := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setErr != nil {
		logger.Errorf("%s\n", setErr)
		device.Close()
		return ERROR_INTERMITTENT_FAILURE
	}

	device.DisableSomeRoamingForBrokenMobileSemantics()
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

	return handle
}

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
	settings := goStringFixed(cSettings)

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

//export wgGetSocketV4
func wgGetSocketV4(tunnelHandle int32) int32 {
	tunnel, err := tunnels.Get(tunnelHandle)
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

//export wgGetSocketV6
func wgGetSocketV6(tunnelHandle int32) int32 {
	tunnel, err := tunnels.Get(tunnelHandle)
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
