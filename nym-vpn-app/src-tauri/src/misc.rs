#[cfg(unix)]
use tracing::{info, warn};

// under X11 with nvidia gpu, there is an upstream issue with
// webkit dmabuf renderer
// see https://github.com/tauri-apps/tauri/issues/9304
#[cfg(unix)]
pub fn nvidia_check() {
    if std::fs::exists("/dev/nvidia0")
        .inspect_err(|e| warn!("unable to check for nvidia gpu {}", e))
        .unwrap_or(false)
        && std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .to_lowercase()
            == "x11"
    {
        info!("X11 and nvidia gpu detected, disabling webkit dmabuf renderer");
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
}
