use crate::fs::app::AppFs;
use crate::fs::config::AppConfig;
use crate::log::DebugLogging;
use tokio::sync::Mutex;

pub mod app;

pub type SharedAppState = Mutex<app::AppState>;
pub type SharedAppConfig = Mutex<AppFs<AppConfig>>;
pub type SharedDebugLogging = Mutex<DebugLogging>;
