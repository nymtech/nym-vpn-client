use serde::Serialize;
use tauri::Emitter;
use tracing::{debug, trace};
use ts_rs::TS;

use crate::error::{BackendError, ErrorKey};
use crate::vpnd::account::AccountState;
use crate::vpnd::config::VpndConfig;
use crate::vpnd::tunnel::ConnectingState;
use crate::vpnd::{
    client::VpndStatus,
    events::{ConflictDetected, DiagnosticsSuggestedReason, MixnetEvent},
    tunnel::TunnelState,
};

pub const EVENT_VPND_STATUS: &str = "vpnd-status";
pub const EVENT_TUNNEL_STATE: &str = "tunnel-state";
pub const EVENT_ACCOUNT_STATE: &str = "account-state";
pub const EVENT_VPN_CONFIG: &str = "vpn-config";
pub const EVENT_MIXNET: &str = "mixnet-event";
pub const EVENT_DIAGNOSTICS_SUGGESTED: &str = "diagnostics-suggested";
pub const EVENT_CONFLICT_DETECTED: &str = "conflict-detected";
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub const EVENT_UPDATE_PENDING: &str = "update-pending";

#[derive(Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
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
#[ts(export, export_to = "tauri.ts")]
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
            MixnetEvent::NoBandwidth => Self::Error(ErrorKey::MixnetNoBandwidth),
            MixnetEvent::RemainingBandwidth(_) => Self::Event(event),
            MixnetEvent::SphinxPacketMetrics => Self::Event(event),
        }
    }
}

pub trait AppHandleEventEmitter {
    fn emit_vpnd_status(&self, status: VpndStatus);
    fn emit_vpnd_config(&self, config: VpndConfig);
    fn emit_tunnel_update(&self, state: &TunnelState);
    fn emit_connecting(&self);
    fn emit_disconnecting(&self);
    fn emit_disconnected(&self, error: Option<BackendError>);
    fn emit_mixnet_event(&self, event: MixnetEvent);
    fn emit_account_state_update(&self, state: &AccountState);
    fn emit_diagnostics_suggested(&self, reason: DiagnosticsSuggestedReason);
    fn emit_conflict_detected(&self, conflict: ConflictDetected);
}

impl AppHandleEventEmitter for tauri::AppHandle {
    fn emit_vpnd_status(&self, status: VpndStatus) {
        self.emit(EVENT_VPND_STATUS, status).ok();
    }

    fn emit_vpnd_config(&self, config: VpndConfig) {
        self.emit(EVENT_VPN_CONFIG, config).ok();
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
            TunnelStateEvent::new(&TunnelState::Connecting(ConnectingState::default()), None),
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

    fn emit_account_state_update(&self, state: &AccountState) {
        trace!(
            "sending account state event [{}]: {}",
            EVENT_ACCOUNT_STATE,
            state.as_ref()
        );
        self.emit(EVENT_ACCOUNT_STATE, state).ok();
    }

    fn emit_diagnostics_suggested(&self, reason: DiagnosticsSuggestedReason) {
        debug!(
            "sending event [{}]: {:?}",
            EVENT_DIAGNOSTICS_SUGGESTED, reason
        );
        self.emit(EVENT_DIAGNOSTICS_SUGGESTED, reason).ok();
    }

    fn emit_conflict_detected(&self, conflict: ConflictDetected) {
        debug!(
            "sending event [{}]: {:?}",
            EVENT_CONFLICT_DETECTED, conflict
        );
        self.emit(EVENT_CONFLICT_DETECTED, conflict).ok();
    }
}
