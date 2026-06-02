# Linux custom split-tunnel apps — design

Jira: [NYM-1349](https://nymtech.atlassian.net/browse/NYM-1349) (sub-task of [NYM-1107](https://nymtech.atlassian.net/browse/NYM-1107))

## Context

NymVPN's split-tunneling settings list the apps a user can run outside the VPN
tunnel. On **Linux**, that list is built entirely from `get_linux_apps()`, which
parses freedesktop `.desktop` entries; apps without a desktop entry are never
shown. Linux also has **no daemon-side exclude list** — the daemon's
split-tunnel RPCs (`add_split_tunnel_app`, etc.) are no-ops on Linux. Instead the
app _launches_ a chosen executable through the `nym-exclude` cgroup helper
(`Command.create('nym-exclude', …)` in `SplitTunneling.tsx`), which runs that
process outside the tunnel.

NYM-1107 asks for an "Exclude Custom App…" flow so users can pick any executable
via a file dialog. Per the parent ticket's acceptance criteria and andy's
comment, user-added apps are **app-side state** that must persist across restarts
and appear in the list alongside discovered apps.

This task delivers the **Linux logic layer plus a backend-triggered native file
dialog**. The Windows path (daemon exclude-list reconciliation) is handled in a
separate session and is explicitly out of scope here.

## Goal / outcome

- A user can open a native file-picker from the split-tunnel screen, choose an
  executable, and have it added to the app list.
- Custom apps persist across app restarts and show up next to discovered apps,
  launchable via the existing `nym-exclude` flow.
- Custom apps can be removed without affecting discovered apps.

## Data model & storage

Reuse the existing `App` struct
(`src-tauri/src/fs/app_discovery/mod.rs`): `name`, `executable_path`,
`icon: Option<String>`. For a custom app:

- `executable_path` = the picked path.
- `name` = the file stem of the path (e.g. `/opt/foo/bar` → `bar`).
- `icon` = `None`.

Persist a `Vec<App>` in the sled DB under a **new `Key` enum variant**
`CustomSplitTunnelApps` (kebab-case key `custom-split-tunnel-apps`) in
`src-tauri/src/db.rs`. Read/write via the existing `Db::get_typed` /
`Db::insert` helpers. The `Db` is already injected into commands as
`State<'_, Db>` (see `src-tauri/src/commands/db.rs`).

A small set of helpers (load list / save list / add / remove) lives next to the
commands — kept in one focused place so the storage shape has a single owner.

## Commands (`src-tauri/src/commands/tunnel.rs`, registered in `main.rs`)

### `add_custom_split_tunnel_app`

```rust
pub async fn add_custom_split_tunnel_app(
    app: AppHandle,
    db: State<'_, Db>,
) -> Result<Option<App>, BackendError>
```

1. Open the native picker following the `fs.rs` pattern:
   `app.dialog().file().blocking_pick_file()`. On Linux no extension filter is
   applied (Linux executables have no canonical extension; the regular-file
   check below is the real guard).
2. If the dialog returns `None` → user cancelled → return `Ok(None)` (no change,
   no error).
3. Resolve the `FilePath` to a `PathBuf`. Validate it is an **existing regular
   file** (`std::fs::metadata(&path)?.is_file()`); otherwise return a clear
   `BackendError` (non-executable / invalid selection).
4. Load the persisted custom list. If an entry with the same `executable_path`
   already exists → return a duplicate `BackendError` (not added).
5. Build `App { name: <file stem>, executable_path, icon: None }`, append,
   persist, and return `Ok(Some(app))`.

### `remove_custom_split_tunnel_app`

```rust
pub async fn remove_custom_split_tunnel_app(
    db: State<'_, Db>,
    path: String,
) -> Result<(), BackendError>
```

Load the custom list, drop the entry whose `executable_path == path`, persist.
Discovered apps are never touched (they aren't stored).

### `get_app_list` (extended)

```rust
pub async fn get_app_list(
    app: AppHandle,
    db: State<'_, Db>,
) -> Result<Vec<App>, BackendError>
```

Get discovered apps (existing `get_installed_apps` on a blocking thread), load
the persisted custom list, **merge deduped by `executable_path`** (a custom
entry whose path coincides with a discovered app is dropped from the custom
side), and sort by name. This is what surfaces custom apps in the UI and makes
them launchable through the existing `nym-exclude` path.

## Error states (parent AC)

- Non-regular-file selection → clear `BackendError`, nothing added.
- Path already in the custom list → duplicate `BackendError`, nothing added.
- Dialog cancelled → `Ok(None)`, no change, no error.

## Cross-platform note

The storage + merge logic is platform-agnostic and harmless on Windows, but
Windows' daemon-list reconciliation (andy's comment) is **deferred to the
separate Windows session**. This task is built and verified on **Linux only**.

## Types

Running `npm run tsgen` regenerates `DbKey` in `src/types/tauri.ts` (new enum
variant). No new cross-IPC structs — `App` is already exported. The two new
commands must be added to the `tauri::generate_handler!` list in
`src-tauri/src/main.rs`, and `get_app_list` keeps its name (only gains a `db`
arg, transparent to the frontend `invoke('get_app_list')`).

## Testing

Rust unit tests for the storage/validation/merge logic (no dialog needed —
factor the pure logic out of the dialog-driven command so it is testable):

- add → entry persisted; second add of same path → duplicate error, list
  unchanged.
- validation rejects a directory / missing path.
- remove drops only the matching custom entry.
- merge dedups by `executable_path` and keeps discovered entries.

Manual end-to-end on Linux: open split-tunnel settings → trigger the custom-app
command → pick an executable → confirm it appears in the list, persists across
an app restart, launches via `nym-exclude`, and can be removed.

## Alternative considered (rejected)

Pushing custom apps to the daemon via the split-tunnel RPC — rejected because the
daemon maintains no Linux exclude list (the RPCs are no-ops on Linux), so it
would neither persist nor take effect.

## Out of scope

- Windows behaviour (separate session).
- The React UI affordance (the "Exclude Custom App…" button placement / styling
  and toast wiring) beyond invoking the new command — the command is the
  deliverable; minimal frontend invocation can follow.
