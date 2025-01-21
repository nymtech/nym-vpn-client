use nym_vpn_proto::connection_status_update::StatusType;
use nym_vpn_proto::ConnectionStatusUpdate;
use serde::Serialize;
use std::collections::HashMap;
use tauri::Emitter;
use tracing::{debug, trace};
use ts_rs::TS;

use crate::error::ErrorKey;
use crate::grpc::tunnel::TunnelState;
use crate::{error::BackendError, grpc::client::VpndStatus};

pub const EVENT_VPND_STATUS: &str = "vpnd-status";
pub const EVENT_TUNNEL_STATE: &str = "tunnel-state";
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
pub struct TunnelStateEvent {
    state: TunnelState,
    // TODO not sure if this is still needed
    error: Option<BackendError>,
}

impl TunnelStateEvent {
    pub fn new(state: &TunnelState, error: Option<BackendError>) -> Self {
        Self {
            state: state.clone(),
            error,
        }
    }
}

pub trait AppHandleEventEmitter {
    fn emit_vpnd_status(&self, status: VpndStatus);
    fn emit_tunnel_update(&self, state: &TunnelState);
    fn emit_connecting(&self);
    fn emit_disconnecting(&self);
    fn emit_disconnected(&self, error: Option<BackendError>);
    fn emit_connection_progress(&self, key: ConnectProgressMsg);
}

impl AppHandleEventEmitter for tauri::AppHandle {
    fn emit_vpnd_status(&self, status: VpndStatus) {
        self.emit(EVENT_VPND_STATUS, status).ok();
    }

    fn emit_tunnel_update(&self, state: &TunnelState) {
        debug!("sending event [{}]: {}", EVENT_TUNNEL_STATE, state);
        self.emit(EVENT_TUNNEL_STATE, TunnelStateEvent::new(state, None))
            .ok();
    }

    fn emit_connecting(&self) {
        debug!("sending event [{}]: Connecting", EVENT_TUNNEL_STATE);
        self.emit(
            EVENT_TUNNEL_STATE,
            TunnelStateEvent::new(&TunnelState::Connecting(None), None),
        )
        .ok();
    }

    fn emit_disconnecting(&self) {
        debug!("sending event [{}]: Disconnecting", EVENT_TUNNEL_STATE);
        self.emit(
            EVENT_TUNNEL_STATE,
            TunnelStateEvent::new(&TunnelState::Disconnecting(None), None),
        )
        .ok();
    }

    fn emit_disconnected(&self, error: Option<BackendError>) {
        debug!("sending event [{}]: Disconnected", EVENT_TUNNEL_STATE);
        self.emit(
            EVENT_TUNNEL_STATE,
            TunnelStateEvent::new(&TunnelState::Disconnected, error),
        )
        .ok();
    }

    fn emit_connection_progress(&self, key: ConnectProgressMsg) {
        trace!("sending event [{}]: {:?}", EVENT_CONNECTION_PROGRESS, key);
        self.emit(EVENT_CONNECTION_PROGRESS, ProgressEventPayload { key })
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
    MixnetBandwidthRate,
    NoBandwidth,
    WgTunnelError,
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
    let error = BackendError::with_optional_data(
        &update.message,
        ErrorKey::from(status),
        Some(update.details),
    );
    match &status {
        StatusType::EntryGatewayNotRoutingMixnetMessages => Some(error),
        StatusType::ExitRouterNotRespondingToIpv4Ping => Some(error),
        StatusType::ExitRouterNotRoutingIpv4Traffic => Some(error),
        StatusType::ExitRouterNotRespondingToIpv6Ping => Some(error),
        StatusType::ExitRouterNotRoutingIpv6Traffic => Some(error),
        StatusType::NoBandwidth => Some(error),
        StatusType::WgTunnelError => Some(error),
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
                StatusType::WgTunnelError => StatusUpdate::WgTunnelError,
                StatusType::MixnetBandwidthRate => StatusUpdate::MixnetBandwidthRate,
                _ => StatusUpdate::Unknown, // Unspecified & Unknown
            },
            message: update.message.clone(),
            data: Some(update.details.clone()),
            error: status_update_to_error(update),
        }
    }
}
