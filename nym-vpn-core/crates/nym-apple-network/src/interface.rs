// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CStr;

use objc2::rc::Retained;

use super::sys;
pub use sys::nw_interface_type_t;

/// An interface that a network connection uses to send and receive data.
#[derive(Debug)]
pub struct Interface {
    inner: Retained<sys::OS_nw_interface>,
}

impl Interface {
    /// Create new `Interface` retaining the raw pointer that we don't own.
    /// Returns `None` if the pointer is null.
    pub(crate) fn retain(nw_interface_ref: sys::nw_interface_t) -> Option<Self> {
        Some(Self {
            inner: unsafe { Retained::retain(nw_interface_ref)? },
        })
    }

    pub fn name(&self) -> Result<String, std::str::Utf8Error> {
        unsafe {
            let ptr = sys::nw_interface_get_name(self.as_raw_mut());
            assert!(!ptr.is_null());
            Ok(CStr::from_ptr(ptr).to_str()?.to_owned())
        }
    }

    pub fn index(&self) -> u32 {
        unsafe { sys::nw_interface_get_index(self.as_raw_mut()) }
    }

    pub fn interface_type(&self) -> InterfaceType {
        let raw_interface_type = unsafe { sys::nw_interface_get_type(self.as_raw_mut()) };
        InterfaceType::from(raw_interface_type)
    }

    fn as_raw_mut(&self) -> sys::nw_interface_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

/// Types of network interfaces, based on their link layer media types.
#[derive(Debug, Copy, Clone)]
pub enum InterfaceType {
    Other,
    Wifi,
    Cellular,
    Wired,
    Loopback,
    Unknown(nw_interface_type_t),
}

impl From<sys::nw_interface_type_t> for InterfaceType {
    fn from(value: sys::nw_interface_type_t) -> Self {
        match value {
            sys::nw_interface_type_other => Self::Other,
            sys::nw_interface_type_wifi => Self::Wifi,
            sys::nw_interface_type_cellular => Self::Cellular,
            sys::nw_interface_type_wired => Self::Wired,
            sys::nw_interface_type_loopback => Self::Loopback,
            other => Self::Unknown(other),
        }
    }
}

impl InterfaceType {
    pub(crate) fn as_raw(&self) -> sys::nw_interface_type_t {
        match self {
            InterfaceType::Other => sys::nw_interface_type_other,
            InterfaceType::Wifi => sys::nw_interface_type_wifi,
            InterfaceType::Cellular => sys::nw_interface_type_cellular,
            InterfaceType::Wired => sys::nw_interface_type_wired,
            InterfaceType::Loopback => sys::nw_interface_type_loopback,
            InterfaceType::Unknown(other) => *other,
        }
    }
}
