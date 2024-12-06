// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cell::RefCell, rc::Rc};

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

impl Path {
    /// Create new `Path` retaining the raw pointer that we don't own.
    /// Returns `None` if the pointer is null.
    pub(crate) fn retain(nw_path_ref: sys::nw_path_t) -> Option<Self> {
        Some(Self {
            inner: unsafe { Retained::retain(nw_path_ref)? },
        })
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

        // SAFETY: Use stack block since interface enumerator is not escaping (NW_NOESCAPE)
        let block = block2::StackBlock::new(move |nw_interface_ref| {
            // SAFETY: nw_interface_ref must never be null.
            let interface = Interface::retain(nw_interface_ref).expect("invalid nw_interface_ref");

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

        // SAFETY: Use stack block since interface enumerator is not escaping (NW_NOESCAPE)
        let block = block2::StackBlock::new(move |nw_endpoint_ref| {
            // SAFETY: nw_endpoint_ref must never be null.
            let endpoint = Endpoint::retain(nw_endpoint_ref).expect("invalid nw_endpoint_ref");

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PathStatus {
    Invalid,
    Unsatisfied,
    Satisfied,
    Satisfiable,
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
