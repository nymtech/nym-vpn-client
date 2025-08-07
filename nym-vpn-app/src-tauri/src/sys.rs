use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
use std::process::Command;
use sysinfo::System;
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
use tracing::{error, info, warn};
use ts_rs::TS;

#[cfg(any(target_os = "linux", target_os = "openbsd"))]
#[derive(Debug, Clone, Default, Serialize, TS, strum::AsRefStr)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum GpuType {
    #[strum(serialize = "NVIDIA")]
    Nvidia,
    #[strum(serialize = "AMD")]
    Amd,
    Intel,
    #[default]
    Unknown,
}

#[cfg(any(target_os = "linux", target_os = "openbsd"))]
#[derive(Debug, Clone, Default, Serialize, TS, strum::AsRefStr)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum DisplayServer {
    X11,
    Wayland,
    #[default]
    Unknown,
}

#[cfg(any(target_os = "linux", target_os = "openbsd"))]
fn get_display_server() -> DisplayServer {
    match env::var("XDG_SESSION_TYPE")
        .inspect_err(|e| warn!("XDG_SESSION_TYPE not set or not valid: {e}"))
        .map(|s| s.to_lowercase())
    {
        Ok(s) if s == "x11" => DisplayServer::X11,
        Ok(s) if s == "wayland" => DisplayServer::Wayland,
        Ok(s) => {
            warn!("unknown display server: {}", s);
            DisplayServer::Unknown
        }
        _ => DisplayServer::Unknown,
    }
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OsInfo {
    /// long version
    pub version: String,
    pub kernel: Option<String>,
    pub arch: String,
    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    pub display_server: DisplayServer,
    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    pub gpu: GpuType,
}

impl OsInfo {
    pub fn new() -> Self {
        Self {
            version: System::long_os_version().unwrap_or_else(|| env::consts::OS.into()),
            kernel: System::kernel_version(),
            arch: env::consts::ARCH.to_string(),
            #[cfg(any(target_os = "linux", target_os = "openbsd"))]
            display_server: get_display_server(),
            #[cfg(any(target_os = "linux", target_os = "openbsd"))]
            gpu: gpu_info(),
        }
    }

    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    pub fn linux_check(&self) {
        // with NVIDIA gpu, there is an upstream issue with webkit dmabuf renderer
        // see https://github.com/tauri-apps/tauri/issues/9304
        if matches!(self.gpu, GpuType::Nvidia) {
            info!("NVIDIA gpu detected, disabling webkit dmabuf renderer");
            unsafe {
                env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
        }
    }

    pub fn stringify_identifier(&self) -> String {
        let parts = [
            self.version.clone(),
            self.kernel.clone().unwrap_or_default(),
            self.arch.clone(),
            sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string()),
        ];

        parts.join(" ")
    }

    pub fn hash_identifier(&self) -> String {
        let os_name = self.stringify_identifier();
        let hash = Sha256::digest(os_name.as_bytes());
        format!("{hash:x}")
    }
}

#[cfg(any(target_os = "linux", target_os = "openbsd"))]
fn gpu_info() -> GpuType {
    use tracing::debug;

    let Ok(output) = Command::new("lspci").arg("-nn").output().inspect_err(|e| {
        error!("failed to run lspci: {}", e);
    }) else {
        return GpuType::Unknown;
    };
    if !output.status.success() {
        error!("lspci failed: {}", String::from_utf8_lossy(&output.stderr));
        return GpuType::Unknown;
    }
    let output = String::from_utf8_lossy(&output.stdout);
    let Some(info) = output
        .lines()
        .find(|line| line.to_lowercase().contains("vga compatible controller"))
    else {
        warn!("no VGA device found in lspci output");
        return GpuType::Unknown;
    };
    debug!("GPU info: {}", info);
    if info.to_lowercase().contains("nvidia") {
        return GpuType::Nvidia;
    } else if info.to_lowercase().contains("amd") || info.contains("radeon") {
        return GpuType::Amd;
    } else if info.to_lowercase().contains("intel") {
        return GpuType::Intel;
    }
    warn!("unknown GPU type: {}", info);
    GpuType::Unknown
}

impl std::fmt::Display for OsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.version,
            self.kernel.as_deref().unwrap_or("unknown"),
            self.arch
        )
    }
}
