// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, ops::Deref, ptr::NonNull};

use super::sys;

/// Smart pointer based on libdispatch reference counting system.
#[repr(transparent)]
pub struct Retained<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T> Retained<T> {
    /// Create new smart pointer assuming the ownership over the object.
    /// The retain count will stay the same.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    /// Create new smart pointer with shared ownership.
    /// Increments reference counter by 1.
    #[allow(unused)]
    pub unsafe fn retain(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| {
            // Safety: upheld by the caller.
            unsafe { sys::dispatch_retain(ptr.as_ptr().cast()) };
            Self { ptr }
        })
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr.as_ptr().cast()
    }
}

impl<T: ?Sized> Drop for Retained<T> {
    fn drop(&mut self) {
        // Safety: the pointer must be valid.
        unsafe { sys::dispatch_release(self.ptr.as_ptr().cast()) };
    }
}

impl<T> Clone for Retained<T> {
    /// Retain the object, increasing its reference count.
    fn clone(&self) -> Self {
        // Safety: the pointer must be valid.
        unsafe { sys::dispatch_retain(self.ptr.as_ptr().cast()) };
        Self { ptr: self.ptr }
    }
}

impl<T: ?Sized> fmt::Pointer for Retained<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr.as_ptr(), f)
    }
}

impl<T: ?Sized> Deref for Retained<T> {
    type Target = T;

    /// Obtain an immutable reference to the object.
    #[inline]
    fn deref(&self) -> &T {
        // Safety: The pointer's validity is verified when the type is created.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> fmt::Debug for Retained<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ptr.as_ptr().fmt(f)
    }
}
