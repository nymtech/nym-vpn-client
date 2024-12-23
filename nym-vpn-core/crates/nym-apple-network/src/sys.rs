// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Bindings for network framework.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, c_int};

use nix::sys::socket::sockaddr;
use objc2::{
    encode::{Encoding, RefEncode},
    runtime::NSObjectProtocol,
    Message,
};

use nym_apple_dispatch::dispatch_queue_t;

macro_rules! create_opaque_type {
    ($type_name: ident, $typedef_name: ident) => {
        // NW objects are objc types when compiled with objc compiler.
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $type_name {
            _inner: [u8; 0],
        }

        pub type $typedef_name = *mut $type_name;

        // Safety: NW types are internally objects.
        unsafe impl RefEncode for $type_name {
            const ENCODING_REF: Encoding = Encoding::Object;
        }

        // Safety: NW types respond to objc messages.
        unsafe impl Message for $type_name {}

        // Safety: NW types implement NSObject.
        unsafe impl NSObjectProtocol for $type_name {}
    };
}

create_opaque_type!(OS_nw_object, nw_object_t);
create_opaque_type!(OS_nw_path_monitor, nw_path_monitor_t);
create_opaque_type!(OS_nw_path, nw_path_t);
create_opaque_type!(OS_nw_interface, nw_interface_t);
create_opaque_type!(OS_nw_endpoint, nw_endpoint_t);

pub type nw_path_monitor_update_handler_t = block2::Block<dyn Fn(nw_path_t)>;
pub type nw_path_monitor_cancel_handler_t = block2::Block<dyn Fn()>;
pub type nw_path_status_t = c_int;
pub type nw_path_enumerate_interfaces_block_t =
    block2::Block<dyn Fn(nw_interface_t) -> objc2::runtime::Bool>;
pub type nw_path_enumerate_gateways_block_t =
    block2::Block<dyn Fn(nw_endpoint_t) -> objc2::runtime::Bool>;

pub type nw_path_status_type_t = c_int;
pub const nw_path_status_invalid: nw_path_status_type_t = 0;
pub const nw_path_status_satisfied: nw_path_status_type_t = 1;
pub const nw_path_status_unsatisfied: nw_path_status_type_t = 2;
pub const nw_path_status_satisfiable: nw_path_status_type_t = 3;

pub type nw_interface_type_t = c_int;
pub const nw_interface_type_other: nw_endpoint_type_t = 0;
pub const nw_interface_type_wifi: nw_endpoint_type_t = 1;
pub const nw_interface_type_cellular: nw_endpoint_type_t = 2;
pub const nw_interface_type_wired: nw_endpoint_type_t = 3;
pub const nw_interface_type_loopback: nw_endpoint_type_t = 4;

pub type nw_endpoint_type_t = c_int;
pub const nw_endpoint_type_invalid: nw_endpoint_type_t = 0;
pub const nw_endpoint_type_address: nw_endpoint_type_t = 1;
pub const nw_endpoint_type_host: nw_endpoint_type_t = 2;
pub const nw_endpoint_type_bonjour_service: nw_endpoint_type_t = 3;
pub const nw_endpoint_type_url: nw_endpoint_type_t = 4;

#[link(name = "Network", kind = "framework")]
unsafe extern "C" {
    pub fn nw_retain(object: nw_object_t);
    pub fn nw_release(object: nw_object_t);

    pub fn nw_path_monitor_create() -> nw_path_monitor_t;
    pub fn nw_path_monitor_create_with_type(
        required_interface_type: nw_interface_type_t,
    ) -> nw_path_monitor_t;
    pub fn nw_path_monitor_prohibit_interface_type(
        monitor: nw_path_monitor_t,
        interface_type: nw_interface_type_t,
    );
    pub fn nw_path_monitor_set_queue(monitor: nw_path_monitor_t, dispatch_queue: dispatch_queue_t);
    pub fn nw_path_monitor_set_update_handler(
        monitor: nw_path_monitor_t,
        update_handler: &nw_path_monitor_update_handler_t,
    );
    pub fn nw_path_monitor_set_cancel_handler(
        monitor: nw_path_monitor_t,
        update_handler: &nw_path_monitor_cancel_handler_t,
    );
    pub fn nw_path_monitor_start(monitor: nw_path_monitor_t);
    pub fn nw_path_monitor_cancel(monitor: nw_path_monitor_t);

    pub fn nw_path_get_status(path: nw_path_t) -> nw_path_status_t;
    pub fn nw_path_uses_interface_type(
        path: nw_path_t,
        interface_type: nw_interface_type_t,
    ) -> objc2::ffi::BOOL;
    pub fn nw_path_is_equal(path: nw_path_t, other_path: nw_path_t) -> objc2::ffi::BOOL;
    pub fn nw_path_enumerate_interfaces(
        path: nw_path_t,
        enumerate_block: &nw_path_enumerate_interfaces_block_t,
    );
    pub fn nw_path_enumerate_gateways(
        path: nw_path_t,
        enumerate_block: &nw_path_enumerate_gateways_block_t,
    );
    pub fn nw_interface_get_type(interface: nw_interface_t) -> nw_interface_type_t;
    pub fn nw_interface_get_name(interface: nw_interface_t) -> *const c_char;
    pub fn nw_interface_get_index(interface: nw_interface_t) -> u32;

    pub fn nw_endpoint_create_host(host: *const c_char, port: *const c_char) -> nw_endpoint_t;
    pub fn nw_endpoint_get_type(endpoint: nw_endpoint_t) -> nw_endpoint_type_t;
    pub fn nw_endpoint_get_hostname(endpoint: nw_endpoint_t) -> *const c_char;
    pub fn nw_endpoint_get_port(endpoint: nw_endpoint_t) -> u16;

    pub fn nw_endpoint_create_address(host: *const sockaddr) -> nw_endpoint_t;
    pub fn nw_endpoint_get_address(endpoint: nw_endpoint_t) -> *const sockaddr;

    pub fn nw_endpoint_create_bonjour_service(
        name: *const c_char,
        service_type: *const c_char,
        domain: *const c_char,
    ) -> nw_endpoint_t;
    pub fn nw_endpoint_get_bonjour_service_name(endpoint: nw_endpoint_t) -> *const c_char;
    pub fn nw_endpoint_get_bonjour_service_type(endpoint: nw_endpoint_t) -> *const c_char;
    pub fn nw_endpoint_get_bonjour_service_domain(endpoint: nw_endpoint_t) -> *const c_char;

    pub fn nw_endpoint_create_url(url: *const c_char) -> nw_endpoint_t;
    pub fn nw_endpoint_get_url(endpoint: nw_endpoint_t) -> *const c_char;
}
