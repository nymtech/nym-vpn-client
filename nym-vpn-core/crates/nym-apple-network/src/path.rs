// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cell::RefCell, ptr::NonNull, rc::Rc};

use objc2::rc::Retained;
use objc2_foundation::{NSObjectProtocol, NSString};

use super::{
    endpoint::Endpoint,
    interface::{Interface, InterfaceType},
    sys,
};
pub use sys::nw_path_status_t;

/// An object that contains information about the properties of the network that a connection uses, or that are available to your app.
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
        unsafe { Retained::cast::<NSString>((*self.inner).description()) }.to_string()
    }

    pub fn status(&self) -> PathStatus {
        PathStatus::from(unsafe { sys::nw_path_get_status(self.as_raw_mut()) })
    }

    pub fn uses_interface_type(&self, interface_type: InterfaceType) -> bool {
        unsafe { sys::nw_path_uses_interface_type(self.as_raw_mut(), interface_type.as_raw()) }
    }

    pub fn available_interfaces(&self) -> Vec<Interface> {
        let interfaces = Rc::new(RefCell::new(Vec::new()));
        let cloned_interfaces = interfaces.clone();

        // SAFETY: Use stack block since enumerator is not escaping
        let block = block2::StackBlock::new(move |nw_interface_ref| {
            let interface = Interface::retain(
                NonNull::new(nw_interface_ref)
                    .expect("nw_interface_ref is guaranteed to be non-null"),
            );

            cloned_interfaces.borrow_mut().push(interface);

            // Return yes to continue iteration
            objc2::runtime::Bool::YES
        });
        unsafe { sys::nw_path_enumerate_interfaces(self.as_raw_mut(), &block) };
        interfaces.take()
    }

    pub fn gateways(&self) -> Vec<Endpoint> {
        let gateways = Rc::new(RefCell::new(Vec::new()));
        let cloned_gateways = gateways.clone();

        // SAFETY: Use stack block since enumerator is not escaping
        let block = block2::StackBlock::new(move |nw_endpoint_ref| {
            let endpoint = Endpoint::retain(
                NonNull::new(nw_endpoint_ref)
                    .expect("nw_endpoint_ref is guaranteed to be non-null"),
            );

            cloned_gateways.borrow_mut().push(endpoint);

            // Return yes to continue iteration
            objc2::runtime::Bool::YES
        });
        unsafe { sys::nw_path_enumerate_gateways(self.as_raw_mut(), &block) };
        gateways.take()
    }

    fn as_raw_mut(&self) -> sys::nw_path_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        unsafe { sys::nw_path_is_equal(self.as_raw_mut(), other.as_raw_mut()) }
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
