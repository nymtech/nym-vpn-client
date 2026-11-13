// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;

/// Stops the kernel from writing core dumps of the daemon, which would hold the
/// WireGuard private key and account tokens in plain text. Unlike `RLIMIT_CORE`,
/// this also covers piped `core_pattern` handlers such as systemd-coredump and apport.
#[cfg(target_os = "linux")]
pub(crate) fn disable() -> io::Result<()> {
    Ok(nix::sys::prctl::set_dumpable(false)?)
}

/// Stops the kernel from writing core dumps of the daemon, which would hold the
/// WireGuard private key and account tokens in plain text.
#[cfg(target_os = "macos")]
pub(crate) fn disable() -> io::Result<()> {
    Ok(nix::sys::resource::setrlimit(
        nix::sys::resource::Resource::RLIMIT_CORE,
        0,
        0,
    )?)
}

/// Excludes the daemon from Windows Error Reporting for all users, so that its crashes
/// don't produce dumps holding the WireGuard private key and account tokens in plain text.
///
/// The exclusion is stored in the registry under the executable's file name, so it persists
/// across restarts until [`restore`] removes it. Requires administrator rights.
#[cfg(windows)]
pub(crate) fn disable() -> io::Result<()> {
    let exe_name = exe_name()?;
    unsafe { windows::Win32::System::ErrorReporting::WerAddExcludedApplication(&exe_name, true) }?;
    Ok(())
}

/// Removes the persistent Windows Error Reporting exclusion added by [`disable`].
/// Requires administrator rights.
#[cfg(windows)]
pub(crate) fn restore() -> io::Result<()> {
    let exe_name = exe_name()?;
    unsafe {
        windows::Win32::System::ErrorReporting::WerRemoveExcludedApplication(&exe_name, true)
    }?;
    Ok(())
}

#[cfg(windows)]
fn exe_name() -> io::Result<windows::core::HSTRING> {
    let exe_path = std::env::current_exe()?;
    let exe_name = exe_path
        .file_name()
        .ok_or_else(|| io::Error::other("Current executable path has no file name"))?;
    Ok(windows::core::HSTRING::from(exe_name))
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn disable_marks_process_not_dumpable() {
        super::disable().unwrap();
        assert!(!nix::sys::prctl::get_dumpable().unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn disable_sets_core_limit_to_zero() {
        super::disable().unwrap();
        assert_eq!(
            nix::sys::resource::getrlimit(nix::sys::resource::Resource::RLIMIT_CORE).unwrap(),
            (0, 0)
        );
    }
}
