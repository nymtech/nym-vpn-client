# Linux Custom Split-Tunnel Apps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let Linux users add any executable to the split-tunnel app list via a backend-triggered native file dialog, persisted app-side and merged with discovered apps.

**Architecture:** A new `custom_apps` module under `fs/app_discovery/` owns the pure logic (validate a picked file, dedup, merge, sort) and thin sled-DB read/write helpers. Three Tauri commands in `commands/tunnel.rs` orchestrate it: `add_custom_split_tunnel_app` (opens the native picker), `remove_custom_split_tunnel_app`, and an extended `get_app_list` that merges discovered + custom apps. Linux has no daemon exclude list, so all state lives in the app's sled DB.

**Tech Stack:** Rust + Tauri 2, `tauri-plugin-dialog` 2.6, sled (via the existing `Db` wrapper), `ts-rs` for type generation.

**Spec:** `docs/superpowers/specs/2026-06-02-linux-custom-split-tunnel-apps-design.md`

---

## File Structure

- **Modify** `src-tauri/src/fs/app_discovery/mod.rs` — add `Deserialize, PartialEq` to `App`; register `pub mod custom_apps;`.
- **Create** `src-tauri/src/fs/app_discovery/custom_apps.rs` — pure logic (`build_custom_app`, `insert_unique`, `remove`, `merge`) + DB helpers (`load`, `save`) + unit tests.
- **Modify** `src-tauri/src/db.rs` — add `CustomSplitTunnelApps` to the `Key` enum.
- **Modify** `src-tauri/src/error.rs` — add `SplitTunnelAppInvalid` + `SplitTunnelAppDuplicate` to `ErrorKey`; add `PartialEq` to `ErrorKey` derive.
- **Modify** `src-tauri/src/commands/tunnel.rs` — add the two new commands; extend `get_app_list`.
- **Modify** `src-tauri/src/main.rs` — register the two new commands.
- **Regenerate** `src/types/tauri.ts` — via `npm run tsgen` (updates `DbKey` + `ErrorKey`).

---

## Task 1: Type plumbing (derives, error keys, db key)

**Files:**

- Modify: `src-tauri/src/fs/app_discovery/mod.rs:1-22`
- Modify: `src-tauri/src/db.rs:28-42`
- Modify: `src-tauri/src/error.rs:112-150`

- [ ] **Step 1: Make `App` storable and comparable**

In `src-tauri/src/fs/app_discovery/mod.rs`, change the import and derive. Replace line 2:

```rust
use serde::{Deserialize, Serialize};
```

Replace the derive on `App` (line 14):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct App {
    pub name: String,
    /// Absolute path to the main executable.
    pub executable_path: String,
    /// Absolute path to the cached icon PNG, when available. Stored in tauri app cache directory.
    pub icon: Option<String>,
}
```

- [ ] **Step 2: Add the sled DB key**

In `src-tauri/src/db.rs`, add a variant to the `Key` enum (after `NetworkStatsEnabled`, around line 35):

```rust
    NetworkStatsEnabled,
    // user-added custom split tunnel apps (app-side state, Linux)
    CustomSplitTunnelApps,
```

- [ ] **Step 3: Add the error keys**

In `src-tauri/src/error.rs`, add `PartialEq` to the `ErrorKey` derive (line 112):

```rust
#[derive(Debug, Serialize, TS, Clone, PartialEq)]
```

Then add two variants to `ErrorKey` (before the country-query group, after `DeviceTimeDesync`):

```rust
    DeviceTimeDesync,
    // Custom split tunnel app errors (app backend layer)
    SplitTunnelAppInvalid,
    SplitTunnelAppDuplicate,
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo build`
Expected: builds successfully (warnings about unused `CustomSplitTunnelApps` variant are OK — `Key` already has `#[allow(dead_code)]`).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/fs/app_discovery/mod.rs src-tauri/src/db.rs src-tauri/src/error.rs
git commit -m "feat(split-tunnel): add types for custom apps storage"
```

---

## Task 2: `custom_apps` module — pure logic + storage (TDD)

**Files:**

- Create: `src-tauri/src/fs/app_discovery/custom_apps.rs`
- Modify: `src-tauri/src/fs/app_discovery/mod.rs` (register module)

- [ ] **Step 1: Create the module with stubs and register it**

Create `src-tauri/src/fs/app_discovery/custom_apps.rs`:

```rust
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
pub fn build_custom_app(_path: &Path) -> Result<App, BackendError> {
    todo!()
}

/// Append `app` to `list`, rejecting a duplicate `executable_path`.
pub fn insert_unique(_list: &mut Vec<App>, _app: App) -> Result<(), BackendError> {
    todo!()
}

/// Remove any entry whose `executable_path` matches `path`.
pub fn remove(_list: &mut Vec<App>, _path: &str) {
    todo!()
}

/// Merge discovered + custom apps, deduped by `executable_path`, sorted by name.
pub fn merge(_discovered: Vec<App>, _custom: Vec<App>) -> Vec<App> {
    todo!()
}

/// Load the persisted custom app list (empty if unset).
pub fn load(_db: &Db) -> Result<Vec<App>, DbError> {
    todo!()
}

/// Persist the custom app list.
pub fn save(_db: &Db, _apps: &[App]) -> Result<(), DbError> {
    todo!()
}
```

In `src-tauri/src/fs/app_discovery/mod.rs`, register the module (after the existing `mod` declarations, around line 12):

```rust
pub mod custom_apps;
```

- [ ] **Step 2: Write the failing tests**

Append to `src-tauri/src/fs/app_discovery/custom_apps.rs`:

```rust
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
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib custom_apps`
Expected: tests compile but FAIL/panic with `not yet implemented` (the `todo!()` stubs).

- [ ] **Step 4: Implement the real bodies**

Replace the six stub functions in `src-tauri/src/fs/app_discovery/custom_apps.rs` with:

```rust
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
    })
}

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

pub fn remove(list: &mut Vec<App>, path: &str) {
    list.retain(|a| a.executable_path != path);
}

pub fn merge(mut discovered: Vec<App>, custom: Vec<App>) -> Vec<App> {
    for app in custom {
        if !discovered.iter().any(|a| a.executable_path == app.executable_path) {
            discovered.push(app);
        }
    }
    discovered.sort_by_key(|a| a.name.to_lowercase());
    discovered
}

pub fn load(db: &Db) -> Result<Vec<App>, DbError> {
    Ok(db
        .get_typed::<Vec<App>>(Key::CustomSplitTunnelApps.as_ref())?
        .unwrap_or_default())
}

pub fn save(db: &Db, apps: &[App]) -> Result<(), DbError> {
    db.insert(Key::CustomSplitTunnelApps.as_ref(), apps)?;
    Ok(())
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib custom_apps`
Expected: all 7 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/fs/app_discovery/custom_apps.rs src-tauri/src/fs/app_discovery/mod.rs
git commit -m "feat(split-tunnel): custom apps storage and merge logic"
```

---

## Task 3: Wire the Tauri commands

**Files:**

- Modify: `src-tauri/src/commands/tunnel.rs:1-16` (imports), `:248-282` (commands)
- Modify: `src-tauri/src/main.rs:324`

- [ ] **Step 1: Add imports to `tunnel.rs`**

In `src-tauri/src/commands/tunnel.rs`, update the `fs::app_discovery` import (line 5) to include the new module, add the `Db` import, and add the dialog trait. The import block becomes:

```rust
use crate::commands::gateway::Hop;
use crate::{
    db::Db,
    error::{BackendError, ErrorKey},
    events::AppHandleEventEmitter,
    fs::app_discovery::{App, custom_apps, get_installed_apps},
    state::{SharedAppState, app::VpnMode},
    vpnd::{
        client::{Node, VpndClient, VpndError},
        config::{MixnetTrafficConfig, MixnetTrafficDefaults, VpndConfig},
        gateway::GatewaySelectionAlgorithm,
        tunnel::{ConnectingState, FrontingMode, SplitApp, TunnelState},
    },
};
use std::net::IpAddr;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tracing::{debug, info, instrument, warn};
```

(`ErrorKey` is already imported; keep it. If `cargo build` reports `ErrorKey` unused elsewhere, that is pre-existing — leave it.)

- [ ] **Step 2: Extend `get_app_list` to merge custom apps**

Replace the existing `get_app_list` command (`src-tauri/src/commands/tunnel.rs:248-254`):

```rust
#[instrument(skip_all)]
#[tauri::command]
pub async fn get_app_list(
    app: tauri::AppHandle,
    db: State<'_, Db>,
) -> Result<Vec<App>, BackendError> {
    let discovered = tokio::task::spawn_blocking(move || get_installed_apps(app))
        .await
        .map_err(|e| BackendError::internal(&e.to_string(), None))??;
    let custom = custom_apps::load(&db)?;
    Ok(custom_apps::merge(discovered, custom))
}
```

- [ ] **Step 3: Add the two new commands**

In `src-tauri/src/commands/tunnel.rs`, after `is_split_tunnel_supported` (after line 282), add:

```rust
#[instrument(skip_all)]
#[tauri::command]
pub async fn add_custom_split_tunnel_app(
    app: tauri::AppHandle,
    db: State<'_, Db>,
) -> Result<Option<App>, BackendError> {
    let Some(file_path) = app.dialog().file().blocking_pick_file() else {
        info!("user cancelled custom split tunnel app dialog");
        return Ok(None);
    };

    let path = file_path
        .as_path()
        .ok_or_else(|| BackendError::internal("failed to resolve picked file path", None))?
        .to_path_buf();
    info!("[command] add_custom_split_tunnel_app: {}", path.display());

    let new_app = custom_apps::build_custom_app(&path)?;
    let mut apps = custom_apps::load(&db)?;
    custom_apps::insert_unique(&mut apps, new_app.clone())?;
    custom_apps::save(&db, &apps)?;

    Ok(Some(new_app))
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn remove_custom_split_tunnel_app(
    db: State<'_, Db>,
    path: String,
) -> Result<(), BackendError> {
    info!("[command] remove_custom_split_tunnel_app: {path}");
    let mut apps = custom_apps::load(&db)?;
    custom_apps::remove(&mut apps, &path);
    custom_apps::save(&db, &apps)?;
    Ok(())
}
```

- [ ] **Step 4: Register the commands in `main.rs`**

In `src-tauri/src/main.rs`, after `tunnel::is_split_tunnel_supported,` (line 324), add:

```rust
            tunnel::is_split_tunnel_supported,
            tunnel::add_custom_split_tunnel_app,
            tunnel::remove_custom_split_tunnel_app,
```

- [ ] **Step 5: Build and lint**

Run: `cd src-tauri && cargo build && cargo +nightly clippy -- -Dwarnings`
Expected: builds with no errors and no clippy warnings.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/tunnel.rs src-tauri/src/main.rs
git commit -m "feat(split-tunnel): add/remove custom app commands with native dialog"
```

---

## Task 4: Regenerate TypeScript types

**Files:**

- Modify: `src/types/tauri.ts` (generated)

- [ ] **Step 1: Regenerate**

Run: `npm run tsgen`
Expected: `cargo test` runs (all tests pass) and `src/types/tauri.ts` is updated.

- [ ] **Step 2: Verify the generated changes**

Run: `git diff src/types/tauri.ts`
Expected: `DbKey` gains `"custom-split-tunnel-apps"` and `ErrorKey` gains `"split-tunnel-app-invalid"` + `"split-tunnel-app-duplicate"`. No other unexpected changes.

- [ ] **Step 3: Frontend checks**

Run: `npm run tscheck && npm run lint`
Expected: pass (no TS usage changed; only the generated unions grew).

- [ ] **Step 4: Commit**

```bash
git add src/types/tauri.ts
git commit -m "chore: regenerate tauri types for custom split tunnel apps"
```

---

## Final Verification

- [ ] **Rust:** `cd src-tauri && cargo test && cargo +nightly clippy -- -Dwarnings && cargo +nightly fmt --check`
- [ ] **Frontend:** `npm run check`
- [ ] **Manual end-to-end (Linux):** with `npm run dev:app`:
  1. From a JS console / temporary button, `invoke('add_custom_split_tunnel_app')` → native file picker opens.
  2. Pick a regular executable → returns an `App`; `invoke('get_app_list')` shows it merged in.
  3. Restart the app → `invoke('get_app_list')` still shows the custom app (persistence).
  4. Pick the same file again → rejected with `split-tunnel-app-duplicate`.
  5. Pick a directory (or pass a bogus path) → rejected with `split-tunnel-app-invalid`.
  6. Cancel the dialog → returns `null`, no error, list unchanged.
  7. `invoke('remove_custom_split_tunnel_app', { path })` → entry gone; discovered apps untouched.

## Notes for the implementer

- Linux split tunnel has **no daemon exclude list**; do not call the daemon RPCs for custom apps. The custom app shows in the list and is launched by the existing `nym-exclude` flow.
- The duplicate check is intentionally only against the **custom** list (not discovered apps) — adding a path that matches a discovered app is allowed and de-duped at display time by `merge`.
- `blocking_pick_file()` is called directly inside the async command, matching the existing `zip_logs` pattern in `commands/fs.rs`.
