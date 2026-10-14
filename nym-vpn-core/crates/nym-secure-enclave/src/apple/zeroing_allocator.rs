// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::c_void;

use objc2_core_foundation::{CFAllocator, CFAllocatorContext, CFIndex, CFOptionFlags, CFRetained};

/// Creates a CoreFoundation allocator that wipes memory before freeing it.
pub(super) fn create_zeroing_allocator() -> Option<CFRetained<CFAllocator>> {
    let mut context = CFAllocatorContext {
        version: 0,
        info: std::ptr::null_mut(),
        retain: None,
        release: None,
        copyDescription: None,
        allocate: Some(zeroing_allocate),
        reallocate: Some(zeroing_reallocate),
        deallocate: Some(zeroing_deallocate),
        preferredSize: None,
    };
    unsafe { CFAllocator::new(None, &mut context) }
}

unsafe extern "C-unwind" fn zeroing_allocate(
    size: CFIndex,
    _hint: CFOptionFlags,
    _info: *mut c_void,
) -> *mut c_void {
    unsafe { libc::malloc(size as usize) }
}

unsafe extern "C-unwind" fn zeroing_reallocate(
    ptr: *mut c_void,
    new_size: CFIndex,
    _hint: CFOptionFlags,
    info: *mut c_void,
) -> *mut c_void {
    // libc::realloc may move the block and free the old one without wiping it.
    let new_ptr = unsafe { libc::malloc(new_size as usize) };
    if !new_ptr.is_null() {
        let old_size = unsafe { libc::malloc_size(ptr) };
        unsafe {
            std::ptr::copy_nonoverlapping(
                ptr.cast::<u8>(),
                new_ptr.cast::<u8>(),
                old_size.min(new_size as usize),
            );
            zeroing_deallocate(ptr, info);
        }
    }
    new_ptr
}

unsafe extern "C-unwind" fn zeroing_deallocate(ptr: *mut c_void, _info: *mut c_void) {
    let size = unsafe { libc::malloc_size(ptr) };
    unsafe { libc::memset_s(ptr, size, 0, size) };
    unsafe { libc::free(ptr) };
}
