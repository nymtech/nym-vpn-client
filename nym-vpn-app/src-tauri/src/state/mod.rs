use tokio::sync::Mutex;

pub mod app;

pub type SharedAppState = Mutex<app::AppState>;
