use nym_platform_metadata::SysInfo;
use serde::Serialize;
use std::{env, process::Command};
use tracing::{debug, error, info, warn};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS, strum::AsRefStr)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts")]
pub enum GpuType {
    #[strum(serialize = "NVIDIA")]
    Nvidia,
    #[strum(serialize = "AMD")]
    Amd,
    Intel,
    Unknown(Option<String>),
}

#[derive(Debug, Clone, Serialize, TS, strum::AsRefStr, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts")]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown(Option<String>),
}

fn get_display_server() -> Option<DisplayServer> {
    if !cfg!(any(target_os = "linux", target_os = "openbsd")) {
        return None;
    }

    let display_server = match env::var("XDG_SESSION_TYPE")
        .inspect_err(|e| warn!("XDG_SESSION_TYPE not set or not valid: {e}"))
        .map(|s| s.to_lowercase())
    {
        Ok(s) if s == "x11" => DisplayServer::X11,
        Ok(s) if s == "wayland" => DisplayServer::Wayland,
        Ok(s) => {
            warn!("unknown display server: {}", s);
            DisplayServer::Unknown(Some(s))
        }
        _ => DisplayServer::Unknown(None),
    };
    Some(display_server)
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct OsInfo {
    /// long version
    pub version: String,
    pub kernel: String,
    pub arch: String,
    pub hash: String,
    // set on Linux only
    pub display_server: Option<DisplayServer>,
    pub gpu: Option<GpuType>,
}

impl OsInfo {
    pub fn new() -> Self {
        let system = SysInfo::new();
        let hash = system.hash_identifier();
        let kernel = sysinfo::System::kernel_version().unwrap_or_else(|| "unknown".to_string());
        Self {
            version: system.os_version,
            kernel,
            arch: system.arch,
            display_server: get_display_server(),
            gpu: gpu_info(),
            hash,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    pub fn linux_check(&self) {
        // with NVIDIA gpu, there is an upstream issue with webkit dmabuf renderer
        // see https://github.com/tauri-apps/tauri/issues/9304
        if matches!(self.gpu, Some(GpuType::Nvidia)) {
            info!("NVIDIA gpu detected, disabling webkit dmabuf renderer");
            unsafe {
                env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "openbsd"))]
    pub fn print_linux_info(&self) {
        if let Some(ds) = &self.display_server {
            info!("display server: {}", ds.as_ref());
        }
        if let Some(gpu) = &self.gpu {
            info!("gpu: {}", gpu.as_ref());
        }
    }
}

/// Linux only
fn gpu_info() -> Option<GpuType> {
    if !cfg!(any(target_os = "linux", target_os = "openbsd")) {
        return None;
    }

    let Ok(output) = Command::new("lspci").arg("-nn").output().inspect_err(|e| {
        error!("failed to run lspci: {}", e);
    }) else {
        return Some(GpuType::Unknown(None));
    };
    if !output.status.success() {
        error!("lspci failed: {}", String::from_utf8_lossy(&output.stderr));
        return Some(GpuType::Unknown(None));
    }
    let output = String::from_utf8_lossy(&output.stdout);
    let Some(info) = output
        .lines()
        .find(|line| line.to_lowercase().contains("vga compatible controller"))
    else {
        warn!("no VGA device found in lspci output");
        return Some(GpuType::Unknown(None));
    };
    debug!("GPU info: {}", info);
    if info.to_lowercase().contains("nvidia") {
        return Some(GpuType::Nvidia);
    } else if info.to_lowercase().contains("amd") || info.contains("radeon") {
        return Some(GpuType::Amd);
    } else if info.to_lowercase().contains("intel") {
        return Some(GpuType::Intel);
    }
    info!("unknown GPU type: {}", info);
    Some(GpuType::Unknown(Some(info.to_string())))
}

impl std::fmt::Display for OsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.version, self.kernel, self.arch)
    }
}

impl Default for GpuType {
    fn default() -> Self {
        GpuType::Unknown(None)
    }
}
