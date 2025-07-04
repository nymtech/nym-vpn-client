use nym_platform_metadata::version;
use std::env::consts;
use tracing::info;

pub struct SysInfo {
    pub os_version: String,
    pub arch: String,
    pub extra: Vec<String>,
}

impl SysInfo {
    pub fn new() -> Self {
        let os_version = version();
        let arch = consts::ARCH.to_string();
        let extra_metadata = nym_platform_metadata::extra_metadata()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>();

        SysInfo {
            os_version,
            arch,
            extra: extra_metadata,
        }
    }

    pub fn display(&self, print_extra: bool) {
        info!("os version: {}", self.os_version);
        info!("os arch: {}", self.arch);
        if print_extra {
            for info in &self.extra {
                info!("os {info}");
            }
        }
    }
}
