use std::fmt::Display;

use tracing::{error, warn};

use crate::vpnd::client::VpndError;

pub mod account;
pub mod cli;
pub mod daemon;
pub mod db;
pub mod diagnostic;
pub mod favorites;
pub mod fs;
pub mod gateway;
pub mod gateway_independence;
pub mod log;
pub mod network_stats;
pub mod sentry;
pub mod socks5;
pub mod sys;
pub mod tunnel;
#[cfg(windows)]
pub mod updater;
mod updater_types;
pub mod window;

pub mod tray;

/// For a setting that vpnd has already taken: if the local save failed,
/// revert vpnd so it matches what is saved, then return the save error.
/// `revert` is only awaited when `saved` is an error.
pub(crate) async fn revert_vpnd_on_save_error<T, E: Display>(
    saved: Result<T, E>,
    revert: impl Future<Output = Result<(), VpndError>>,
) -> Result<T, E> {
    if let Err(save_err) = &saved {
        warn!("failed to save the setting: {save_err}, reverting vpnd");
        if let Err(revert_err) = revert.await {
            error!("failed to revert vpnd, it may not match the saved setting: {revert_err}");
        }
    }
    saved
}
