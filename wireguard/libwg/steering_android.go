//go:build android

/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package main

/*
#include <stdint.h>
#include <stdlib.h>

typedef void (*steering_protect_fn)(void *ctx, int32_t fd);
typedef int32_t (*steering_owner_uid_fn)(void *ctx, int32_t protocol, const char *src, const char *dst);

static void call_steering_protect(steering_protect_fn fn, void *ctx, int32_t fd) {
	fn(ctx, fd);
}
static int32_t call_steering_owner_uid(steering_owner_uid_fn fn, void *ctx, int32_t protocol, const char *src, const char *dst) {
	return fn(ctx, protocol, src, dst);
}
*/
import "C"

import (
	"net/netip"
	"strings"
	"unsafe"

	"golang.org/x/sys/unix"

	"github.com/nymtech/nym-vpn-client/wireguard/libwg/container"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/logging"
	"github.com/nymtech/nym-vpn-client/wireguard/libwg/steering"
)

var steeringEngines = container.New[*steering.Engine]()

//export steeringTurnOn
func steeringTurnOn(tunFd int32, innerFd int32, mtu int32,
	excludedUids *C.uint32_t, uidCount int32,
	dnsServers *C.char,
	protectCb C.steering_protect_fn, ownerUidCb C.steering_owner_uid_fn, cbCtx unsafe.Pointer,
	logSink LogSink, logContext LogContext) int32 {

	logger := logging.NewLogger(logSink, logContext)

	// A null callback would otherwise defeat the fail-safe checks further
	// down the call chain (newBypassStack's nil-Protect error, Classifier's
	// nil-callback fail-closed): the Go closures wrapping protectCb/
	// ownerUidCb below are always non-nil regardless of whether the
	// underlying C function pointer is, so invoking a null protectCb would
	// crash (e.g. inside net.Dialer.Control) instead of failing cleanly.
	// Reject here, before Start ever takes ownership of the fds.
	if protectCb == nil || ownerUidCb == nil {
		logger.Errorf("steeringTurnOn: nil callback")
		unix.Close(int(tunFd))
		unix.Close(int(innerFd))
		return ERROR_GENERAL_FAILURE
	}

	var uids []uint32
	if excludedUids != nil && uidCount > 0 {
		uids = append(uids, unsafe.Slice((*uint32)(unsafe.Pointer(excludedUids)), int(uidCount))...)
	}

	var dns []netip.Addr
	if dnsServers != nil {
		for _, s := range strings.Split(C.GoString(dnsServers), ",") {
			if addr, err := netip.ParseAddr(strings.TrimSpace(s)); err == nil {
				dns = append(dns, addr)
			}
		}
	}

	cb := steering.Callbacks{
		Protect: func(fd int32) {
			C.call_steering_protect(protectCb, cbCtx, C.int32_t(fd))
		},
		OwnerUID: func(proto steering.Proto, src, dst netip.AddrPort) int32 {
			cSrc := C.CString(src.String())
			cDst := C.CString(dst.String())
			defer C.free(unsafe.Pointer(cSrc))
			defer C.free(unsafe.Pointer(cDst))
			return int32(C.call_steering_owner_uid(ownerUidCb, cbCtx, C.int32_t(proto), cSrc, cDst))
		},
	}

	engine, err := steering.Start(int(tunFd), int(innerFd), steering.Config{
		ExcludedUIDs:  uids,
		UnderlyingDNS: dns,
		MTU:           int(mtu),
	}, cb, logger)
	if err != nil {
		logger.Errorf("steeringTurnOn: %s", err)
		return ERROR_GENERAL_FAILURE
	}

	handle, err := steeringEngines.Insert(engine)
	if err != nil {
		logger.Errorf("steeringTurnOn: %s", err)
		engine.Stop()
		return ERROR_GENERAL_FAILURE
	}
	return handle
}

//export steeringTurnOff
func steeringTurnOff(handle int32) {
	engine, err := steeringEngines.Remove(handle)
	if err != nil {
		return
	}
	(*engine).Stop()
}
