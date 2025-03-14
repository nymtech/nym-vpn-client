// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types shared between nym-vpn-lib and other crates in the workspace.

mod account;
mod connection_data;
mod tunnel_event;
mod tunnel_state;

pub use account::{
    forget_account::ForgetAccountError,
    register_device::RegisterDeviceError,
    request_zknym::{RequestZkNymError, RequestZkNymErrorReason, RequestZkNymSuccess},
    store_account::StoreAccountError,
    sync_account::SyncAccountError,
    sync_device::SyncDeviceError,
    AccountCommandError, VpnApiErrorResponse,
};
pub use connection_data::{
    ConnectionData, Gateway, MixnetConnectionData, NymAddress, TunnelConnectionData,
    WireguardConnectionData, WireguardNode,
};
pub use tunnel_event::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
    TunnelEvent,
};
pub use tunnel_state::{
    ActionAfterDisconnect, ClientErrorReason, ErrorStateReason, TunnelState, TunnelType,
};
