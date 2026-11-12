// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

/// Stops the kernel from writing core dumps of the daemon, which would hold the
/// WireGuard private key and account tokens in plain text. Unlike `RLIMIT_CORE`,
/// this also covers piped `core_pattern` handlers such as systemd-coredump and apport.
#[cfg(target_os = "linux")]
pub(crate) fn disable() -> nix::Result<()> {
    nix::sys::prctl::set_dumpable(false)
}

/// Stops the kernel from writing core dumps of the daemon, which would hold the
/// WireGuard private key and account tokens in plain text.
#[cfg(target_os = "macos")]
pub(crate) fn disable() -> nix::Result<()> {
    nix::sys::resource::setrlimit(nix::sys::resource::Resource::RLIMIT_CORE, 0, 0)
}

#[cfg(windows)]
const EXE_NAME: windows::core::PCWSTR = windows::core::w!("nym-vpnd.exe");

/// Excludes the daemon from Windows Error Reporting for all users, so that its crashes
/// don't produce dumps holding the WireGuard private key and account tokens in plain text.
#[cfg(windows)]
pub(crate) fn disable() -> windows::core::Result<()> {
    unsafe { windows::Win32::System::ErrorReporting::WerAddExcludedApplication(EXE_NAME, true) }
}

#[cfg(windows)]
pub(crate) fn restore() -> windows::core::Result<()> {
    unsafe { windows::Win32::System::ErrorReporting::WerRemoveExcludedApplication(EXE_NAME, true) }
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
