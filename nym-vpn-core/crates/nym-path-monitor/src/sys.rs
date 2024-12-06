#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, c_int, c_void};

pub type dispatch_queue_t = *mut objc2_foundation::NSObject;

pub type nw_path_monitor_t = *mut objc2_foundation::NSObject;
pub type nw_path_t = *mut objc2_foundation::NSObject;
pub type nw_path_monitor_update_handler_t = block2::Block<dyn Fn(nw_path_t)>;
pub type nw_path_status_t = c_int;
pub type nw_interface_t = *mut objc2_foundation::NSObject;
pub type nw_path_enumerate_interfaces_block_t =
    block2::Block<dyn Fn(nw_interface_t) -> objc2::runtime::Bool>;
pub type nw_interface_type_t = c_int;

pub const nw_path_status_invalid: c_int = 0;
pub const nw_path_status_satisfied: c_int = 1;
pub const nw_path_status_unsatisfied: c_int = 2;
pub const nw_path_status_satisfiable: c_int = 3;

pub const nw_interface_type_other: c_int = 0;
pub const nw_interface_type_wifi: c_int = 1;
pub const nw_interface_type_cellular: c_int = 2;
pub const nw_interface_type_wired: c_int = 3;
pub const nw_interface_type_loopback: c_int = 4;

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub fn nw_path_monitor_create() -> nw_path_monitor_t;

    pub fn nw_path_monitor_set_queue(monitor: nw_path_monitor_t, dispatch_queue: dispatch_queue_t);
    pub fn nw_path_monitor_set_update_handler(
        monitor: nw_path_monitor_t,
        update_handler: &nw_path_monitor_update_handler_t,
    );

    pub fn nw_path_monitor_start(monitor: nw_path_monitor_t);
    pub fn nw_path_monitor_cancel(monitor: nw_path_monitor_t);

    pub fn nw_path_get_status(path: nw_path_t) -> nw_path_status_t;
    pub fn nw_path_is_equal(path: nw_path_t, other_path: nw_path_t) -> objc2::ffi::BOOL;
    pub fn nw_path_enumerate_interfaces(
        path: nw_path_t,
        enumerate_block: &nw_path_enumerate_interfaces_block_t,
    );
    pub fn nw_interface_get_type(interface: nw_interface_t) -> nw_interface_type_t;
    pub fn nw_interface_get_name(interface: nw_interface_t) -> *mut c_char;
    pub fn nw_interface_get_index(interface: nw_interface_t) -> u32;

    pub fn nw_retain(objc: *mut c_void);
    pub fn nw_release(objc: *mut c_void);
}
