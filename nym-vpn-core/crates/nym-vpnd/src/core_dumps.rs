// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

/// Stops the kernel from writing core dumps of the daemon, which would hold the
/// WireGuard private key and account tokens in plain text. Unlike `RLIMIT_CORE`,
/// this also covers piped `core_pattern` handlers such as systemd-coredump and apport.
pub(crate) fn disable() -> nix::Result<()> {
    nix::sys::prctl::set_dumpable(false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn disable_marks_process_not_dumpable() {
        super::disable().unwrap();
        assert!(!nix::sys::prctl::get_dumpable().unwrap());
    }
}
