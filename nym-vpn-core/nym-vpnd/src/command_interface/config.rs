use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub(super) fn default_socket_path() -> PathBuf {
    #[cfg(unix)]
    return Path::new("/var/run/nym-vpn.sock").to_path_buf();

    #[cfg(windows)]
    return Path::new(r"\\.\pipe\nym-vpn").to_path_buf();
}

pub(super) fn default_uri_addr() -> SocketAddr {
    "[::1]:53181".parse().unwrap()
}
