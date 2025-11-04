// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{HttpRpcSettings, Socks5Settings, Socks5State, Socks5Status};

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::Socks5Status> for Socks5Status {
    type Error = ConversionError;

    fn try_from(value: proto::Socks5Status) -> Result<Self, Self::Error> {
        let state = proto::socks5_status::State::try_from(value.state)
            .map_err(|e| ConversionError::Decode("Socks5Status.state", e))
            .map(Socks5State::from)?;

        let socks5_settings = value
            .socks5_settings
            .map(Socks5Settings::from)
            .unwrap_or_else(|| Socks5Settings {
                listen_address: String::new(),
            });

        let http_rpc_settings = value
            .http_rpc_settings
            .map(HttpRpcSettings::from)
            .unwrap_or_else(|| HttpRpcSettings {
                listen_address: String::new(),
            });

        Ok(Self {
            state,
            socks5_settings,
            http_rpc_settings,
            error_message: value.error_message,
            active_connections: value.active_connections,
        })
    }
}

impl From<proto::socks5_status::State> for Socks5State {
    fn from(value: proto::socks5_status::State) -> Self {
        match value {
            proto::socks5_status::State::Disabled => Self::Disabled,
            proto::socks5_status::State::Idle => Self::Idle,
            proto::socks5_status::State::Connected => Self::Connected,
            proto::socks5_status::State::Error => Self::Error,
        }
    }
}

impl From<proto::Socks5Settings> for Socks5Settings {
    fn from(value: proto::Socks5Settings) -> Self {
        Self {
            listen_address: value.listen_address,
        }
    }
}

impl From<proto::HttpRpcSettings> for HttpRpcSettings {
    fn from(value: proto::HttpRpcSettings) -> Self {
        Self {
            listen_address: value.listen_address,
        }
    }
}

