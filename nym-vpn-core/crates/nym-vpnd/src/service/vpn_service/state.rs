// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
};

use nym_vpn_lib::{NodeIdentity, Recipient};

use crate::service::ConnectionFailedError;

// The current state of the VPN service
#[derive(Debug, Clone)]
pub(crate) enum VpnState {
    NotConnected,
    Connecting,
    Connected(Box<VpnConnectedStateDetails>),
    Disconnecting,
    ConnectionFailed(ConnectionFailedError),
}

impl fmt::Display for VpnState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnState::NotConnected => write!(f, "NotConnected"),
            VpnState::Connecting => write!(f, "Connecting"),
            VpnState::Connected(details) => write!(f, "Connected({})", details),
            VpnState::Disconnecting => write!(f, "Disconnecting"),
            VpnState::ConnectionFailed(reason) => write!(f, "ConnectionFailed({})", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MixConnectedStateDetails {
    pub(crate) nym_address: Recipient,
    pub(crate) exit_ipr: Recipient,
    pub(crate) ipv4: Ipv4Addr,
    pub(crate) ipv6: Ipv6Addr,
}

#[derive(Debug, Clone)]
pub(crate) struct WgConnectedStateDetails {
    pub(crate) entry_ipv4: Ipv4Addr,
    pub(crate) exit_ipv4: Ipv4Addr,
}

#[derive(Debug, Clone)]
pub(crate) enum ConnectedStateDetails {
    Mix(Box<MixConnectedStateDetails>),
    Wg(WgConnectedStateDetails),
}

impl fmt::Display for ConnectedStateDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mix(details) => {
                write!(
                    f,
                    "nym_address: {}, exit_ipr: {}, ipv4: {}, ipv6: {}",
                    details.nym_address, details.exit_ipr, details.ipv4, details.ipv6
                )
            }
            Self::Wg(details) => {
                write!(
                    f,
                    "entry_ipv4: {}, exit_ipv4: {}",
                    details.entry_ipv4, details.exit_ipv4
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VpnConnectedStateDetails {
    pub(crate) entry_gateway: NodeIdentity,
    pub(crate) exit_gateway: NodeIdentity,
    pub(crate) specific_details: ConnectedStateDetails,
    pub(crate) since: time::OffsetDateTime,
}

impl fmt::Display for VpnConnectedStateDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entry_gateway: {}, exit_gateway: {}, specific_details: {}, since: {}",
            self.entry_gateway, self.exit_gateway, self.specific_details, self.since
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConnectedResultDetails {
    pub(crate) entry_gateway: NodeIdentity,
    pub(crate) exit_gateway: NodeIdentity,
    pub(crate) specific_details: ConnectedStateDetails,
    pub(crate) since: time::OffsetDateTime,
}

impl fmt::Display for ConnectedResultDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entry_gateway: {}, exit_gateway: {}, specific_details: {}, since: {}",
            self.entry_gateway, self.exit_gateway, self.specific_details, self.since
        )
    }
}
