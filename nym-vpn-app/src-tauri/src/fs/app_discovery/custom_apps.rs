//! Storage and logic for user-added ("custom") split-tunnel apps.
//!
//! On Linux there is no daemon-side exclude list, so apps the user picks via
//! the file dialog are persisted app-side in the sled DB and merged into the
//! discovered app list.

use std::path::Path;

use crate::db::{Db, DbError, Key};
use crate::error::{BackendError, ErrorKey};

use super::App;

/// Build an [`App`] from a user-picked path, validating it is a regular file.
pub fn build_custom_app(path: &Path) -> Result<App, BackendError> {
    let metadata = std::fs::metadata(path).map_err(|e| {
        BackendError::new(
            &format!("cannot access selected file '{}': {e}", path.display()),
            ErrorKey::SplitTunnelAppInvalid,
        )
    })?;
    if !metadata.is_file() {
        return Err(BackendError::new(
            &format!("selected path '{}' is not a regular file", path.display()),
            ErrorKey::SplitTunnelAppInvalid,
        ));
    }

    let name = path
        .file_stem()
        .or_else(|| path.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    Ok(App {
        name,
        executable_path: path.to_string_lossy().into_owned(),
        icon: None,
        is_custom: true,
    })
}

/// Append `app` to `list`, rejecting a duplicate `executable_path`.
pub fn insert_unique(list: &mut Vec<App>, app: App) -> Result<(), BackendError> {
    if list.iter().any(|a| a.executable_path == app.executable_path) {
        return Err(BackendError::new(
            &format!("app '{}' is already in the custom split tunnel list", app.executable_path),
            ErrorKey::SplitTunnelAppDuplicate,
        ));
    }
    list.push(app);
    Ok(())
}

/// Remove any entry whose `executable_path` matches `path`.
pub fn remove(list: &mut Vec<App>, path: &str) {
    list.retain(|a| a.executable_path != path);
}

/// Merge discovered + custom apps, deduped by `executable_path`, sorted by name.
pub fn merge(mut discovered: Vec<App>, custom: Vec<App>) -> Vec<App> {
    for app in custom {
        if !discovered.iter().any(|a| a.executable_path == app.executable_path) {
            discovered.push(app);
        }
    }
    discovered.sort_by_key(|a| a.name.to_lowercase());
    discovered
}

/// Load the persisted custom app list (empty if unset).
pub fn load(db: &Db) -> Result<Vec<App>, DbError> {
    let mut apps = db
        .get_typed::<Vec<App>>(Key::CustomSplitTunnelApps.as_ref())?
        .unwrap_or_default();
    // Everything stored here is user-added; enforce the flag so the UI can
    // offer removal (also upgrades entries persisted before the field existed).
    for app in &mut apps {
        app.is_custom = true;
    }
    Ok(apps)
}

/// Persist the custom app list.
pub fn save(db: &Db, apps: &[App]) -> Result<(), DbError> {
    db.insert(Key::CustomSplitTunnelApps.as_ref(), apps)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nymvpn-custom-apps-{}-{}", std::process::id(), tag));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn app(name: &str, path: &str) -> App {
        App {
            name: name.to_string(),
            executable_path: path.to_string(),
            icon: None,
            is_custom: false,
        }
    }

    #[test]
    fn build_custom_app_from_regular_file() {
        let dir = scratch_dir("build-ok");
        let file = dir.join("my-binary");
        fs::write(&file, b"#!/bin/sh\n").unwrap();

        let result = build_custom_app(&file).unwrap();
        assert_eq!(result.name, "my-binary");
        assert_eq!(result.executable_path, file.to_string_lossy().into_owned());
        assert_eq!(result.icon, None);
        assert!(result.is_custom);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_custom_app_strips_extension_for_name() {
        let dir = scratch_dir("build-ext");
        let file = dir.join("Cursor.AppImage");
        fs::write(&file, b"x").unwrap();

        let result = build_custom_app(&file).unwrap();
        assert_eq!(result.name, "Cursor");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_custom_app_rejects_directory() {
        let dir = scratch_dir("build-dir");
        let err = build_custom_app(&dir).unwrap_err();
        assert_eq!(err.key, ErrorKey::SplitTunnelAppInvalid);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_custom_app_rejects_missing_path() {
        let missing = std::env::temp_dir().join("nymvpn-custom-apps-definitely-missing-xyz");
        let err = build_custom_app(&missing).unwrap_err();
        assert_eq!(err.key, ErrorKey::SplitTunnelAppInvalid);
    }

    #[test]
    fn insert_unique_adds_then_rejects_duplicate() {
        let mut list = Vec::new();
        insert_unique(&mut list, app("foo", "/usr/bin/foo")).unwrap();
        assert_eq!(list.len(), 1);

        let err = insert_unique(&mut list, app("foo-again", "/usr/bin/foo")).unwrap_err();
        assert_eq!(err.key, ErrorKey::SplitTunnelAppDuplicate);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn remove_drops_only_matching_entry() {
        let mut list = vec![app("foo", "/usr/bin/foo"), app("bar", "/usr/bin/bar")];
        remove(&mut list, "/usr/bin/foo");
        assert_eq!(list, vec![app("bar", "/usr/bin/bar")]);
    }

    #[test]
    fn merge_dedups_by_path_and_sorts_by_name() {
        let discovered = vec![app("Zed", "/usr/bin/zed"), app("Firefox", "/usr/bin/firefox")];
        let custom = vec![
            app("Firefox copy", "/usr/bin/firefox"), // same path as discovered -> dropped
            app("Custom", "/opt/custom/app"),
        ];

        let merged = merge(discovered, custom);

        let names: Vec<&str> = merged.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["Custom", "Firefox", "Zed"]);
        assert_eq!(
            merged.iter().filter(|a| a.executable_path == "/usr/bin/firefox").count(),
            1
        );
    }
}
