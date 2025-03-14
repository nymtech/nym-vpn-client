use std::path::PathBuf;

pub(super) fn default_socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/var/run/nym-vpn.sock")
    }

    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\nym-vpn")
    }
}
