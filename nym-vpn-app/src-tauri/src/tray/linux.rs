//! Linux: ksni-based `StatusNotifierItem` backend.
//!
//! We prefer this over Tauri's tray-icon backend on Linux because that GTK implementation
//! only calls `app_indicator_set_menu()`. There's no API to receive left-click separately
//! from the menu open, so we can't bind left-click to "toggle window". ksni implements SNI
//! directly and exposes `activate` / `secondary_activate` callbacks.
//!
//! The catch is that SNI needs a `StatusNotifierWatcher` on the session bus, which minimal
//! window managers like i3 don't provide. [`Backend::try_new`] probes for one and returns
//! `None` when it's absent so the caller ([`super::Backend`]) can fall back to the native
//! XEmbed backend.

use std::sync::LazyLock;
use std::time::Duration;

use image::ImageFormat;
use ksni::{Handle, TrayMethods, menu::StandardItem};
use tauri::AppHandle;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{debug, instrument, trace};

use super::{IconKind, TrayBackend, TrayState, quit_app, show_window};

const APP_ICON: &[u8] = include_bytes!("../../icons/tray_icon.png");
const CONNECTED_ICON: &[u8] = include_bytes!("../../icons/tray_icon_connected.png");
const CONNECTING_ICON: &[u8] = include_bytes!("../../icons/tray_icon_connecting.png");
const DISCONNECTED_ICON: &[u8] = include_bytes!("../../icons/tray_icon_disconnected.png");
const ERROR_ICON: &[u8] = include_bytes!("../../icons/tray_icon_error.png");
const ICON_DEBOUNCE: Duration = Duration::from_millis(300);

fn decode_argb(bytes: &[u8]) -> ksni::Icon {
    let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .expect("embedded tray icon must decode");
    let width = img.width() as i32;
    let height = img.height() as i32;
    let mut data = img.into_rgba8().into_vec();
    // ksni expects ARGB; image gives RGBA.
    for px in data.as_chunks_mut::<4>().0 {
        px.rotate_right(1);
    }
    ksni::Icon {
        width,
        height,
        data,
    }
}

static APP_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(APP_ICON));
static CONNECTED_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(CONNECTED_ICON));
static CONNECTING_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(CONNECTING_ICON));
static DISCONNECTED_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(DISCONNECTED_ICON));
static ERROR_ARGB: LazyLock<ksni::Icon> = LazyLock::new(|| decode_argb(ERROR_ICON));

fn pixmap_for_kind(kind: IconKind) -> ksni::Icon {
    match kind {
        IconKind::Default => APP_ARGB.clone(),
        IconKind::Connected => CONNECTED_ARGB.clone(),
        IconKind::Connecting => CONNECTING_ARGB.clone(),
        IconKind::Disconnected => DISCONNECTED_ARGB.clone(),
        IconKind::Error => ERROR_ARGB.clone(),
    }
}

#[derive(Clone)]
pub(super) struct NymTray {
    app: AppHandle,
    icon: IconKind,
    show_hide: String,
    quit: String,
    status: String,
    mode: String,
    entry: String,
    exit: String,
    entry_visible: bool,
}

impl ksni::Tray for NymTray {
    fn id(&self) -> String {
        // Stable per-app id so the panel can remember the slot.
        "nym-vpn-app".into()
    }

    fn title(&self) -> String {
        crate::APP_NAME.into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: crate::APP_NAME.into(),
            description: self.status.clone(),
            ..Default::default()
        }
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![pixmap_for_kind(self.icon)]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        trace!("tray activate (left-click)");
        let _ = show_window(&self.app, true);
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        trace!("tray secondary_activate (middle-click)");
        let _ = show_window(&self.app, true);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        // Position parity with the desktop menu:
        // show_hide, separator, status, mode, exit, [entry], separator, quit
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::with_capacity(8);

        items.push(
            StandardItem {
                label: self.show_hide.clone(),
                activate: Box::new(|t: &mut Self| {
                    trace!("show/hide menu clicked");
                    let _ = show_window(&t.app, true);
                }),
                ..Default::default()
            }
            .into(),
        );
        items.push(ksni::MenuItem::Separator);
        items.push(
            StandardItem {
                label: self.status.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );
        items.push(
            StandardItem {
                label: self.mode.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );
        items.push(
            StandardItem {
                label: self.exit.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );
        if self.entry_visible {
            items.push(
                StandardItem {
                    label: self.entry.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(ksni::MenuItem::Separator);
        items.push(
            StandardItem {
                label: self.quit.clone(),
                activate: Box::new(|t: &mut Self| {
                    trace!("quit menu clicked");
                    quit_app(t.app.clone());
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

pub(super) struct Backend {
    // Live ksni handle. Always present: [`Backend::try_new`] only returns `Some`
    // once the tray has successfully registered with a StatusNotifierWatcher, so
    // every `update_*` call has a live tray to drive.
    handle: Handle<NymTray>,
    icon_debounce: Mutex<Option<JoinHandle<()>>>,
    // Mirrors the tray's entry visibility so we can skip redundant menu rebuilds
    // (each `Handle::update` emits a D-Bus layout-changed signal).
    entry_visible: Mutex<bool>,
}

impl Backend {
    /// Build the ksni backend around an already-spawned handle (see
    /// [`spawn_ksni_once`]). `entry_visible` seeds the mirror used to skip
    /// redundant menu rebuilds and must match the spawned tray's state.
    pub(super) fn from_handle(handle: Handle<NymTray>, entry_visible: bool) -> Self {
        Self {
            handle,
            icon_debounce: Mutex::new(None),
            entry_visible: Mutex::new(entry_visible),
        }
    }
}

/// Attempt to bring up the ksni tray once, seeded from `state`.
///
/// Returns `Some(handle)` when a StatusNotifierWatcher is available on the session
/// bus, `None` otherwise. `ksni::spawn()` must be driven to completion (it then
/// detaches its service onto its own async-io executor); we do that on a short-lived
/// thread so this is safe to call from the main thread or a tokio runtime without
/// risking a nested-runtime panic.
pub(super) fn spawn_ksni_once(app: &AppHandle, state: &TrayState) -> Option<Handle<NymTray>> {
    let tray = NymTray {
        app: app.clone(),
        icon: state.icon,
        show_hide: state.show_hide.clone(),
        quit: state.quit.clone(),
        status: state.status.clone(),
        mode: state.mode.clone(),
        entry: state.entry.clone(),
        exit: state.exit.clone(),
        entry_visible: state.entry_visible,
    };
    std::thread::Builder::new()
        .name("ksni-tray-spawn".into())
        .spawn(move || match futures::executor::block_on(tray.spawn()) {
            Ok(handle) => {
                debug!("ksni tray spawned");
                Some(handle)
            }
            Err(e) => {
                trace!("ksni tray spawn failed: {e:?}");
                None
            }
        })
        .ok()?
        .join()
        .ok()
        .flatten()
}

impl TrayBackend for Backend {
    #[instrument(skip_all)]
    async fn update_tray_icon(&self, icon: IconKind) {
        let mut pending = self.icon_debounce.lock().await;
        if let Some(h) = pending.take() {
            h.abort();
        }
        let handle = self.handle.clone();
        *pending = Some(tokio::spawn(async move {
            tokio::time::sleep(ICON_DEBOUNCE).await;
            handle.update(move |t: &mut NymTray| t.icon = icon).await;
        }));
    }

    async fn update_tray_show_hide(&self, show_hide: String) {
        self.handle
            .update(move |t: &mut NymTray| t.show_hide = show_hide)
            .await;
    }

    async fn update_tray_quit(&self, quit: String) {
        self.handle
            .update(move |t: &mut NymTray| t.quit = quit)
            .await;
    }

    async fn update_tray_mode(&self, mode: String) {
        self.handle
            .update(move |t: &mut NymTray| t.mode = mode)
            .await;
    }

    async fn update_tray_state(&self, state: String) {
        self.handle
            .update(move |t: &mut NymTray| t.status = state)
            .await;
    }

    async fn update_tray_entry(&self, entry: String) {
        self.handle
            .update(move |t: &mut NymTray| t.entry = entry)
            .await;
    }

    async fn update_tray_exit(&self, exit: String) {
        self.handle
            .update(move |t: &mut NymTray| t.exit = exit)
            .await;
    }

    async fn update_tray_entry_visible(&self, visible: bool) {
        let mut current = self.entry_visible.lock().await;
        if *current == visible {
            return;
        }
        *current = visible;
        self.handle
            .update(move |t: &mut NymTray| t.entry_visible = visible)
            .await;
    }
}
