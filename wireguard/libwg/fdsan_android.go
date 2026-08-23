//go:build android

/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package main

/*
#include <android/fdsan.h>

// android_fdsan_{get,set}_error_level exist only on API >= 29. <android/fdsan.h>
// hides their declarations below that level (__BIONIC_AVAILABILITY_GUARD(29)) and
// libwg is compiled against API 21, so declare them here as weak symbols: they
// resolve to the real libc functions at load time on API >= 29 and are null on
// older systems. The enum itself is declared unconditionally by the header.
extern enum android_fdsan_error_level android_fdsan_get_error_level(void) __attribute__((weak));
extern enum android_fdsan_error_level android_fdsan_set_error_level(enum android_fdsan_error_level new_level) __attribute__((weak));

static void nym_downgrade_fdsan(void) {
	// libwg also loads on pre-29 devices (plain WG tunnel), where these weak
	// symbols are null; null-check before calling so init() never crashes there.
	if (android_fdsan_get_error_level == 0 || android_fdsan_set_error_level == 0) {
		return;
	}
	// Only ever relax fdsan, never tighten it: if the host already set a lower
	// level, leave it alone.
	if (android_fdsan_get_error_level() > ANDROID_FDSAN_ERROR_LEVEL_WARN_ONCE) {
		android_fdsan_set_error_level(ANDROID_FDSAN_ERROR_LEVEL_WARN_ONCE);
	}
}
*/
import "C"

// The Go runtime closes its file descriptors with direct close(2) syscalls that
// bypass bionic's fdsan instrumentation, so fdsan's per-fd ownership tags drift
// out of sync with the real fd table in any process that embeds Go alongside
// C++ users of unique_fd (netd's FwmarkClient behind VpnService.protect(), and
// libgui/libhwui's graphics Fence handoff). On hardened devices where fdsan is
// FATAL (e.g. GrapheneOS / Pixel with FORCIBLY_ENABLE_MEMORY_TAGGING), that
// drift surfaces as a phantom "close of fd owned by unique_fd" abort that kills
// the whole process — reproduced deterministically the moment split-tunnel
// steering starts protecting one dialed socket per excluded-app flow.
//
// Downgrade the process-wide fdsan error level to WARN_ONCE so the invalid
// ownership assertions log once instead of aborting. This does NOT disable MTE
// (a separate mechanism) and does NOT hide genuine memory-safety bugs; it only
// relaxes an fd-ownership check that cannot be satisfied in a mixed Go/C++ fd
// table. init() runs when libwg is loaded, before any steering fd churn.
func init() {
	C.nym_downgrade_fdsan()
}
