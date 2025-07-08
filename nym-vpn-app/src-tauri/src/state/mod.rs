use crate::fs::app::AppFs;
use crate::fs::config::AppConfig;
use tokio::sync::Mutex;

pub mod app;

pub type SharedAppState = Mutex<app::AppState>;
pub type SharedAppConfig = Mutex<AppFs<AppConfig>>;
