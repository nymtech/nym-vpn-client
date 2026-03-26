use freedesktop_desktop_entry::DesktopEntry;
use freedesktop_icons::lookup;
use serde::Serialize;
use std::path::PathBuf;
use tracing::{debug, info};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct App {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub desktop_file: String,
}

/// Scan XDG desktop entry directories and return visible GUI applications.
pub fn get_desktop_apps() -> Vec<App> {
    let locales = &["en".to_string()];
    let entries = freedesktop_desktop_entry::desktop_entries(locales);

    let mut apps: Vec<App> = entries
        .into_iter()
        .filter(|entry: &DesktopEntry| {
            entry.type_() == Some("Application")
                && !entry.no_display()
                && !entry.hidden()
                && entry.exec().is_some()
        })
        .filter_map(|entry: DesktopEntry| {
            let name = entry.name(locales)?.into_owned();
            let exec = entry.exec()?.to_string();

            let icon = entry.icon().and_then(|icon_name| resolve_icon(icon_name));

            Some(App {
                id: entry.appid.clone(),
                name,
                exec,
                icon,
                desktop_file: entry.path.to_string_lossy().into_owned(),
            })
        })
        .collect();

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.id == b.id);
    debug!("found {} desktop apps", apps.len());
    apps
}

/// Resolve an icon name to a canonical file path string.
/// If the name is already an absolute path, use it when it exists.
fn resolve_icon(icon_name: &str) -> Option<String> {
    info!("resolving icon: {}", icon_name);
    let path = if icon_name.starts_with('/') {
        let path = PathBuf::from(icon_name);
        path.exists().then_some(path)?
    } else {
        lookup(icon_name).with_size(48).with_cache().find()?
    };
    std::fs::canonicalize(&path)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}
