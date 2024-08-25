// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::IpAddr};

use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::oneshot;

use crate::service::{ImportCredentialError, StoreAccountError};

use super::{
    VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceInfoResult,
    VpnServiceStatusResult,
};

#[allow(clippy::large_enum_variant)]
pub(crate) enum VpnServiceCommand {
    Connect(oneshot::Sender<VpnServiceConnectResult>, ConnectArgs),
    Disconnect(oneshot::Sender<VpnServiceDisconnectResult>),
    Status(oneshot::Sender<VpnServiceStatusResult>),
    Info(oneshot::Sender<VpnServiceInfoResult>),
    ImportCredential(
        oneshot::Sender<Result<Option<OffsetDateTime>, ImportCredentialError>>,
        Vec<u8>,
    ),
    StoreAccount(oneshot::Sender<Result<(), StoreAccountError>>, String),
}

impl fmt::Display for VpnServiceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VpnServiceCommand::Connect(_, args) => write!(f, "Connect {{ {args:?} }}"),
            VpnServiceCommand::Disconnect(_) => write!(f, "Disconnect"),
            VpnServiceCommand::Status(_) => write!(f, "Status"),
            VpnServiceCommand::Info(_) => write!(f, "Info"),
            VpnServiceCommand::ImportCredential(_, _) => write!(f, "ImportCredential"),
            VpnServiceCommand::StoreAccount(_, _) => write!(f, "StoreAccount"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ConnectArgs {
    pub(crate) entry: Option<EntryPoint>,
    pub(crate) exit: Option<ExitPoint>,
    pub(crate) options: ConnectOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConnectOptions {
    pub(crate) dns: Option<IpAddr>,
    pub(crate) disable_routing: bool,
    pub(crate) enable_two_hop: bool,
    pub(crate) enable_poisson_rate: bool,
    pub(crate) disable_background_cover_traffic: bool,
    pub(crate) enable_credentials_mode: bool,
    pub(crate) min_mixnode_performance: Option<u8>,
    pub(crate) min_gateway_performance: Option<u8>,
}
