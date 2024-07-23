use tauri::Manager;
use tracing::{debug, trace};
use ts_rs::TS;

use crate::{error::BackendError, grpc::client::VpndStatus, states::app::ConnectionState};

pub const EVENT_VPND_STATUS: &str = "vpnd-status";
pub const EVENT_CONNECTION_STATE: &str = "connection-state";
pub const EVENT_CONNECTION_PROGRESS: &str = "connection-progress";

#[derive(Clone, Debug, serde::Serialize)]
pub enum ConnectProgressMsg {
    Initializing,
    InitDone,
}

#[derive(Clone, serde::Serialize)]
pub struct ProgressEventPayload {
    pub key: ConnectProgressMsg,
}

#[derive(Clone, serde::Serialize, TS)]
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

#[derive(Clone, serde::Serialize, TS)]
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
