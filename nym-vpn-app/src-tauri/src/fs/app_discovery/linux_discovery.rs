use super::App;
use super::utils::is_excluded;
use crate::error::BackendError;
use freedesktop_desktop_entry::{DesktopEntry, desktop_entries, get_languages_from_env};
use freedesktop_icons::lookup;
use regex::Regex;
use std::path::PathBuf;
use tracing::{debug, info};

pub fn get_linux_apps() -> Result<Vec<App>, BackendError> {
    let locales = get_languages_from_env();
    let entries = desktop_entries(&locales);

    let mut apps: Vec<App> = entries
        .into_iter()
        .filter_map(|entry: DesktopEntry| {
            if entry.type_() != Some("Application") || entry.no_display() || entry.hidden() {
                return None;
            }
            let name = entry.name(&locales)?.into_owned();

            if is_excluded(&name) {
                return None;
            }

            let executable_path = cleanup_entry(&entry)?;

            let icon = entry.icon().and_then(resolve_icon);
            Some(App {
                name,
                executable_path,
                icon,
                is_custom: false,
            })
        })
        .collect();

    apps.sort_by_key(|k| k.name.to_lowercase());
    apps.dedup_by(|a, b| a.name == b.name);
    debug!("discovered {} Linux apps", apps.len());

    Ok(apps)
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

/// Cleanup the executable path for Linux
/// Remove desktop entry placeholder like %U, %u, ...
/// Remove flatpak unique placeholders
fn cleanup_entry(entry: &DesktopEntry) -> Option<String> {
    let exec = entry.exec()?.to_string();
    info!("exec: {}", exec);

    let exec_cleaned = if entry.flatpak().is_some() {
        let re = Regex::new(r"@@.*?@@").unwrap();
        re.replace_all(&exec, "")
    } else {
        let re = Regex::new(r"%[cdDfFikmnNuUv]").unwrap();
        re.replace_all(&exec, "")
    };

    Some(exec_cleaned.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("nymvpn-linux-disc-{}-{}", std::process::id(), tag));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_icon_returns_none_for_missing_absolute_path() {
        assert!(resolve_icon("/tmp/nymvpn-test-definitely-missing-icon-xyz123.png").is_none());
    }

    #[test]
    fn resolve_icon_resolves_existing_absolute_path_to_canonical_form() {
        let dir = scratch_dir("icon");
        let file = dir.join("app-icon.png");
        fs::write(&file, b"fake png data").unwrap();

        let canonical = fs::canonicalize(&file)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(resolve_icon(&file.to_string_lossy()), Some(canonical));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_icon_returns_none_for_unknown_theme_icon_name() {
        // A bare name (no leading '/') that is absent from every installed icon theme.
        assert!(resolve_icon("nymvpn-definitely-nonexistent-icon-xyzzy").is_none());
    }
}
