// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use futures::channel::oneshot::Receiver as OneshotReceiver;

use crate::service::ConnectionFailedError;

use super::{ConnectedResultDetails, VpnConnectedStateDetails, VpnState};

#[derive(Debug)]
pub(crate) enum VpnServiceConnectResult {
    Success(VpnServiceConnectHandle),
    Fail(String),
}

impl VpnServiceConnectResult {
    pub(crate) fn is_success(&self) -> bool {
        matches!(self, VpnServiceConnectResult::Success(_))
    }
}

#[derive(Debug)]
pub(crate) struct VpnServiceConnectHandle {
    pub(crate) listener_vpn_status_rx: nym_vpn_lib::StatusReceiver,
    #[allow(unused)]
    pub(crate) listener_vpn_exit_rx: OneshotReceiver<nym_vpn_lib::NymVpnExitStatusMessage>,
}

#[derive(Debug)]
pub(crate) enum VpnServiceDisconnectResult {
    Success,
    NotRunning,
    #[allow(unused)]
    Fail(String),
}

impl VpnServiceDisconnectResult {
    pub(crate) fn is_success(&self) -> bool {
        matches!(self, VpnServiceDisconnectResult::Success)
    }
}

// Respond with the current state of the VPN service. This is currently almost the same as VpnState,
// but it's conceptually not the same thing, so we keep them separate.
#[derive(Clone, Debug)]
pub(crate) enum VpnServiceStatusResult {
    NotConnected,
    Connecting,
    Connected(Box<ConnectedResultDetails>),
    Disconnecting,
    ConnectionFailed(ConnectionFailedError),
}

#[derive(Clone, Debug)]
pub(crate) struct VpnServiceInfoResult {
    pub(crate) version: String,
    pub(crate) build_timestamp: Option<time::OffsetDateTime>,
    pub(crate) triple: String,
    pub(crate) git_commit: String,
    pub(crate) network_name: String,
    pub(crate) endpoints: Vec<nym_vpn_lib::nym_config::defaults::ValidatorDetails>,
    pub(crate) nym_vpn_api_url: Option<String>,
}

impl fmt::Display for VpnServiceStatusResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceStatusResult::NotConnected => write!(f, "NotConnected"),
            VpnServiceStatusResult::Connecting => write!(f, "Connecting"),
            VpnServiceStatusResult::Connected(details) => write!(f, "Connected({})", details),
            VpnServiceStatusResult::Disconnecting => write!(f, "Disconnecting"),
            VpnServiceStatusResult::ConnectionFailed(reason) => {
                write!(f, "ConnectionFailed({})", reason)
            }
        }
    }
}

impl From<VpnState> for VpnServiceStatusResult {
    fn from(state: VpnState) -> Self {
        match state {
            VpnState::NotConnected => VpnServiceStatusResult::NotConnected,
            VpnState::Connecting => VpnServiceStatusResult::Connecting,
            VpnState::Connected(details) => VpnServiceStatusResult::Connected(details.into()),
            VpnState::Disconnecting => VpnServiceStatusResult::Disconnecting,
            VpnState::ConnectionFailed(reason) => VpnServiceStatusResult::ConnectionFailed(reason),
        }
    }
}

impl From<VpnConnectedStateDetails> for ConnectedResultDetails {
    fn from(details: VpnConnectedStateDetails) -> Self {
        ConnectedResultDetails {
            entry_gateway: details.entry_gateway,
            exit_gateway: details.exit_gateway,
            specific_details: details.specific_details,
            since: details.since,
        }
    }
}

impl From<Box<VpnConnectedStateDetails>> for Box<ConnectedResultDetails> {
    fn from(details: Box<VpnConnectedStateDetails>) -> Self {
        Box::new((*details).into())
    }
}
