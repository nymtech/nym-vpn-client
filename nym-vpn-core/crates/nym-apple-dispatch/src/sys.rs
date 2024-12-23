// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Bindings for libdispatch.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::c_char;

use objc2::{
    encode::{Encoding, RefEncode},
    runtime::NSObjectProtocol,
    Message,
};

macro_rules! create_opaque_type {
    ($type_name: ident, $typedef_name: ident) => {
        // Dispatch objects are objc types when compiled with objc compiler.
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $type_name {
            _inner: [u8; 0],
        }

        pub type $typedef_name = *mut $type_name;

        // Safety: dispatch types are internally objects.
        unsafe impl RefEncode for $type_name {
            const ENCODING_REF: Encoding = Encoding::Object;
        }

        // Safety: dispatch types respond to objc messages.
        unsafe impl Message for $type_name {}

        // Safety: dispatch types implement NSObject.
        unsafe impl NSObjectProtocol for $type_name {}
    };
}

create_opaque_type!(dispatch_object_s, dispatch_object_t);
create_opaque_type!(OS_dispatch_queue, dispatch_queue_t);
create_opaque_type!(OS_dispatch_queue_main, dispatch_queue_main_t);
create_opaque_type!(OS_dispatch_queue_attr, dispatch_queue_attr_t);

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

    #[allow(unused)]
    pub fn dispatch_retain(object: dispatch_object_t);
    pub fn dispatch_release(object: dispatch_object_t);

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
