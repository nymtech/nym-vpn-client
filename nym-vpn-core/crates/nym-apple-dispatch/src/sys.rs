// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Bindings for libdispatch.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::c_char;

use objc2::runtime::ProtocolObject;
use objc2_foundation::NSObjectProtocol;

// Dispatch objects are objc types when compiled with objc compiler.
pub type dispatch_object_s = ProtocolObject<dyn NSObjectProtocol>;
#[allow(unused)]
pub type dispatch_object_t = *mut dispatch_object_s;

pub type OS_dispatch_queue = dispatch_object_s;
pub type dispatch_queue_t = *mut OS_dispatch_queue;

pub type OS_dispatch_queue_main = dispatch_object_s;
pub type dispatch_queue_main_t = *mut OS_dispatch_queue_main;

pub type OS_dispatch_queue_attr = dispatch_object_s;
pub type dispatch_queue_attr_t = *mut OS_dispatch_queue_attr;

#[cfg_attr(
    any(target_os = "macos", target_os = "ios"),
    link(name = "System", kind = "dylib")
)]
#[cfg_attr(
    not(any(target_os = "macos", target_os = "ios")),
    link(name = "dispatch", kind = "dylib")
)]
extern "C" {
    static _dispatch_main_q: dispatch_object_s;

    pub fn dispatch_queue_create(
        label: *const c_char,
        attr: dispatch_queue_attr_t,
    ) -> dispatch_queue_t;
    pub fn dispatch_queue_get_label(queue: dispatch_queue_t) -> *const c_char;
}

pub fn dispatch_get_main_queue() -> dispatch_queue_main_t {
    unsafe { &_dispatch_main_q as *const _ as dispatch_queue_main_t }
}

pub const DISPATCH_QUEUE_SERIAL: dispatch_queue_attr_t =
    std::ptr::null_mut() as dispatch_queue_attr_t;
