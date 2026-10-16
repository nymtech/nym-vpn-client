## maintenance notes

When updating `tauri` version (Rust) there are some files that\
need to be checked for upstream changes. They must be updated\
manually if needed.

### Windows NSIS installer hooks

We use the upstream NSIS template and extend it with installer hooks\
`src-tauri/bundle/windows/hooks.nsh`

The hooks rely on template internals (`$UpdateMode`, `$DeleteAppDataCheckboxState`,
`$AppStartMenuFolder`, the `ARCH` define and the position of the `NSIS_HOOK_*`
macros), so check them against upstream when bumping tauri.

Upstream source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi

### deb desktop entry

Due to specific needs, we use a customized desktop entry template\
`src-tauri/bundle/deb/main.desktop`

Upstream source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/linux/freedesktop/main.desktop
