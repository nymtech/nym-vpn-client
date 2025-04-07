// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::{Foundation::BOOL, Security::SECURITY_ATTRIBUTES};

use super::AbsoluteSecurityDescriptor;

/// Struct that contains the security identifier for an object and specifies whether the handle retrieved by specifying this struct is inheritable.
#[derive(Debug)]
pub struct SecurityAttributes {
    inner: SECURITY_ATTRIBUTES,
    _security_descriptor: AbsoluteSecurityDescriptor,
}

unsafe impl Send for SecurityAttributes {}

impl SecurityAttributes {
    /// Create new security attributes with security descriptor.
    pub fn new(security_descriptor: AbsoluteSecurityDescriptor) -> Self {
        Self {
            inner: SECURITY_ATTRIBUTES {
                bInheritHandle: BOOL::from(false),
                lpSecurityDescriptor: unsafe { security_descriptor.inner().0 as _ },
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            },
            _security_descriptor: security_descriptor,
        }
    }

    /// Returns a mutable pointer to the underlying `SECURITY_ATTRIBUTES` struct.
    ///
    /// # Safety
    /// The returned pointer is guaranteed to remain valid during the lifetime of this struct.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut SECURITY_ATTRIBUTES {
        &mut self.inner
    }
}
