// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Used to monitor volume mounts and dismounts, and reapply the split
//! tunnel config if any of the excluded paths are affected by them.
use super::path_monitor::PathMonitorHandle;
use nym_common::ErrorExt;
use nym_windows::window::{WindowCloseHandle, create_hidden_window};
use std::{
    ffi::OsString,
    io,
    path::{self, Path},
    sync::{Arc, Mutex, mpsc as sync_mpsc},
};

use windows_sys::Win32::{
    Storage::FileSystem::GetLogicalDrives,
    UI::WindowsAndMessaging::{
        DBT_DEVICEARRIVAL, DBT_DEVICEREMOVECOMPLETE, DBT_DEVTYP_VOLUME, DBTF_NET,
        DEV_BROADCAST_HDR, DEV_BROADCAST_VOLUME, DefWindowProcW, WM_DEVICECHANGE,
    },
};

pub(super) struct VolumeMonitor(());

pub(super) struct VolumeMonitorHandle {
    window_handle: WindowCloseHandle,
}

impl Drop for VolumeMonitorHandle {
    fn drop(&mut self) {
        self.window_handle.close();
    }
}

impl VolumeMonitor {
    pub fn spawn(
        path_monitor: PathMonitorHandle,
        update_tx: sync_mpsc::Sender<()>,
        paths: Arc<Mutex<(Vec<OsString>, Vec<OsString>)>>,
    ) -> VolumeMonitorHandle {
        // A bitmask containing all (known) mounted drives.
        let known_state = Arc::new(Mutex::new(0u32));

        // Lock before registering event handler
        let mut known_state_guard = known_state.lock().unwrap();

        let window_handle =
            start_internal_monitor(known_state.clone(), path_monitor, update_tx, paths);

        *known_state_guard = get_logical_drives().unwrap_or_else(|error| {
            tracing::error!(
                "{}",
                error.display_chain_with_msg("Failed to initialize state of mounted volumes")
            );
            0
        });

        VolumeMonitorHandle { window_handle }
    }
}

/// Monitors window events received by session 0.
fn start_internal_monitor(
    known_state: Arc<Mutex<u32>>,
    path_monitor: PathMonitorHandle,
    update_tx: sync_mpsc::Sender<()>,
    paths: Arc<Mutex<(Vec<OsString>, Vec<OsString>)>>,
) -> WindowCloseHandle {
    use windows::Win32::Foundation::LRESULT;

    create_hidden_window(move |window, message, w_param, l_param| {
        if !is_device_arrival_or_removal(message, w_param.0) {
            return LRESULT(unsafe { DefWindowProcW(window.0, message, w_param.0, l_param.0) });
        }
        let paths_guard = paths.lock().unwrap();
        let (paths, hybrid_paths) = &*paths_guard;
        let mut known_state_guard = known_state.lock().unwrap();

        let volumes = unsafe { parse_device_volume_broadcast(&*(l_param.0 as *const _)) };

        let prev_state = *known_state_guard;
        let is_arrival = w_param.0 == DBT_DEVICEARRIVAL as usize;
        if is_arrival {
            *known_state_guard |= volumes;
        } else {
            *known_state_guard &= !volumes;
        }

        // Compare against known state to ignore duplicate notifications
        // from frontends
        let state_diff = *known_state_guard ^ prev_state;
        if state_diff != 0 && matches_volume(volumes, paths, hybrid_paths) {
            // Reapply config
            let _ = update_tx.send(());
            let _ = path_monitor.refresh();
        }

        // Always grant the request
        LRESULT(1)
    })
}

/// Return a bitmask representing all currently available disk drives.
/// Each bit refers to a volume letter. The bit 0 refers to 'A', bit 1
/// refers to 'B', bit 2 to 'C', etc.
fn get_logical_drives() -> io::Result<u32> {
    let result = unsafe { GetLogicalDrives() };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(result)
}

/// Return whether any of the paths in `paths` or `hybrid_paths` reside on any volume in `volumes` (a mask).
fn matches_volume(volumes: u32, paths: &[OsString], hybrid_paths: &[OsString]) -> bool {
    for path in paths.iter().chain(hybrid_paths.iter()) {
        let path: &Path = path.as_ref();
        if let Some(path::Component::Prefix(prefix)) = path.components().next() {
            match prefix.kind() {
                path::Prefix::VerbatimDisk(disk) | path::Prefix::Disk(disk) => {
                    if !disk.is_ascii_uppercase() {
                        tracing::warn!("Ignoring invalid volume \"{}\"", disk as char);
                        continue;
                    }
                    let disk = disk - b'A';
                    if volumes & (1 << disk) != 0 {
                        return true;
                    }
                }
                _ => (),
            }
        }
    }
    false
}

fn is_device_arrival_or_removal(message: u32, w_param: usize) -> bool {
    message == WM_DEVICECHANGE
        && (w_param == DBT_DEVICEARRIVAL as usize || w_param == DBT_DEVICEREMOVECOMPLETE as usize)
}

/// Return volumes affected by the device arrival or removal message as a mask.
/// This has the same format as `get_logical_drives()`.
unsafe fn parse_device_volume_broadcast(broadcast: &DEV_BROADCAST_HDR) -> u32 {
    if broadcast.dbch_devicetype != DBT_DEVTYP_VOLUME {
        return 0;
    }

    let volume_broadcast = unsafe { &*(broadcast as *const _ as *const DEV_BROADCAST_VOLUME) };
    if volume_broadcast.dbcv_flags & DBTF_NET != 0 {
        // Ignore net event
        return 0;
    }

    volume_broadcast.dbcv_unitmask
}
