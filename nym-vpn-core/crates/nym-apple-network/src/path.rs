// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cell::RefCell, ptr::NonNull, rc::Rc};

use objc2::runtime::NSObjectProtocol;
use objc2_foundation::NSString;

use super::{
    endpoint::Endpoint,
    interface::{Interface, InterfaceType},
    rc::Retained,
    sys,
};
pub use sys::nw_path_status_t;

/// An object that contains information about the properties of the network that a connection uses, or that are available to your app.
#[repr(transparent)]
#[derive(Debug)]
pub struct Path {
    inner: Retained<sys::OS_nw_path>,
}

unsafe impl Send for Path {}

impl Path {
    /// Create new `Path` retaining the raw pointer that we don't own.
    pub(crate) fn retain(nw_path_ref: NonNull<sys::OS_nw_path>) -> Self {
        Self {
            inner: unsafe { Retained::retain(nw_path_ref.as_ptr()) }
                .expect("failed to retain nw_path_ref"),
        }
    }

    pub fn description(&self) -> String {
        unsafe { objc2::rc::Retained::cast::<NSString>((*self.inner).description()) }.to_string()
    }

    pub fn status(&self) -> PathStatus {
        PathStatus::from(unsafe { sys::nw_path_get_status(self.inner.as_mut_ptr()) })
    }

    pub fn uses_interface_type(&self, interface_type: InterfaceType) -> bool {
        unsafe {
            sys::nw_path_uses_interface_type(self.inner.as_mut_ptr(), interface_type.as_raw())
        }
    }

    pub fn available_interfaces(&self) -> Vec<Interface> {
        let interfaces = Rc::new(RefCell::new(Vec::new()));
        let cloned_interfaces = interfaces.clone();

        // Safety: Use stack block since enumerator is not escaping
        let block = block2::StackBlock::new(move |nw_interface_ref| {
            let interface = Interface::retain(
                NonNull::new(nw_interface_ref)
                    .expect("nw_interface_ref is guaranteed to be non-null"),
            );

            cloned_interfaces.borrow_mut().push(interface);

            // Return yes to continue iteration
            objc2::runtime::Bool::YES
        });
        unsafe { sys::nw_path_enumerate_interfaces(self.inner.as_mut_ptr(), &block) };
        interfaces.take()
    }

    pub fn gateways(&self) -> Vec<Endpoint> {
        let gateways = Rc::new(RefCell::new(Vec::new()));
        let cloned_gateways = gateways.clone();

        // Safety: Use stack block since enumerator is not escaping
        let block = block2::StackBlock::new(move |nw_endpoint_ref| {
            let endpoint = Endpoint::retain(
                NonNull::new(nw_endpoint_ref)
                    .expect("nw_endpoint_ref is guaranteed to be non-null"),
            );

            cloned_gateways.borrow_mut().push(endpoint);

            // Return yes to continue iteration
            objc2::runtime::Bool::YES
        });
        unsafe { sys::nw_path_enumerate_gateways(self.inner.as_mut_ptr(), &block) };
        gateways.take()
    }

    /// Checks whether the path can route IPv4 traffic.
    pub fn supports_ipv4(&self) -> bool {
        unsafe { sys::nw_path_has_ipv4(self.inner.as_mut_ptr()) }
    }

    /// Checks whether the path can route IPv6 traffic.
    pub fn supports_ipv6(&self) -> bool {
        unsafe { sys::nw_path_has_ipv6(self.inner.as_mut_ptr()) }
    }

    /// Checks whether the path has a DNS server configured.
    pub fn supports_dns(&self) -> bool {
        unsafe { sys::nw_path_has_dns(self.inner.as_mut_ptr()) }
    }

    /// Checks whether the path uses an interface in Low Data Mode.
    pub fn is_constrained(&self) -> bool {
        unsafe { sys::nw_path_is_constrained(self.inner.as_mut_ptr()) }
    }

    /// Checks whether the path uses an interface that is considered expensive, such as Cellular or a Personal Hotspot.
    pub fn is_expensive(&self) -> bool {
        unsafe { sys::nw_path_is_expensive(self.inner.as_mut_ptr()) }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        unsafe { sys::nw_path_is_equal(self.inner.as_mut_ptr(), other.inner.as_mut_ptr()) }
    }
}

/// Status values indicating whether a path can be used by connections.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PathStatus {
    /// The path cannot be evaluated.
    Invalid,

    /// The path is not available for use.
    Unsatisfied,

    /// The path is available to establish connections and send data.
    Satisfied,

    /// The path is not currently available, but establishing a new connection may activate the path.
    Satisfiable,

    /// The path unknown to the crate.
    Unknown(nw_path_status_t),
}

impl From<sys::nw_path_status_t> for PathStatus {
    fn from(value: sys::nw_path_status_t) -> Self {
        match value {
            sys::nw_path_status_invalid => Self::Invalid,
            sys::nw_path_status_satisfied => Self::Satisfied,
            sys::nw_path_status_unsatisfied => Self::Unsatisfied,
            sys::nw_path_status_satisfiable => Self::Satisfiable,
            other => Self::Unknown(other),
        }
    }
}
