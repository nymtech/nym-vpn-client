use tokio::sync::Mutex;

pub mod app;
pub mod updater;

pub type SharedAppState = Mutex<app::AppState>;
