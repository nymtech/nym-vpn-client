use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize,
    Window as TauriWindow, WindowBuilder, WindowEvent,
};
use tracing::{debug, error, trace};
use ts_rs::TS;

use crate::db::{Db, Key};

pub struct AppWindow(pub TauriWindow);

impl AppWindow {
    pub fn new(app: &AppHandle, label: &str) -> Result<Self> {
        Ok(AppWindow(app.get_window(label).ok_or_else(|| {
            anyhow!("failed to get window {}", label)
        })?))
    }

    pub fn get_or_create(app: &AppHandle, label: &str) -> Result<Self> {
        let window = app
            .get_window(label)
            .or_else(|| {
                debug!("main window not found, re-creating it");
                WindowBuilder::from_config(app, app.config().tauri.windows.first().unwrap().clone())
                    .build()
                    .inspect_err(|e| error!("failed to create window: {e}"))
                    .ok()
            })
            .ok_or_else(|| anyhow!("failed to get window {}", label))?;
        Ok(AppWindow(window))
    }

    /// restore any saved window size and position
    pub fn setup(&self, db: &Db) -> Result<()> {
        let size = db.get_typed::<WindowSize>(Key::WindowSize)?;
        if let Some(s) = size {
            debug!("___restoring window size: {:?}", s);
            self.0
                .set_size(s)
                .inspect_err(|e| error!("failed to set window size {}", e))
                .ok();
        }

        let position = db.get_typed::<WindowPosition>(Key::WindowPosition)?;
        if let Some(p) = position {
            debug!("___restoring window position: {:?}", p);
            self.0
                .set_position(p)
                .inspect_err(|e| error!("failed to set window position {}", e))
                .ok();
        }
        Ok(())
    }

    pub fn is_visible(&self) -> bool {
        self.0.is_visible().ok().unwrap_or(false)
    }

    pub fn is_minimized(&self) -> bool {
        self.0.is_maximized().ok().unwrap_or(false)
    }

    pub fn _listen_to_resize(&self, db: &Db) {
        let c_db = db.clone();
        self.0.on_window_event(move |event| {
            if let WindowEvent::Resized(s) = event {
                trace!("___RESIZED {}x{}", s.width, s.height);
                c_db.insert(Key::WindowSize, WindowSize::from(s))
                    .inspect_err(|e| error!("failed to save window size: {e}"))
                    .ok();
            }
        });
    }

    pub fn _listen_to_move(&self, db: &Db) {
        let c_db = db.clone();
        self.0.on_window_event(move |event| {
            if let WindowEvent::Moved(p) = event {
                trace!("___MOVED {},{}", p.x, p.y);
                c_db.insert(Key::WindowPosition, WindowPosition::from(p))
                    .inspect_err(|e| error!("failed to save window position: {e}"))
                    .ok();
            }
        });
    }

    // remove splash screen from HTML and show main window
    pub fn no_splash(&self) {
        self.0
            .eval("document.getElementById('splash').remove();")
            .expect("failed to remove splash screen");

        self.0.show().expect("failed to show main window");
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum WindowSize {
    Physical { width: u32, height: u32 },
    Logical { width: f64, height: f64 },
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum WindowPosition {
    Physical { x: i32, y: i32 },
    Logical { x: f64, y: f64 },
}

impl From<WindowSize> for tauri::Size {
    fn from(size: WindowSize) -> Self {
        match size {
            WindowSize::Physical { width, height } => {
                tauri::Size::Physical(PhysicalSize::new(width, height))
            }
            WindowSize::Logical { width, height } => {
                tauri::Size::Logical(LogicalSize::new(width, height))
            }
        }
    }
}

impl From<&PhysicalSize<u32>> for WindowSize {
    fn from(size: &PhysicalSize<u32>) -> Self {
        WindowSize::Physical {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<WindowPosition> for tauri::Position {
    fn from(position: WindowPosition) -> Self {
        match position {
            WindowPosition::Physical { x, y } => {
                tauri::Position::Physical(PhysicalPosition::new(x, y))
            }
            WindowPosition::Logical { x, y } => {
                tauri::Position::Logical(LogicalPosition::new(x, y))
            }
        }
    }
}

impl From<&PhysicalPosition<i32>> for WindowPosition {
    fn from(size: &PhysicalPosition<i32>) -> Self {
        WindowPosition::Physical {
            x: size.x,
            y: size.y,
        }
    }
}
