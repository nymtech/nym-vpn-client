use tauri::Manager;
use time::OffsetDateTime;
use tracing::debug;

use crate::states::app::ConnectionState;

pub const EVENT_CONNECTION_STATE: &str = "connection-state";
pub const EVENT_CONNECTION_PROGRESS: &str = "connection-progress";

#[derive(Clone, serde::Serialize)]
pub enum ConnectProgressMsg {
    Initializing,
    InitDone,
}

#[derive(Clone, serde::Serialize)]
pub struct ProgressEventPayload {
    pub key: ConnectProgressMsg,
}

#[derive(Clone, serde::Serialize)]
pub struct ConnectionEventPayload {
    state: ConnectionState,
    error: Option<String>,
    start_time: Option<i64>, // unix timestamp in seconds
}

impl ConnectionEventPayload {
    pub fn new(state: ConnectionState, error: Option<String>, start_time: Option<i64>) -> Self {
        Self {
            state,
            error,
            start_time,
        }
    }
}

pub trait AppHandleEventEmitter {
    fn emit_disconnected(&self, error: Option<String>);
    fn emit_connected(&self, now: OffsetDateTime, gateway: String);
}

impl AppHandleEventEmitter for tauri::AppHandle {
    fn emit_disconnected(&self, error: Option<String>) {
        debug!("sending event [{}]: Disconnected", EVENT_CONNECTION_STATE);
        self.emit_all(
            EVENT_CONNECTION_STATE,
            ConnectionEventPayload::new(ConnectionState::Disconnected, error, None),
        )
        .ok();
    }

    fn emit_connected(&self, now: OffsetDateTime, _gateway: String) {
        debug!("sending event [{}]: Connected", EVENT_CONNECTION_STATE);
        self.emit_all(
            EVENT_CONNECTION_STATE,
            ConnectionEventPayload::new(
                // TODO: once the frontend can handle it, send the connection info as part of the
                // connection state
                //ConnectionState::Connected(ConnectionInfo { gateway }),
                ConnectionState::Connected,
                None,
                Some(now.unix_timestamp()),
            ),
        )
        .ok();
    }
}
