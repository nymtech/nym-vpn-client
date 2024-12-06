// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::CStr;

use nix::sys::socket::{SockaddrLike, SockaddrStorage};
use objc2::rc::Retained;

use super::sys;
pub use sys::nw_endpoint_type_t;

/// A local or remote endpoint in a network connection.
#[derive(Debug)]
pub struct Endpoint {
    inner: Retained<sys::OS_nw_endpoint>,
}

impl Endpoint {
    /// Create new `Endpoint` retaining the raw pointer that we don't own.
    /// Returns `None` if the pointer is null.
    pub(crate) fn retain(nw_endpoint_ref: sys::nw_endpoint_t) -> Option<Self> {
        Some(Self {
            inner: unsafe { Retained::retain(nw_endpoint_ref)? },
        })
    }

    pub fn r#type(&self) -> EndpointType {
        let raw_type = unsafe { sys::nw_endpoint_get_type(self.as_raw_mut()) };

        EndpointType::from(raw_type)
    }

    pub fn hostname(&self) -> Result<String, std::str::Utf8Error> {
        let ptr = unsafe { sys::nw_endpoint_get_hostname(self.as_raw_mut()) };
        Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_owned())
    }

    pub fn port(&self) -> u16 {
        unsafe { sys::nw_endpoint_get_port(self.as_raw_mut()) }
    }

    pub fn address(&self) -> Option<SockaddrStorage> {
        let sa_ptr = unsafe { sys::nw_endpoint_get_address(self.as_raw_mut()) };
        if sa_ptr.is_null() {
            None
        } else {
            unsafe { SockaddrStorage::from_raw(sa_ptr, None) }
        }
    }

    pub fn bonjour_service_name(&self) -> Result<String, std::str::Utf8Error> {
        let ptr = unsafe { sys::nw_endpoint_get_bonjour_service_name(self.as_raw_mut()) };
        Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_owned())
    }

    pub fn bonjour_service_type(&self) -> Result<String, std::str::Utf8Error> {
        let ptr = unsafe { sys::nw_endpoint_get_bonjour_service_type(self.as_raw_mut()) };
        Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_owned())
    }

    pub fn bonjour_service_domain(&self) -> Result<String, std::str::Utf8Error> {
        let ptr = unsafe { sys::nw_endpoint_get_bonjour_service_domain(self.as_raw_mut()) };
        Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_owned())
    }

    pub fn url(&self) -> Result<String, std::str::Utf8Error> {
        let ptr = unsafe { sys::nw_endpoint_get_url(self.as_raw_mut()) };
        Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_owned())
    }

    fn as_raw_mut(&self) -> sys::nw_endpoint_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EndpointType {
    Invalid,
    Address,
    Host,
    BonjourService,
    Url,
    Unknown(nw_endpoint_type_t),
}

impl From<nw_endpoint_type_t> for EndpointType {
    fn from(value: nw_endpoint_type_t) -> Self {
        match value {
            sys::nw_endpoint_type_invalid => Self::Invalid,
            sys::nw_endpoint_type_address => Self::Address,
            sys::nw_endpoint_type_host => Self::Host,
            sys::nw_endpoint_type_bonjour_service => Self::BonjourService,
            sys::nw_endpoint_type_url => Self::Url,
            other => Self::Unknown(other),
        }
    }
}
