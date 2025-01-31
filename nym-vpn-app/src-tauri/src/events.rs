use serde::Serialize;
use tauri::Emitter;
use tracing::{debug, trace};
use ts_rs::TS;

use crate::error::{BackendError, ErrorKey};
use crate::grpc::{client::VpndStatus, events::MixnetEvent, tunnel::TunnelState};

pub const EVENT_VPND_STATUS: &str = "vpnd-status";
pub const EVENT_TUNNEL_STATE: &str = "tunnel-state";
pub const EVENT_MIXNET: &str = "mixnet-event";
pub const EVENT_CONNECTION_PROGRESS: &str = "connection-progress";

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

#[derive(Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum MixnetEventPayload {
    Event(MixnetEvent),
    Error(ErrorKey),
}

impl MixnetEventPayload {
    pub fn new(event: MixnetEvent) -> Self {
        match event {
            MixnetEvent::EntryGwDown => Self::Error(ErrorKey::EntryGwDown),
            MixnetEvent::ExitGwDownIpv4 => Self::Error(ErrorKey::ExitGwDownIpv4),
            MixnetEvent::ExitGwDownIpv6 => Self::Error(ErrorKey::ExitGwDownIpv6),
            MixnetEvent::ExitGwRoutingErrorIpv4 => Self::Error(ErrorKey::ExitGwRoutingErrorIpv4),
            MixnetEvent::ExitGwRoutingErrorIpv6 => Self::Error(ErrorKey::ExitGwRoutingErrorIpv6),
            MixnetEvent::ConnectedIpv4 => Self::Event(event),
            MixnetEvent::ConnectedIpv6 => Self::Event(event),
            MixnetEvent::NoBandwidth => Self::Error(ErrorKey::NoBandwidth),
            MixnetEvent::RemainingBandwidth(_) => Self::Event(event),
            MixnetEvent::SphinxPacketMetrics => Self::Event(event),
        }
    }
}

pub trait AppHandleEventEmitter {
    fn emit_vpnd_status(&self, status: VpndStatus);
    fn emit_tunnel_update(&self, state: &TunnelState);
    fn emit_connecting(&self);
    fn emit_disconnecting(&self);
    fn emit_disconnected(&self, error: Option<BackendError>);
    fn emit_mixnet_event(&self, event: MixnetEvent);
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

    fn emit_mixnet_event(&self, event: MixnetEvent) {
        self.emit(EVENT_MIXNET, MixnetEventPayload::new(event)).ok();
    }

    fn emit_connection_progress(&self, key: ConnectProgressMsg) {
        trace!("sending event [{}]: {:?}", EVENT_CONNECTION_PROGRESS, key);
        self.emit(EVENT_CONNECTION_PROGRESS, ProgressEventPayload { key })
            .ok();
    }
}
