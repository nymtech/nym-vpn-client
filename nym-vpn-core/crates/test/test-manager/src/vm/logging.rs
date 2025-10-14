// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

pub async fn forward_logs<T: AsyncRead + Unpin>(prefix: &str, stdio: T, level: log::Level) {
    let reader = BufReader::new(stdio);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::log!(level, "{prefix}{line}");
    }
}
