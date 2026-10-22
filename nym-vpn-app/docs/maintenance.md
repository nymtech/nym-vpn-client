## maintenance notes

When updating `tauri` (the Rust crate or the CLI) there are some files that\
need to be checked for upstream changes. They must be updated\
manually if needed.

### Windows NSIS installer hooks

We use the upstream NSIS template and extend it with installer hooks\
`src-tauri/bundle/windows/hooks.nsh`

The hooks rely on template internals (`$UpdateMode`, `$DeleteAppDataCheckboxState`,
`$AppStartMenuFolder`, the `ARCH` define and the position of the `NSIS_HOOK_*`
macros relative to `CheckIfAppIsRunning`, `WriteUninstaller` and the uninstall
registry entry), so check them against upstream when bumping tauri.

The template ships with `tauri-bundler`, which is pinned by the Tauri CLI
(`@tauri-apps/cli` in `package.json`), not by the Rust `tauri` crate. Check the
hooks whenever the CLI version changes, against the template of the bundler
version it pins (`tauri-bundler` in the root `Cargo.toml` at the matching
`tauri-cli-v<version>` tag).

Upstream source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi

### deb desktop entry

Due to specific needs, we use a customized desktop entry template\
`src-tauri/bundle/deb/main.desktop`

Upstream source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/linux/freedesktop/main.desktop
