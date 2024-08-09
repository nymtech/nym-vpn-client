use nym_vpn_proto::connection_status_update::StatusType;
use nym_vpn_proto::ConnectionStatusUpdate;
use serde::Serialize;
use std::collections::HashMap;
use tauri::Manager;
use tracing::{debug, trace};
use ts_rs::TS;

use crate::error::ErrorKey;
use crate::{error::BackendError, grpc::client::VpndStatus, states::app::ConnectionState};

pub const EVENT_VPND_STATUS: &str = "vpnd-status";
pub const EVENT_CONNECTION_STATE: &str = "connection-state";
pub const EVENT_CONNECTION_PROGRESS: &str = "connection-progress";
pub const EVENT_STATUS_UPDATE: &str = "status-update";

#[derive(Clone, Debug, Serialize)]
pub enum ConnectProgressMsg {
    Initializing,
    InitDone,
}

#[derive(Clone, Serialize)]
pub struct ProgressEventPayload {
    pub key: ConnectProgressMsg,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ConnectionEvent {
    Update(ConnectionEventPayload),
    Failed(Option<BackendError>),
}

impl ConnectionEvent {
    pub fn update(
        state: ConnectionState,
        error: Option<BackendError>,
        start_time: Option<i64>,
    ) -> Self {
        Self::Update(ConnectionEventPayload::new(state, error, start_time))
    }
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct ConnectionEventPayload {
    state: ConnectionState,
    error: Option<BackendError>,
    start_time: Option<i64>, // unix timestamp in seconds
}

impl ConnectionEventPayload {
    pub fn new(
        state: ConnectionState,
        error: Option<BackendError>,
        start_time: Option<i64>,
    ) -> Self {
        Self {
            state,
            error,
            start_time,
        }
    }
}

pub trait AppHandleEventEmitter {
    fn emit_vpnd_status(&self, status: VpndStatus);
    fn emit_connecting(&self);
    fn emit_disconnecting(&self);
    fn emit_disconnected(&self, error: Option<BackendError>);
    fn emit_connection_progress(&self, key: ConnectProgressMsg);
}

impl AppHandleEventEmitter for tauri::AppHandle {
    fn emit_vpnd_status(&self, status: VpndStatus) {
        self.emit_all(EVENT_VPND_STATUS, status).ok();
    }

    fn emit_connecting(&self) {
        debug!("sending event [{}]: Connecting", EVENT_CONNECTION_STATE);
        self.emit_all(
            EVENT_CONNECTION_STATE,
            ConnectionEvent::update(ConnectionState::Connecting, None, None),
        )
        .ok();
    }

    fn emit_disconnecting(&self) {
        debug!("sending event [{}]: Disconnecting", EVENT_CONNECTION_STATE);
        self.emit_all(
            EVENT_CONNECTION_STATE,
            ConnectionEvent::update(ConnectionState::Disconnecting, None, None),
        )
        .ok();
    }

    fn emit_disconnected(&self, error: Option<BackendError>) {
        debug!("sending event [{}]: Disconnected", EVENT_CONNECTION_STATE);
        self.emit_all(
            EVENT_CONNECTION_STATE,
            ConnectionEvent::update(ConnectionState::Disconnected, error, None),
        )
        .ok();
    }

    fn emit_connection_progress(&self, key: ConnectProgressMsg) {
        trace!("sending event [{}]: {:?}", EVENT_CONNECTION_PROGRESS, key);
        self.emit_all(EVENT_CONNECTION_PROGRESS, ProgressEventPayload { key })
            .ok();
    }
}

/// mirror of `nym_vpn_proto::connection_status_update::StatusType`
#[derive(Clone, Serialize, TS)]
#[ts(export)]
enum StatusUpdate {
    Unknown,
    EntryGatewayConnectionEstablished,
    ExitRouterConnectionEstablished,
    TunnelEndToEndConnectionEstablished,
    EntryGatewayNotRoutingMixnetMessages,
    ExitRouterNotRespondingToIpv4Ping,
    ExitRouterNotRespondingToIpv6Ping,
    ExitRouterNotRoutingIpv4Traffic,
    ExitRouterNotRoutingIpv6Traffic,
    ConnectionOkIpv4,
    ConnectionOkIpv6,
    RemainingBandwidth,
    NoBandwidth,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct StatusUpdatePayload {
    status: StatusUpdate,
    message: String,
    data: Option<HashMap<String, String>>,
    error: Option<BackendError>,
}

fn status_update_to_error(update: ConnectionStatusUpdate) -> Option<BackendError> {
    let status = update.kind();
    let error = BackendError::new_with_optional_data(
        &update.message,
        ErrorKey::from(status),
        Some(update.details),
    );
    match &status {
        StatusType::EntryGatewayNotRoutingMixnetMessages => Some(error),
        StatusType::ExitRouterNotRespondingToIpv4Ping => Some(error),
        StatusType::ExitRouterNotRoutingIpv4Traffic => Some(error),
        StatusType::NoBandwidth => Some(error),
        _ => None,
    }
}

impl From<ConnectionStatusUpdate> for StatusUpdatePayload {
    fn from(update: ConnectionStatusUpdate) -> Self {
        Self {
            status: match update.kind() {
                StatusType::EntryGatewayConnectionEstablished => {
                    StatusUpdate::EntryGatewayConnectionEstablished
                }
                StatusType::ExitRouterConnectionEstablished => {
                    StatusUpdate::ExitRouterConnectionEstablished
                }
                StatusType::TunnelEndToEndConnectionEstablished => {
                    StatusUpdate::TunnelEndToEndConnectionEstablished
                }
                StatusType::EntryGatewayNotRoutingMixnetMessages => {
                    StatusUpdate::EntryGatewayNotRoutingMixnetMessages
                }
                StatusType::ExitRouterNotRespondingToIpv4Ping => {
                    StatusUpdate::ExitRouterNotRespondingToIpv4Ping
                }
                StatusType::ExitRouterNotRespondingToIpv6Ping => {
                    StatusUpdate::ExitRouterNotRespondingToIpv6Ping
                }
                StatusType::ExitRouterNotRoutingIpv4Traffic => {
                    StatusUpdate::ExitRouterNotRoutingIpv4Traffic
                }
                StatusType::ExitRouterNotRoutingIpv6Traffic => {
                    StatusUpdate::ExitRouterNotRoutingIpv6Traffic
                }
                StatusType::ConnectionOkIpv4 => StatusUpdate::ConnectionOkIpv4,
                StatusType::ConnectionOkIpv6 => StatusUpdate::ConnectionOkIpv6,
                StatusType::RemainingBandwidth => StatusUpdate::RemainingBandwidth,
                StatusType::NoBandwidth => StatusUpdate::NoBandwidth,
                _ => StatusUpdate::Unknown,
            },
            message: update.message.clone(),
            data: Some(update.details.clone()),
            error: status_update_to_error(update),
        }
    }
}
