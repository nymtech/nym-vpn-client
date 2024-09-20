use std::{io, net::Ipv6Addr};

pub fn set_tun_ipv6_addr(_device_name: &str, _ipv6_addr: Ipv6Addr) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("ip")
        .args([
            "-6",
            "addr",
            "add",
            &_ipv6_addr.to_string(),
            "dev",
            _device_name,
        ])
        .output()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("ifconfig")
        .args([_device_name, "inet6", "add", &_ipv6_addr.to_string()])
        .output()?;

    Ok(())
}
