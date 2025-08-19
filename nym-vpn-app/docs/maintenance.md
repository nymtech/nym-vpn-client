## maintenance notes

When updating `tauri` version (Rust) there are some files that\
need to be checked for upstream changes. They must be updated\
manually if needed.

### Windows NSIS template

Due to the app specific needs, we use a customized NSIS template\
`src-tauri/bundle/windows/installer.nsi`

Upstream source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi

### deb desktop entry

Due to specific needs, we use a customized desktop entry template\
`src-tauri/bundle/deb/main.desktop`

Upstream source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/linux/freedesktop/main.desktop
