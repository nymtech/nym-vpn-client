// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use windows::Win32::{
    Foundation::{CloseHandle, DuplicateHandle, DUPLICATE_SAME_ACCESS, HANDLE},
    System::Threading::{CreateEventW, GetCurrentProcess, SetEvent},
};

/// Windows event object
pub struct Event(HANDLE);

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl Event {
    /// Create a new event object using `CreateEventW`
    pub fn new(manual_reset: bool, initial_state: bool) -> windows::core::Result<Self> {
        Ok(Self(unsafe {
            CreateEventW(None, manual_reset, initial_state, None)
        }?))
    }

    /// Signal the event object
    pub fn set(&self) -> windows::core::Result<()> {
        unsafe { SetEvent(self.0) }
    }

    /// Return raw event object
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }

    /// Duplicate the event object with `DuplicateHandle()`
    pub fn duplicate(&self) -> io::Result<Event> {
        let mut new_event = HANDLE::default();
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.0,
                GetCurrentProcess(),
                &mut new_event,
                0,
                false,
                DUPLICATE_SAME_ACCESS,
            )
        }?;
        Ok(Event(new_event))
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        if let Err(e) = unsafe { CloseHandle(self.0) } {
            tracing::error!("Failed to close event handle: {}", e);
        }
    }
}
