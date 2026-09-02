use std::{
    fmt, iter,
    os::windows::ffi::OsStrExt,
    sync::atomic::{AtomicBool, Ordering},
};

use windows::{
    Win32::UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{
            IDYES, MB_ICONERROR, MB_SETFOREGROUND, MB_YESNO, MessageBoxW, SW_SHOWNORMAL,
        },
    },
    core::{PCWSTR, w},
};
use windows_version::{OsVersion, is_server};

const WINDOWS_UPGRADE_URL: &str = "https://support.microsoft.com/windows/upgrade-to-windows-11-faq-fb6206a2-1a0f-448a-80f1-8668ee5b2bf9";
const WEBVIEW2_DOWNLOAD_URL: &str =
    "https://developer.microsoft.com/microsoft-edge/webview2/#download-section";
const NYM_SUPPORT_URL: &str = "https://support.nym.com/";
const MINIMUM_WINDOWS_MAJOR_VERSION: u32 = 10;

static NATIVE_ERROR_SHOWN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WindowsVersion {
    major: u32,
    minor: u32,
    build: u32,
    server: bool,
}

impl fmt::Display for WindowsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.build)?;
        if self.server {
            write!(f, " (server)")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WindowsCompatibility {
    Supported,
    Unsupported,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WebView2Compatibility {
    Available(String),
    MissingOrDamaged(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeStartupError {
    UnsupportedWindows,
    WebView2Unavailable,
    Unexpected,
}

impl NativeStartupError {
    fn dialog(self) -> NativeDialog {
        match self {
            Self::UnsupportedWindows => NativeDialog {
                title: "NymVPN requires a newer version of Windows",
                message: "This version of Windows is not supported. NymVPN requires Windows 10 or Windows 11. Upgrade Windows, then try again.\n\nOpen Microsoft's Windows upgrade help now?",
                action_url: WINDOWS_UPGRADE_URL,
            },
            Self::WebView2Unavailable => NativeDialog {
                title: "NymVPN needs Microsoft Edge WebView2",
                message: "NymVPN could not start because Microsoft Edge WebView2 Runtime is missing, damaged, or could not be initialized. WebView2 displays the NymVPN interface. Install or repair it, then reopen NymVPN.\n\nOpen Microsoft's WebView2 download page now?",
                action_url: WEBVIEW2_DOWNLOAD_URL,
            },
            Self::Unexpected => NativeDialog {
                title: "NymVPN could not start",
                message: "NymVPN encountered a problem while starting. Restart the app and try again. If the problem continues, reinstall NymVPN or contact Nym Support.\n\nOpen Nym Support now?",
                action_url: NYM_SUPPORT_URL,
            },
        }
    }
}

struct NativeDialog {
    title: &'static str,
    message: &'static str,
    action_url: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PrerequisiteError {
    #[error("unsupported Windows version: {0}")]
    UnsupportedWindows(WindowsVersion),
    #[error("Microsoft Edge WebView2 Runtime is unavailable: {0}")]
    WebView2Unavailable(String),
}

impl PrerequisiteError {
    pub(crate) fn native_error(&self) -> NativeStartupError {
        match self {
            Self::UnsupportedWindows(_) => NativeStartupError::UnsupportedWindows,
            Self::WebView2Unavailable(_) => NativeStartupError::WebView2Unavailable,
        }
    }
}

pub(crate) fn check_prerequisites() -> Result<(), PrerequisiteError> {
    let version = detect_windows_version();
    match classify_windows_version(version) {
        WindowsCompatibility::Supported => {
            if let Some(version) = version {
                tracing::info!(%version, "supported Windows version detected");
            }
        }
        WindowsCompatibility::Unsupported => {
            if let Some(version) = version {
                return Err(PrerequisiteError::UnsupportedWindows(version));
            }
            tracing::warn!("Windows compatibility was unknown; continuing startup");
        }
        WindowsCompatibility::Unknown => {
            tracing::warn!("failed to determine the Windows version; continuing startup");
        }
    }

    match classify_webview_probe(tauri::webview_version().map_err(|error| error.to_string())) {
        WebView2Compatibility::Available(version) => {
            tracing::info!(%version, "Microsoft Edge WebView2 Runtime detected");
            Ok(())
        }
        WebView2Compatibility::MissingOrDamaged(reason) => {
            Err(PrerequisiteError::WebView2Unavailable(reason))
        }
    }
}

pub(crate) fn show_native_error(error: NativeStartupError) {
    if NATIVE_ERROR_SHOWN.swap(true, Ordering::SeqCst) {
        return;
    }

    let dialog = error.dialog();
    let title = wide_string(dialog.title);
    let message = wide_string(dialog.message);

    let result = unsafe {
        MessageBoxW(
            None,
            PCWSTR(message.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_ICONERROR | MB_YESNO | MB_SETFOREGROUND,
        )
    };

    if result == IDYES {
        open_url(dialog.action_url);
    }
}

fn open_url(url: &str) {
    let url = wide_string(url);
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(url.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        tracing::warn!(code = result.0 as isize, "failed to open startup help URL");
    }
}

fn detect_windows_version() -> Option<WindowsVersion> {
    let version = OsVersion::current();
    if version.major == 0 {
        return None;
    }

    Some(WindowsVersion {
        major: version.major,
        minor: version.minor,
        build: version.build,
        server: is_server(),
    })
}

fn classify_windows_version(version: Option<WindowsVersion>) -> WindowsCompatibility {
    match version {
        None => WindowsCompatibility::Unknown,
        Some(version) if version.server || version.major < MINIMUM_WINDOWS_MAJOR_VERSION => {
            WindowsCompatibility::Unsupported
        }
        Some(_) => WindowsCompatibility::Supported,
    }
}

fn classify_webview_probe(probe: Result<String, String>) -> WebView2Compatibility {
    match probe {
        Ok(version) if !version.trim().is_empty() && version.trim() != "0.0.0.0" => {
            WebView2Compatibility::Available(version)
        }
        Ok(version) => WebView2Compatibility::MissingOrDamaged(format!(
            "the reported runtime version is invalid: {version:?}"
        )),
        Err(error) => WebView2Compatibility::MissingOrDamaged(error),
    }
}

fn wide_string(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client_version(major: u32, minor: u32, build: u32) -> WindowsVersion {
        WindowsVersion {
            major,
            minor,
            build,
            server: false,
        }
    }

    #[test]
    fn rejects_legacy_windows_clients() {
        for version in [
            client_version(6, 1, 7601),
            client_version(6, 2, 9200),
            client_version(6, 3, 9600),
        ] {
            assert_eq!(
                classify_windows_version(Some(version)),
                WindowsCompatibility::Unsupported
            );
        }
    }

    #[test]
    fn accepts_windows_10_11_and_future_client_versions() {
        for version in [
            client_version(10, 0, 10240),
            client_version(10, 0, 19045),
            client_version(10, 0, 26100),
            client_version(11, 0, 1),
        ] {
            assert_eq!(
                classify_windows_version(Some(version)),
                WindowsCompatibility::Supported
            );
        }
    }

    #[test]
    fn rejects_windows_server() {
        let version = WindowsVersion {
            server: true,
            ..client_version(10, 0, 26100)
        };
        assert_eq!(
            classify_windows_version(Some(version)),
            WindowsCompatibility::Unsupported
        );
    }

    #[test]
    fn unknown_windows_version_fails_open() {
        assert_eq!(
            classify_windows_version(None),
            WindowsCompatibility::Unknown
        );
    }

    #[test]
    fn classifies_webview2_probe_results() {
        assert!(matches!(
            classify_webview_probe(Ok("151.0.4129.50".to_owned())),
            WebView2Compatibility::Available(_)
        ));
        assert!(matches!(
            classify_webview_probe(Ok(String::new())),
            WebView2Compatibility::MissingOrDamaged(_)
        ));
        assert!(matches!(
            classify_webview_probe(Ok("0.0.0.0".to_owned())),
            WebView2Compatibility::MissingOrDamaged(_)
        ));
        assert!(matches!(
            classify_webview_probe(Err("probe failed".to_owned())),
            WebView2Compatibility::MissingOrDamaged(_)
        ));
    }

    #[test]
    fn maps_prerequisite_failures_to_actionable_dialogs() {
        assert_eq!(
            PrerequisiteError::UnsupportedWindows(client_version(6, 3, 9600)).native_error(),
            NativeStartupError::UnsupportedWindows
        );
        assert_eq!(
            PrerequisiteError::WebView2Unavailable("probe failed".to_owned()).native_error(),
            NativeStartupError::WebView2Unavailable
        );

        assert_eq!(
            NativeStartupError::UnsupportedWindows.dialog().action_url,
            WINDOWS_UPGRADE_URL
        );
        assert_eq!(
            NativeStartupError::WebView2Unavailable.dialog().action_url,
            WEBVIEW2_DOWNLOAD_URL
        );
        assert_eq!(
            NativeStartupError::Unexpected.dialog().action_url,
            NYM_SUPPORT_URL
        );
    }
}
