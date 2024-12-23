// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Minimalistic wrapper for libdispatch.
//! Documentation: <https://developer.apple.com/documentation/dispatch?language=objc>

#![cfg(any(target_os = "macos", target_os = "ios"))]

mod rc;
mod sys;

use std::{
    ffi::{CStr, CString},
    mem::ManuallyDrop,
    ptr,
};

use rc::Retained;
pub use sys::{dispatch_queue_attr_t, dispatch_queue_t};

/// Dispatch queue.
pub struct Queue {
    /// Underlying queue handle.
    inner: ManuallyDrop<Retained<sys::OS_dispatch_queue>>,

    /// Indicates whether the queue is global.
    is_global_queue: bool,
}

unsafe impl Send for Queue {}

impl Queue {
    /// Returns global main queue.
    pub fn main() -> Self {
        Self {
            inner: unsafe {
                ManuallyDrop::new(
                    Retained::from_raw(sys::dispatch_get_main_queue().cast())
                        .expect("invalid main queue reference"),
                )
            },
            is_global_queue: true,
        }
    }

    /// Create new dispatch queue.
    pub fn new(
        queue_label: Option<&str>,
        queue_attr: QueueAttr,
    ) -> Result<Self, std::ffi::NulError> {
        let queue_label = queue_label.map(CString::new).transpose()?;
        let queue_label_ptr = queue_label
            .as_ref()
            .map(|x| x.as_ptr())
            .unwrap_or(ptr::null());

        Ok(Self {
            inner: unsafe {
                ManuallyDrop::new(
                    Retained::from_raw(sys::dispatch_queue_create(
                        queue_label_ptr,
                        queue_attr.as_raw_mut(),
                    ))
                    .expect("failed to create dispatch queue"),
                )
            },
            is_global_queue: false,
        })
    }

    /// Returns queue label.
    pub fn label(&self) -> Result<String, std::str::Utf8Error> {
        // Safety: libdispatch guarantees to never return null.
        let raw_queue_label = unsafe { sys::dispatch_queue_get_label(self.as_raw_mut()) };
        assert!(!raw_queue_label.is_null());

        let queue_label = unsafe { CStr::from_ptr(raw_queue_label) };
        queue_label.to_str().map(|x| x.to_owned())
    }

    /// Returns the underlying handle to the dispatch queue object.
    pub fn as_raw_mut(&self) -> dispatch_queue_t {
        self.inner.as_mut_ptr()
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        // Do not release global queues as they are singletons.
        if !self.is_global_queue {
            let _ = unsafe { ManuallyDrop::take(&mut self.inner) };
        }
    }
}

#[repr(transparent)]
/// Dispatch queue attribute.
pub struct QueueAttr {
    /// Underlying attribute handle.
    /// There is a special case where it can be null, which is used to indicate a serial queue.
    inner: Option<Retained<sys::OS_dispatch_queue_attr>>,
}

impl QueueAttr {
    /// Returns a serial queue attribute.
    pub fn serial() -> Self {
        Self {
            inner: unsafe { Retained::from_raw(sys::DISPATCH_QUEUE_SERIAL) },
        }
    }

    /// Returns the underlying handle to teh dispatch queue attribute.
    pub fn as_raw_mut(&self) -> dispatch_queue_attr_t {
        self.inner
            .as_ref()
            .map(|x| x.as_mut_ptr())
            .unwrap_or(std::ptr::null_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_main_queue() {
        let queue = Queue::main();
        assert_eq!(queue.label(), Ok("com.apple.main-thread".to_owned()));
    }

    #[test]
    fn test_create_serial_queue() {
        let attr = QueueAttr::serial();
        let queue =
            Queue::new(Some("net.nymtech.queue"), attr).expect("failed to create a serial queue");
        assert_eq!(queue.label(), Ok("net.nymtech.queue".to_owned()));
    }
}
