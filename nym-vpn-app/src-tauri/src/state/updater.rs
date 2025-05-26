use tauri_plugin_updater::Update;
use tokio::sync::Mutex;

pub struct PendingUpdate(pub Mutex<Option<Update>>);
