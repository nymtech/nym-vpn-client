use std::sync::Arc;
use tokio::sync::Mutex;

pub mod app;

pub type SharedAppState = Arc<Mutex<app::AppState>>;
