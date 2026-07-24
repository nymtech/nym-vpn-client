// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

pub const APP_DIR: &str = "nym-vpn-app";

pub async fn app_data_dir() -> Option<PathBuf> {
    let app_dir = dirs::data_dir().map(|mut d| {
        d.push(APP_DIR);
        d
    });
    if let Some(d) = &app_dir
        && tokio::fs::create_dir_all(d).await.is_err()
    {
        return None;
    }
    app_dir
}
