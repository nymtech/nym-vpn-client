mod path_monitor;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{spawn_monitor, MonitorHandle};

#[cfg(target_os = "ios")]
pub use path_monitor::{spawn_monitor, MonitorHandle};
