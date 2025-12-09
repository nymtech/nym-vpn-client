// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    EnableSocks5Request, ExitPoint, HttpRpcSettings, Socks5Settings, Socks5State, Socks5Status,
};
use std::net::SocketAddr;

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::Socks5Status> for Socks5Status {
    type Error = ConversionError;

    fn try_from(value: proto::Socks5Status) -> Result<Self, Self::Error> {
        let state = proto::socks5_status::State::try_from(value.state)
            .map_err(|e| ConversionError::Decode("Socks5Status.state", e))
            .map(Socks5State::from)?;

        let socks5_settings: Socks5Settings = value
            .socks5_settings
            .ok_or(ConversionError::NoValueSet(
                "EnableSocks5Request.socks5_settings",
            ))?
            .try_into()?;

        let http_rpc_settings: HttpRpcSettings = value
            .http_rpc_settings
            .ok_or(ConversionError::NoValueSet(
                "EnableSocks5Request.http_rpc_settings",
            ))?
            .try_into()?;

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

impl TryFrom<proto::EnableSocks5Request> for EnableSocks5Request {
    type Error = ConversionError;

    fn try_from(value: proto::EnableSocks5Request) -> Result<Self, Self::Error> {
        let socks5_settings: Socks5Settings = value
            .socks5_settings
            .ok_or(ConversionError::NoValueSet(
                "EnableSocks5Request.socks5_settings",
            ))?
            .try_into()?;

        let http_rpc_settings: HttpRpcSettings = value
            .http_rpc_settings
            .ok_or(ConversionError::NoValueSet(
                "EnableSocks5Request.http_rpc_settings",
            ))?
            .try_into()?;

        let exit_point = value
            .exit
            .map(ExitPoint::try_from)
            .transpose()?
            .ok_or(ConversionError::NoValueSet("VpnServiceConfig.exit_point"))?;

        Ok(Self {
            socks5_settings,
            http_rpc_settings,
            exit_point,
        })
    }
}

impl TryFrom<proto::Socks5Settings> for Socks5Settings {
    type Error = ConversionError;

    fn try_from(value: proto::Socks5Settings) -> Result<Self, Self::Error> {
        let listen_address: Option<SocketAddr> = if !value.listen_address.is_empty() {
            Some(
                value
                    .listen_address
                    .parse::<SocketAddr>()
                    .map_err(|e| ConversionError::ParseAddr("Socks5Settings.listen_address", e))?,
            )
        } else {
            None
        };

        Ok(Self { listen_address })
    }
}

impl TryFrom<proto::HttpRpcSettings> for HttpRpcSettings {
    type Error = ConversionError;

    fn try_from(value: proto::HttpRpcSettings) -> Result<Self, Self::Error> {
        let listen_address: Option<SocketAddr> = if !value.listen_address.is_empty() {
            Some(
                value
                    .listen_address
                    .parse::<SocketAddr>()
                    .map_err(|e| ConversionError::ParseAddr("HttpRpcSettings.listen_address", e))?,
            )
        } else {
            None
        };

        Ok(Self { listen_address })
    }
}

// Reverse conversions: lib types -> proto types

impl From<Socks5State> for proto::socks5_status::State {
    fn from(value: Socks5State) -> Self {
        match value {
            Socks5State::Disabled => Self::Disabled,
            Socks5State::Idle => Self::Idle,
            Socks5State::Connected => Self::Connected,
            Socks5State::Error => Self::Error,
        }
    }
}

impl From<Socks5Settings> for proto::Socks5Settings {
    fn from(value: Socks5Settings) -> Self {
        let listen_address = match value.listen_address {
            Some(addr) => addr.to_string(),
            None => String::new(),
        };

        Self { listen_address }
    }
}

impl From<HttpRpcSettings> for proto::HttpRpcSettings {
    fn from(value: HttpRpcSettings) -> Self {
        let listen_address = match value.listen_address {
            Some(addr) => addr.to_string(),
            None => String::new(),
        };

        Self { listen_address }
    }
}

impl From<Socks5Status> for proto::Socks5Status {
    fn from(value: Socks5Status) -> Self {
        Self {
            state: proto::socks5_status::State::from(value.state) as i32,
            socks5_settings: Some(proto::Socks5Settings::from(value.socks5_settings)),
            http_rpc_settings: Some(proto::HttpRpcSettings::from(value.http_rpc_settings)),
            error_message: value.error_message,
            active_connections: value.active_connections,
        }
    }
}
