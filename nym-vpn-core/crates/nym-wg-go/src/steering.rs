// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Bindings for the libwg steering engine: routes split-tunnel-excluded apps'
//! traffic directly (bypassing the tunnel) so they keep connectivity under
//! Android VPN lockdown.
//!
//! The `steeringTurnOn`/`steeringTurnOff` cgo exports only exist in libwg's
//! Android build, so everything that touches that FFI surface is gated
//! `#[cfg(target_os = "android")]`. The pure, non-FFI pieces (config types,
//! the DNS-list-to-CSV conversion) have no platform dependency and are left
//! unconditional so they compile and unit-test on the host.

use std::{ffi::CString, net::IpAddr};

#[cfg(target_os = "android")]
use std::{
    ffi::{CStr, c_char, c_void},
    os::fd::{IntoRawFd, OwnedFd, RawFd},
    sync::Arc,
};

#[cfg(target_os = "android")]
use crate::{Error, LoggingCallback, Result, wireguard_go::wg_logger_callback};

/// Callbacks the steering engine invokes on the Rust side to protect its own
/// sockets from being routed back into the tunnel, and to resolve the
/// originating UID of a flow so it can be matched against the excluded-UID
/// set.
///
/// # Panics
/// Implementations must not panic: they are invoked from Go goroutines through
/// an `extern "C"` trampoline, and unwinding across that boundary is undefined
/// behaviour. The trampolines do catch unwinds as a last-resort backstop (a
/// caught panic degrades to "socket left unprotected" / "owner unknown"), but
/// that safety net exists to avoid aborting the process, not as a supported
/// error-reporting channel.
#[cfg(target_os = "android")]
pub trait SteeringCallbacks: Send + Sync + 'static {
    /// Protect `fd` (a raw socket owned by the steering engine) from the VPN,
    /// mirroring `VpnService.protect()`.
    fn protect(&self, fd: RawFd);

    /// protocol: 6 = TCP, 17 = UDP. `src`/`dst` are `netip.AddrPort` strings,
    /// e.g. "1.2.3.4:443" or "[fd00::1]:53". Return the owning UID, or -1
    /// when unknown.
    fn owner_uid(&self, protocol: i32, src: &str, dst: &str) -> i32;
}

/// Steering engine configuration.
pub struct SteeringConfig {
    pub mtu: u16,
    pub excluded_uids: Vec<u32>,
    pub underlying_dns: Vec<IpAddr>,
}

/// Convert the underlying-DNS list into the comma-separated C string that
/// `steeringTurnOn`'s `dns_servers` parameter expects, or `None` (mapped to a
/// NULL pointer by the caller) when there are no DNS servers -- matching the
/// cgo export's documented "may be NULL" contract, and avoiding an
/// allocation for the common no-DNS case.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn dns_servers_csv(dns: &[IpAddr]) -> Option<CString> {
    if dns.is_empty() {
        return None;
    }
    let csv = dns
        .iter()
        .map(|ip| ip.to_string())
        .collect::<Vec<_>>()
        .join(",");
    // SAFETY (infallibility): `IpAddr`'s `Display` only ever emits digits,
    // ASCII letters, '.', ':', and '%', never a NUL byte, so `CString::new`
    // cannot fail here.
    Some(CString::new(csv).expect("IP address strings never contain NUL"))
}

/// Heap-allocated context handed to the Go side as an opaque `void*`, and
/// passed back on every callback invocation.
#[cfg(target_os = "android")]
struct CallbackCtx {
    callbacks: Arc<dyn SteeringCallbacks>,
}

/// # Safety
/// Called by the Go engine with the `ctx` pointer that `Steering::start`
/// registered; that pointer is a live `*const CallbackCtx` for as long as
/// the engine has not returned from `steeringTurnOff`.
#[cfg(target_os = "android")]
unsafe extern "C" fn protect_trampoline(ctx: *mut c_void, fd: i32) {
    // Unwinding out of an `extern "C"` function called from Go is UB, so a
    // panicking implementation must be contained here. Failing to protect a
    // socket is not fail-open: the dial then goes back into the tunnel (or
    // fails), it never leaks around the VPN.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = unsafe { &*(ctx as *const CallbackCtx) };
        ctx.callbacks.protect(fd);
    }));
    if result.is_err() {
        tracing::error!("steering protect callback panicked, socket left unprotected");
    }
}

/// # Safety
/// Same contract as [`protect_trampoline`]. `src`/`dst` are non-null,
/// NUL-terminated C strings owned by the caller for the duration of the call.
#[cfg(target_os = "android")]
unsafe extern "C" fn owner_uid_trampoline(
    ctx: *mut c_void,
    protocol: i32,
    src: *const c_char,
    dst: *const c_char,
) -> i32 {
    // See `protect_trampoline`: a panic must never unwind into Go. -1 is the
    // engine's "owner unknown" value, which fails closed (the flow stays in
    // the tunnel).
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = unsafe { &*(ctx as *const CallbackCtx) };
        let src = unsafe { CStr::from_ptr(src) }.to_string_lossy();
        let dst = unsafe { CStr::from_ptr(dst) }.to_string_lossy();
        ctx.callbacks.owner_uid(protocol, &src, &dst)
    }));
    result.unwrap_or_else(|_| {
        tracing::error!("steering owner_uid callback panicked, treating owner as unknown");
        -1
    })
}

/// Handle to a running steering engine.
#[cfg(target_os = "android")]
pub struct Steering {
    handle: i32,
    // Kept alive for the lifetime of the engine; freed in stop()/drop() only
    // after steeringTurnOff has fully quiesced the engine, so no callback can
    // observe it dangling.
    ctx: *mut CallbackCtx,
}

// SAFETY: `ctx` is only ever dereferenced by the Go engine's callback
// trampolines, which access it through `&CallbackCtx` and never mutate it;
// the wrapped `Arc<dyn SteeringCallbacks>` is itself Send + Sync.
#[cfg(target_os = "android")]
unsafe impl Send for Steering {}
#[cfg(target_os = "android")]
unsafe impl Sync for Steering {}

#[cfg(target_os = "android")]
impl Steering {
    /// Start the steering engine.
    ///
    /// Consumes `tun_fd` (the real TUN device fd) and returns the engine
    /// handle plus the outer end of a `SOCK_DGRAM` socketpair that
    /// downstream code must use in place of the TUN device: the engine reads
    /// tunnel-bound packets from the inner end and, for flows matching an
    /// excluded UID, steers them directly instead of forwarding them to
    /// `tun_fd`.
    ///
    /// # FD ownership
    /// `steeringTurnOn`'s Go side closes *both* `tun_fd` and the inner
    /// socketpair fd on every code path (success and failure). This function
    /// therefore transfers ownership of both fds into the call via
    /// `into_raw_fd` and must never close either afterwards, even when the
    /// call returns a negative error code.
    pub fn start(
        tun_fd: OwnedFd,
        config: SteeringConfig,
        callbacks: Arc<dyn SteeringCallbacks>,
    ) -> Result<(Self, OwnedFd)> {
        let (outer, inner) =
            std::os::unix::net::UnixDatagram::pair().map_err(Error::CreateSteeringSocketPair)?;

        let dns_cstring = dns_servers_csv(&config.underlying_dns);
        let dns_ptr = dns_cstring
            .as_ref()
            .map_or(std::ptr::null(), |c| c.as_ptr());

        let ctx = Box::into_raw(Box::new(CallbackCtx { callbacks }));

        // SAFETY: `tun_fd`/`inner` are valid, open fds whose ownership is
        // unconditionally transferred to the Go side by this call (see FD
        // ownership note above). `protect_trampoline`/`owner_uid_trampoline`
        // are valid extern "C" function pointers for the lifetime of the
        // process. `ctx` is a live heap allocation freed exactly once below,
        // after the engine is confirmed stopped (either immediately here on
        // start failure, or in `stop`/`Drop` after `steeringTurnOff`).
        let handle = unsafe {
            steeringTurnOn(
                tun_fd.into_raw_fd(),
                inner.into_raw_fd(),
                i32::from(config.mtu),
                config.excluded_uids.as_ptr(),
                config.excluded_uids.len() as i32,
                dns_ptr,
                protect_trampoline,
                owner_uid_trampoline,
                ctx as *mut c_void,
                wg_logger_callback,
                std::ptr::null_mut(),
            )
        };
        if handle < 0 {
            // Engine never started: no callback can fire, so it's safe (and
            // necessary) to reclaim ctx here to avoid leaking it.
            drop(unsafe { Box::from_raw(ctx) });
            return Err(Error::StartSteering(handle));
        }
        // SAFETY: `UnixDatagram` -> `OwnedFd` transfers ownership of the
        // outer socketpair end; `into_raw_fd`'d fds above are never closed
        // by us again.
        Ok((Self { handle, ctx }, OwnedFd::from(outer)))
    }

    /// Whether the engine is still forwarding packets.
    ///
    /// Returns `false` once one of the engine's packet pumps has died for a
    /// reason other than an orderly stop. That is unrecoverable: traffic no
    /// longer moves in at least one direction, so the caller must tear the
    /// tunnel down instead of leaving a silent blackhole behind a "Connected"
    /// state.
    pub fn is_alive(&self) -> bool {
        // SAFETY: `handle` is a live engine handle for as long as `self`
        // exists (`stop`/`Drop` are the only paths that retire it, and both
        // consume/finalise `self`). The Go side looks the handle up under its
        // own lock and returns 0 for an unknown handle.
        unsafe { steeringIsAlive(self.handle) != 0 }
    }

    /// Stop the steering engine.
    ///
    /// `steeringTurnOff` blocks until every goroutine of the engine
    /// (including callback invocations) has joined, so it is sound to free
    /// `ctx` immediately afterwards.
    ///
    /// Wrapping `self` in `ManuallyDrop` (rather than tearing down and then
    /// calling `mem::forget`) keeps the exactly-once invariant even if the
    /// `SteeringCallbacks` destructor invoked by dropping the boxed `ctx`
    /// panics: `ManuallyDrop::drop` is a no-op, so unwinding out of this
    /// function can never re-enter `Steering::drop` and repeat
    /// `steeringTurnOff`/`Box::from_raw` on the same handle/pointer.
    pub fn stop(self) {
        let this = std::mem::ManuallyDrop::new(self);
        unsafe {
            steeringTurnOff(this.handle);
            drop(Box::from_raw(this.ctx));
        }
    }
}

#[cfg(target_os = "android")]
impl Drop for Steering {
    fn drop(&mut self) {
        unsafe {
            steeringTurnOff(self.handle);
            drop(Box::from_raw(self.ctx));
        }
    }
}

#[cfg(target_os = "android")]
unsafe extern "C" {
    /// Start the steering engine. Takes ownership of both `tun_fd` and
    /// `inner_fd` unconditionally: the Go side closes both on every return
    /// path, success or failure.
    ///
    /// Returns a non-negative handle on success, or a negative error code.
    unsafe fn steeringTurnOn(
        tun_fd: i32,
        inner_fd: i32,
        mtu: i32,
        excluded_uids: *const u32,
        uid_count: i32,
        dns_servers: *const c_char,
        protect_cb: unsafe extern "C" fn(*mut c_void, i32),
        owner_uid_cb: unsafe extern "C" fn(*mut c_void, i32, *const c_char, *const c_char) -> i32,
        cb_ctx: *mut c_void,
        log_sink: LoggingCallback,
        log_context: *mut c_void,
    ) -> i32;

    /// Report whether the engine behind `handle` is still forwarding packets:
    /// 1 = alive, 0 = a packet pump died, or the handle is unknown.
    unsafe fn steeringIsAlive(handle: i32) -> i32;

    /// Stop the steering engine started by `steeringTurnOn`. Blocks until
    /// all of the engine's goroutines (including any in-flight callback
    /// invocation) have joined.
    unsafe fn steeringTurnOff(handle: i32);
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[test]
    fn dns_servers_csv_empty_is_none() {
        assert!(dns_servers_csv(&[]).is_none());
    }

    #[test]
    fn dns_servers_csv_single_entry() {
        let dns = [IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))];
        let csv = dns_servers_csv(&dns).expect("non-empty input must yield Some");
        assert_eq!(csv.to_str().unwrap(), "1.1.1.1");
    }

    #[test]
    fn dns_servers_csv_preserves_order_for_v4_and_v6_mix() {
        let dns = [
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            IpAddr::V6(Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 1)),
        ];
        let csv = dns_servers_csv(&dns).expect("non-empty input must yield Some");
        assert_eq!(csv.to_str().unwrap(), "1.2.3.4,fd00::1");
    }
}
