mod error;
pub use error::*;

#[cfg(target_os = "linux")]
pub mod linux;
