use serde::Serialize;
use std::collections::HashMap;
use std::os::windows::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tracing::{debug, info};
use ts_rs::TS;

use crate::error::BackendError;
use crate::icon_extractor::extract_icon_to_cache;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct App {
    pub name: String,
    /// Absolute path to the `.exe` file, when resolvable.
    pub executable_path: String,
    /// Absolute path to the icon file.
    pub icon: Option<String>,
}

// ─── Public entry point ──────────────────────────────────────────────────────

/// Return a deduplicated, sorted list of all applications installed on the
/// current Windows system.  Sources:
///
/// 1. All-users **and** per-user Start Menu `.lnk` shortcuts
pub fn get_installed_apps(app: tauri::AppHandle) -> Result<Vec<App>, BackendError> {
    let apps = get_windows_apps(app);
    apps
}

pub fn get_windows_apps(app: tauri::AppHandle) -> Result<Vec<App>, BackendError> {
    let mut apps = HashMap::new();

    scan_start_menu(&mut apps);

    let mut result: Vec<App> = apps.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    debug!("discovered {} Windows apps", result.len());

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| BackendError::internal(&e.to_string(), None))?
        .join("icons");

    info!("icon cache dir: {}", cache_dir.display());

    for entry in &mut result {
        if let Some(icon) = &entry.icon {
            let icon =
                extract_icon_to_cache(&icon, &cache_dir).map(|p| p.to_string_lossy().into_owned());
            entry.icon = icon;
        }
    }

    Ok(result)
}

fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        // All-users
        PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"),
    ];
    // Current user
    if let Some(data) = dirs::data_dir() {
        dirs.push(data.join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    dirs
}

fn scan_start_menu(apps: &mut HashMap<String, App>) {
    for dir in start_menu_dirs() {
        if dir.is_dir() {
            scan_lnk_dir(&dir, apps);
        }
    }
}

fn scan_lnk_dir(dir: &Path, apps: &mut HashMap<String, App>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_lnk_dir(&path, apps);
        } else if path
            .extension()
            .map_or(false, |e| e.eq_ignore_ascii_case("lnk"))
        {
            if let Some(app) = parse_lnk(&path) {
                apps.entry(app.name.clone()).or_insert(app);
            }
        }
    }
}

/// Resolve a `.lnk` shortcut to a `App` via the Windows Shell COM API.
fn parse_lnk(lnk_path: &Path) -> Option<App> {
    let name = lnk_path.file_stem()?.to_string_lossy().into_owned();

    // Skip uninstall / help shortcuts
    let name_lower = name.to_lowercase();
    if name_lower.contains("uninstall") || name_lower.contains("remove") {
        return None;
    }

    let target = unsafe { com_resolve_lnk(lnk_path) }?;

    let p = PathBuf::from(&target);
    // Only care about shortcuts that point to an executable
    if !p
        .extension()
        .map_or(false, |e| e.eq_ignore_ascii_case("exe"))
    {
        return None;
    }
    if !p.exists() {
        return None;
    }

    // let icon = extract_icon_to_cache(exe_path, cache_dir)

    Some(App {
        name,
        icon: Some(target.clone()),
        executable_path: target,
    })
}

/// Use `IShellLinkW` + `IPersistFile` (COM) to read the target path of a `.lnk`.
///
/// # Safety
/// Calls Windows COM APIs.  All pointers are either stack-allocated or come from
/// the operating system; no undefined behaviour is expected.
unsafe fn com_resolve_lnk(lnk_path: &Path) -> Option<String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        IPersistFile, STGM,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{Interface, PCWSTR};

    // Initialize COM for this thread.  Ignore RPC_E_CHANGED_MODE which means
    // COM is already initialized with a different model — that is fine.
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

    let link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.ok()?;

    let persist: IPersistFile = link.cast().ok()?;

    let wide_path: Vec<u16> = lnk_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // STGM_READ = 0
    unsafe { persist.Load(PCWSTR(wide_path.as_ptr()), STGM(0)) }.ok()?;

    // Resolve network/moved targets silently (SLR_NO_UI = 0x1)
    let _ = unsafe { link.Resolve(HWND(std::ptr::null_mut()), 0x1u32) };

    let mut buf = [0u16; 32768];
    let mut find_data = unsafe { std::mem::zeroed::<WIN32_FIND_DATAW>() };
    // fflags: SLGP_RAWPATH = 0x4, or 0 for default
    unsafe { link.GetPath(&mut buf, &mut find_data, 0u32) }.ok()?;

    let len = buf.iter().position(|&c| c == 0)?;
    if len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len]))
}

// // ─── Start Menu shortcuts (.lnk) ─────────────────────────────────────────────

// fn start_menu_dirs() -> Vec<PathBuf> {
//     let mut dirs = vec![
//         // All-users
//         PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"),
//     ];
//     // Current user
//     if let Some(data) = dirs::data_dir() {
//         dirs.push(data.join(r"Microsoft\Windows\Start Menu\Programs"));
//     }
//     dirs
// }

// fn scan_start_menu(apps: &mut HashMap<String, App>) {
//     for dir in start_menu_dirs() {
//         if dir.is_dir() {
//             scan_lnk_dir(&dir, apps);
//         }
//     }
// }

// fn scan_lnk_dir(dir: &Path, apps: &mut HashMap<String, App>) {
//     let Ok(entries) = std::fs::read_dir(dir) else {
//         return;
//     };
//     for entry in entries.flatten() {
//         let path = entry.path();
//         if path.is_dir() {
//             scan_lnk_dir(&path, apps);
//         } else if path
//             .extension()
//             .map_or(false, |e| e.eq_ignore_ascii_case("lnk"))
//         {
//             if let Some(app) = parse_lnk(&path) {
//                 apps.entry(app.name.clone()).or_insert(app);
//             }
//         }
//     }
// }

// /// Resolve a `.lnk` shortcut to a `App` via the Windows Shell COM API.
// fn parse_lnk(lnk_path: &Path) -> Option<App> {
//     let name = lnk_path.file_stem()?.to_string_lossy().into_owned();

//     // Skip uninstall / help shortcuts
//     let name_lower = name.to_lowercase();
//     if name_lower.contains("uninstall") || name_lower.contains("remove") {
//         return None;
//     }

//     let target = unsafe { com_resolve_lnk(lnk_path) }?;

//     let p = PathBuf::from(&target);
//     // Only care about shortcuts that point to an executable
//     if !p
//         .extension()
//         .map_or(false, |e| e.eq_ignore_ascii_case("exe"))
//     {
//         return None;
//     }
//     if !p.exists() {
//         return None;
//     }

//     // let icon = extract_icon_to_cache(exe_path, cache_dir)

//     Some(App {
//         name,
//         icon: target.clone(),
//         executable_path: target,
//     })
// }

// /// Use `IShellLinkW` + `IPersistFile` (COM) to read the target path of a `.lnk`.
// ///
// /// # Safety
// /// Calls Windows COM APIs.  All pointers are either stack-allocated or come from
// /// the operating system; no undefined behaviour is expected.
// unsafe fn com_resolve_lnk(lnk_path: &Path) -> Option<String> {
//     use windows::Win32::Foundation::HWND;
//     use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
//     use windows::Win32::System::Com::{
//         CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
//         IPersistFile, STGM,
//     };
//     use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
//     use windows::core::{Interface, PCWSTR};

//     // Initialize COM for this thread.  Ignore RPC_E_CHANGED_MODE which means
//     // COM is already initialized with a different model — that is fine.
//     let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

//     let link: IShellLinkW =
//         unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.ok()?;

//     let persist: IPersistFile = link.cast().ok()?;

//     let wide_path: Vec<u16> = lnk_path
//         .as_os_str()
//         .encode_wide()
//         .chain(std::iter::once(0))
//         .collect();

//     // STGM_READ = 0
//     unsafe { persist.Load(PCWSTR(wide_path.as_ptr()), STGM(0)) }.ok()?;

//     // Resolve network/moved targets silently (SLR_NO_UI = 0x1)
//     let _ = unsafe { link.Resolve(HWND(std::ptr::null_mut()), 0x1u32) };

//     let mut buf = [0u16; 32768];
//     let mut find_data = unsafe { std::mem::zeroed::<WIN32_FIND_DATAW>() };
//     // fflags: SLGP_RAWPATH = 0x4, or 0 for default
//     unsafe { link.GetPath(&mut buf, &mut find_data, 0u32) }.ok()?;

//     let len = buf.iter().position(|&c| c == 0)?;
//     if len == 0 {
//         return None;
//     }
//     Some(String::from_utf16_lossy(&buf[..len]))
// }
