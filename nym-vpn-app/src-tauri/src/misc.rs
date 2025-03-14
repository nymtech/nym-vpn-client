#[cfg(any(target_os = "linux", target_os = "openbsd"))]
use tracing::{info, warn};

#[cfg(any(target_os = "linux", target_os = "openbsd"))]
pub fn linux_check() {
    if let Some(display_server) = std::env::var("XDG_SESSION_TYPE")
        .inspect_err(|e| warn!("XDG_SESSION_TYPE not set or not valid: {e}"))
        .ok()
        .map(|s| s.to_lowercase())
    {
        info!("display server: {}", display_server);

        // under X11 with nvidia gpu, there is an upstream issue with webkit dmabuf renderer
        // see https://github.com/tauri-apps/tauri/issues/9304
        if display_server == "x11"
            && std::fs::exists("/dev/nvidia0")
                .inspect_err(|e| warn!("unable to check for nvidia gpu {}", e))
                .unwrap_or(false)
        {
            info!("nvidia gpu detected, disabling webkit dmabuf renderer");
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
        }
    }
}
