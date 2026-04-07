use super::App;
use super::utils::is_excluded;
use crate::error::BackendError;
use crate::icon_extractor::extract_icon_to_cache;
use itertools::Itertools;
use std::collections::HashMap;
use std::os::windows::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tracing::debug;

pub fn get_windows_apps(app: tauri::AppHandle) -> Result<Vec<App>, BackendError> {
    let mut apps: HashMap<String, App> = HashMap::new();

    scan_start_menu(&mut apps);

    let mut result: Vec<App> = apps
        .into_values()
        .filter(|app| !is_excluded(&app.name))
        .sorted_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        .collect();
    debug!("discovered {} Windows apps", result.len());

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| BackendError::internal(&e.to_string(), None))?
        .join("icons");

    debug!("icon cache dir: {}", cache_dir.display());

    for entry in &mut result {
        if let Some(icon) = &entry.icon {
            entry.icon =
                extract_icon_to_cache(icon, &cache_dir).map(|p| p.to_string_lossy().into_owned());
        }
    }

    Ok(result)
}

fn start_menu_dirs() -> Vec<PathBuf> {
    let program_data =
        std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());

    let mut dirs = vec![
        // All-users
        PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"),
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
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
            && let Some(app) = parse_lnk(&path)
        {
            apps.entry(app.name.clone()).or_insert(app);
        }
    }
}

fn parse_lnk(lnk_path: &Path) -> Option<App> {
    let name = lnk_path.file_stem()?.to_string_lossy().into_owned();

    let name_lower = name.to_lowercase();
    if name_lower.contains("uninstall") || name_lower.contains("remove") {
        return None;
    }

    let target = unsafe { com_resolve_lnk(lnk_path) }?;

    let p = PathBuf::from(&target);
    if !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) {
        return None;
    }
    if !p.exists() {
        return None;
    }

    Some(App {
        name,
        icon: Some(target.clone()),
        executable_path: target,
    })
}

// COM / IShellLink

/// Resolve the target `.exe` path of a `.lnk` file via the Windows Shell COM API.
///
/// # Safety
/// Calls Windows COM APIs with stack-allocated buffers; all handles are released
/// before the function returns.
unsafe fn com_resolve_lnk(lnk_path: &Path) -> Option<String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        IPersistFile, STGM,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{Interface, PCWSTR};

    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

    let link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.ok()?;

    let persist: IPersistFile = link.cast().ok()?;

    let wide_path: Vec<u16> = lnk_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { persist.Load(PCWSTR(wide_path.as_ptr()), STGM(0)) }.ok()?;

    // Resolve silently (SLR_NO_UI = 0x1)
    let _ = unsafe { link.Resolve(HWND(std::ptr::null_mut()), 0x1u32) };

    let mut buf = [0u16; 32768];
    let mut find_data = unsafe { std::mem::zeroed::<WIN32_FIND_DATAW>() };
    unsafe { link.GetPath(&mut buf, &mut find_data, 0u32) }.ok()?;

    let len = buf.iter().position(|&c| c == 0)?;
    if len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..len]))
}
