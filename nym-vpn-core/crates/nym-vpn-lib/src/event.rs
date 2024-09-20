// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

// For sending status events that the status listener listens to(!)
// Again I'd like to re-iterate that we should create a separte trait for this instead of
// piggybacking on the error trait. Once we can depend on the latest rev of the mono repo it will
// be possible to make this change.
#[derive(Debug, thiserror::Error)]
pub enum WgTunnelErrorEvent {
    #[error("failed to create dir {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to start wireguard monitor: {0}")]
    WireguardMonitor(#[source] talpid_wireguard::Error),

    #[error("failed to send shutdown message to wireguard tunnel")]
    SendWireguardShutdown,
}
