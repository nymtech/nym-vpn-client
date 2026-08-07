// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Bindings for the libwg steering engine: routes split-tunnel-excluded apps'
//! traffic directly (bypassing the tunnel) so they keep connectivity under
//! Android VPN lockdown.

use std::{
    ffi::{CStr, CString, c_char, c_void},
    net::IpAddr,
    os::fd::{IntoRawFd, OwnedFd, RawFd},
    sync::Arc,
};

use crate::{Error, LoggingCallback, Result, wireguard_go::wg_logger_callback};

/// Callbacks the steering engine invokes on the Rust side to protect its own
/// sockets from being routed back into the tunnel, and to resolve the
/// originating UID of a flow so it can be matched against the excluded-UID
/// set.
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

/// Heap-allocated context handed to the Go side as an opaque `void*`, and
/// passed back on every callback invocation.
struct CallbackCtx {
    callbacks: Arc<dyn SteeringCallbacks>,
}

/// # Safety
/// Called by the Go engine with the `ctx` pointer that `Steering::start`
/// registered; that pointer is a live `*const CallbackCtx` for as long as
/// the engine has not returned from `steeringTurnOff`.
unsafe extern "C" fn protect_trampoline(ctx: *mut c_void, fd: i32) {
    let ctx = unsafe { &*(ctx as *const CallbackCtx) };
    ctx.callbacks.protect(fd);
}

/// # Safety
/// Same contract as [`protect_trampoline`]. `src`/`dst` are non-null,
/// NUL-terminated C strings owned by the caller for the duration of the call.
unsafe extern "C" fn owner_uid_trampoline(
    ctx: *mut c_void,
    protocol: i32,
    src: *const c_char,
    dst: *const c_char,
) -> i32 {
    let ctx = unsafe { &*(ctx as *const CallbackCtx) };
    let src = unsafe { CStr::from_ptr(src) }.to_string_lossy();
    let dst = unsafe { CStr::from_ptr(dst) }.to_string_lossy();
    ctx.callbacks.owner_uid(protocol, &src, &dst)
}

/// Handle to a running steering engine.
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
unsafe impl Send for Steering {}
unsafe impl Sync for Steering {}

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

        let dns_csv = config
            .underlying_dns
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(",");
        // Pass NULL rather than an empty string when there are no DNS
        // servers, matching the cgo export's "may be NULL" contract.
        let dns_cstring = if dns_csv.is_empty() {
            None
        } else {
            Some(CString::new(dns_csv).map_err(|_| Error::ConvertToCString("underlying dns"))?)
        };
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

    /// Stop the steering engine.
    ///
    /// `steeringTurnOff` blocks until every goroutine of the engine
    /// (including callback invocations) has joined, so it is sound to free
    /// `ctx` immediately afterwards.
    pub fn stop(self) {
        unsafe {
            steeringTurnOff(self.handle);
            drop(Box::from_raw(self.ctx));
        }
        std::mem::forget(self);
    }
}

impl Drop for Steering {
    fn drop(&mut self) {
        unsafe {
            steeringTurnOff(self.handle);
            drop(Box::from_raw(self.ctx));
        }
    }
}

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

    /// Stop the steering engine started by `steeringTurnOn`. Blocks until
    /// all of the engine's goroutines (including any in-flight callback
    /// invocation) have joined.
    unsafe fn steeringTurnOff(handle: i32);
}
