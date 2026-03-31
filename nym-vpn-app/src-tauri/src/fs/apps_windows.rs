#![cfg(windows)]

use serde::Serialize;
use std::collections::HashMap;
use std::os::windows::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use tracing::debug;
use ts_rs::TS;
use winreg::{RegKey, enums::*};

/// An installed application discovered on Windows.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct WindowsApp {
    pub name: String,
    /// Absolute path to the `.exe` file, when resolvable.
    pub executable_path: Option<String>,
    /// Absolute path to the icon file (`.exe`, `.ico`, `.dll`, …).
    pub icon: Option<String>,
}

// ─── Public entry point ──────────────────────────────────────────────────────

/// Return a deduplicated, sorted list of all applications installed on the
/// current Windows system.  Sources:
///
/// 1. Three registry `Uninstall` hives (HKLM 64-bit, HKLM 32-bit WOW, HKCU)
/// 2. All-users **and** per-user Start Menu `.lnk` shortcuts
/// 3. Microsoft Store / AppX packages (per-user registry branch)
pub fn get_installed_apps() -> Vec<WindowsApp> {
    let mut apps: HashMap<String, WindowsApp> = HashMap::new();

    // scan_registry_uninstall(&mut apps);
    scan_start_menu(&mut apps);
    // scan_appx_packages(&mut apps);

    let mut result: Vec<WindowsApp> = apps.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    debug!("discovered {} Windows apps", result.len());
    result
}

// ─── Registry Uninstall hives ────────────────────────────────────────────────

fn scan_registry_uninstall(apps: &mut HashMap<String, WindowsApp>) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let hives: &[(&RegKey, &str)] = &[
        // 64-bit apps
        (
            &hklm,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        // 32-bit apps on a 64-bit OS
        (
            &hklm,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        // Per-user installs
        (
            &hkcu,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for (hive, path) in hives {
        let Ok(key) = hive.open_subkey(path) else {
            continue;
        };
        for subkey_name in key.enum_keys().flatten() {
            let Ok(subkey) = key.open_subkey(&subkey_name) else {
                continue;
            };
            read_uninstall_entry(&subkey, apps);
        }
    }
}

fn read_uninstall_entry(key: &RegKey, apps: &mut HashMap<String, WindowsApp>) {
    // Skip drivers, system components, and Windows updates
    let system_component: u32 = key.get_value("SystemComponent").unwrap_or(0);
    if system_component != 0 {
        return;
    }

    let Ok(name): Result<String, _> = key.get_value("DisplayName") else {
        return;
    };
    let name = name.trim().to_string();
    if name.is_empty() || looks_like_update(&name) {
        return;
    }

    let display_icon: Option<String> = key.get_value("DisplayIcon").ok();
    let install_location: Option<String> = key.get_value("InstallLocation").ok();

    let executable_path = resolve_exe_from_registry(&display_icon, &install_location);
    let icon = resolve_icon_path(display_icon.as_deref());

    apps.entry(name.clone()).or_insert(WindowsApp {
        name,
        executable_path,
        icon,
    });
}

fn looks_like_update(name: &str) -> bool {
    let l = name.to_lowercase();
    l.starts_with("kb")
        || l.contains("security update")
        || l.contains("update for windows")
        || l.contains("hotfix for")
        || l.contains("cumulative update")
}

/// Strip the optional icon-index suffix from a `DisplayIcon` registry value.
///
/// Values look like:  `"C:\foo\app.exe,0"`  or  `C:\foo\app.exe,-1`
fn strip_icon_index(s: &str) -> &str {
    let s = s.trim();
    if let Some(idx) = s.rfind(',') {
        let after = s[idx + 1..].trim();
        // Only strip if everything after the comma is a (possibly signed) integer
        if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return s[..idx].trim().trim_matches('"');
        }
    }
    s.trim_matches('"')
}

fn resolve_exe_from_registry(
    display_icon: &Option<String>,
    install_location: &Option<String>,
) -> Option<String> {
    // 1. DisplayIcon often points straight to an .exe
    if let Some(raw) = display_icon {
        let p = PathBuf::from(strip_icon_index(raw));
        if p.extension()
            .map_or(false, |e| e.eq_ignore_ascii_case("exe"))
            && p.exists()
        {
            return Some(p.to_string_lossy().into_owned());
        }
    }

    // 2. Fall back: find first .exe in InstallLocation
    if let Some(loc) = install_location {
        let dir = PathBuf::from(loc.trim().trim_matches('"'));
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension()
                        .map_or(false, |e| e.eq_ignore_ascii_case("exe"))
                    {
                        return Some(p.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }

    None
}

fn resolve_icon_path(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    let p = PathBuf::from(strip_icon_index(raw));
    p.exists().then(|| p.to_string_lossy().into_owned())
}

// ─── Start Menu shortcuts (.lnk) ─────────────────────────────────────────────

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

fn scan_start_menu(apps: &mut HashMap<String, WindowsApp>) {
    for dir in start_menu_dirs() {
        if dir.is_dir() {
            scan_lnk_dir(&dir, apps);
        }
    }
}

fn scan_lnk_dir(dir: &Path, apps: &mut HashMap<String, WindowsApp>) {
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

/// Resolve a `.lnk` shortcut to a `WindowsApp` via the Windows Shell COM API.
fn parse_lnk(lnk_path: &Path) -> Option<WindowsApp> {
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

    Some(WindowsApp {
        name,
        icon: Some(target.clone()),
        executable_path: Some(target),
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

// ─── Microsoft Store / AppX packages ─────────────────────────────────────────

fn scan_appx_packages(apps: &mut HashMap<String, WindowsApp>) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // Per-user AppX package repository
    let appx_root = r"SOFTWARE\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages";

    let Ok(packages) = hkcu.open_subkey(appx_root) else {
        return;
    };

    for pkg_name in packages.enum_keys().flatten() {
        let Ok(pkg_key) = packages.open_subkey(&pkg_name) else {
            continue;
        };
        let Ok(apps_key) = pkg_key.open_subkey("Applications") else {
            continue;
        };
        for app_id in apps_key.enum_keys().flatten() {
            let Ok(app_key) = apps_key.open_subkey(&app_id) else {
                continue;
            };
            read_appx_entry(&app_key, apps);
        }
    }
}

fn read_appx_entry(key: &RegKey, apps: &mut HashMap<String, WindowsApp>) {
    let Ok(name): Result<String, _> = key.get_value("DisplayName") else {
        return;
    };
    // Skip unresolved resource strings
    if name.is_empty() || name.starts_with("ms-resource:") {
        return;
    }

    let executable_path: Option<String> = key.get_value("Executable").ok().and_then(|p: String| {
        let pb = PathBuf::from(&p);
        pb.exists().then(|| p)
    });

    // Prefer a square logo; fall back to generic logo key
    let icon: Option<String> = key
        .get_value("SquareLogo150x150")
        .or_else(|_| key.get_value("Logo"))
        .ok()
        .and_then(|p: String| PathBuf::from(&p).exists().then_some(p));

    apps.entry(name.clone()).or_insert(WindowsApp {
        name,
        executable_path,
        icon,
    });
}
