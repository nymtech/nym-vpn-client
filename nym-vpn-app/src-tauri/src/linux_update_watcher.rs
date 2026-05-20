use anyhow::Result;
use procfs::process::Process;

use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use notify::event::EventKind;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info, warn};

use crate::events::EVENT_UPDATE_PENDING;

const DELETED_SUFFIX: &str = " (deleted)";

/// Returns the path of the running binary, stripping any trailing
/// " (deleted)" suffix that Linux appends after the file has been unlinked.
fn resolve_self_path() -> Result<PathBuf> {
    let process = Process::myself()?;
    let link = process.exe()?;
    let s = link.to_string_lossy();
    if let Some(stripped) = s.strip_suffix(DELETED_SUFFIX) {
        Ok(PathBuf::from(stripped))
    } else {
        Ok(link)
    }
}

/// Returns true if the binary on disk has been replaced since this process
/// started — either the /proc/self/exe link is marked deleted, or the inode
/// of the running binary differs from the file currently at its path.
fn update_detected(binary_path: &Path) -> bool {
    let process = match Process::myself() {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to get process myself: {e}");
            return false;
        }
    };
    let link = match process.exe() {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to get process exe: {e}");
            return false;
        }
    };

    if link.to_string_lossy().ends_with(DELETED_SUFFIX) {
        return true;
    }

    let running_ino = match std::fs::metadata("/proc/self/exe") {
        Ok(m) => m.st_ino(),
        Err(e) => {
            warn!("failed to stat /proc/self/exe: {e}");
            return false;
        }
    };
    let on_disk_ino = match std::fs::metadata(binary_path) {
        Ok(m) => m.st_ino(),
        Err(_) => {
            // The file is gone — that's also an update in progress.
            return true;
        }
    };
    running_ino != on_disk_ino
}

fn emit_once(app: &AppHandle, fired: &AtomicBool) {
    if fired
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        info!("binary on disk has been replaced, notifying frontend");
        if let Err(e) = app.emit(EVENT_UPDATE_PENDING, ()) {
            error!("failed to emit {EVENT_UPDATE_PENDING}: {e}");
        }
    }
}

pub fn spawn(app_handle: AppHandle) {
    let binary_path = match resolve_self_path() {
        Ok(p) => p,
        Err(e) => {
            warn!("failed to resolve self binary path, update watcher disabled: {e}");
            return;
        }
    };
    let parent = match binary_path.parent() {
        Some(p) => p.to_path_buf(),
        None => {
            warn!("binary has no parent directory, update watcher disabled");
            return;
        }
    };
    let binary_name = match binary_path.file_name() {
        Some(n) => n.to_os_string(),
        None => {
            warn!("binary has no file name, update watcher disabled");
            return;
        }
    };

    debug!(
        "starting linux update watcher on dir {} for {:?}",
        parent.display(),
        binary_name
    );

    // Run an initial check in case the binary was replaced before we started.
    if update_detected(&binary_path) {
        let fired = AtomicBool::new(false);
        emit_once(&app_handle, &fired);
        return;
    }

    let fired = Arc::new(AtomicBool::new(false));

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                error!("failed to create file watcher: {e}");
                return;
            }
        };
        if let Err(e) = watcher.watch(&parent, RecursiveMode::NonRecursive) {
            error!("failed to watch {}: {e}", parent.display());
            return;
        }

        for res in rx {
            if fired.load(Ordering::SeqCst) {
                break;
            }
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    warn!("file watcher error: {e}");
                    continue;
                }
            };
            let relevant = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            );
            if !relevant {
                continue;
            }
            let touches_binary = event
                .paths
                .iter()
                .any(|p| p.file_name() == Some(&binary_name));
            if !touches_binary {
                continue;
            }
            if update_detected(&binary_path) {
                emit_once(&app_handle, &fired);
                break;
            }
        }

        debug!("linux update watcher stopped");
    });
}
