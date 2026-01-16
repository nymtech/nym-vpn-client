// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("WebSocket stream closed")]
    WsStreamClosed,

    #[error("No API URLs in the given network")]
    MissingApiUrl,

    #[error("Failed to build API client : {0}")]
    BuildApiClient(#[from] nym_vpn_api_client::error::VpnApiClientError),

    #[error("WS error : {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::error::Error),
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
