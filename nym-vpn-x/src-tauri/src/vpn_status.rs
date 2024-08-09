use crate::error::BackendError;
use crate::events::{
    ConnectionEvent, StatusUpdatePayload, EVENT_CONNECTION_STATE, EVENT_STATUS_UPDATE,
};
use crate::states::{app::ConnectionState, SharedAppState};
use anyhow::Result;
use nym_vpn_proto::ConnectionStatusUpdate;
use tauri::Manager;
use time::OffsetDateTime;
use tracing::{debug, info, instrument, trace, warn};

#[instrument(skip_all)]
pub async fn update(
    app: &tauri::AppHandle,
    status: ConnectionState,
    error: Option<BackendError>,
    connection_time: Option<OffsetDateTime>,
    failed: bool,
) -> Result<()> {
    let state = app.state::<SharedAppState>();
    trace!("vpn status: {:?}", status);

    if failed {
        app.emit_all(
            EVENT_CONNECTION_STATE,
            ConnectionEvent::Failed(error.clone()),
        )
        .ok();
    }

    let mut app_state = state.lock().await;
    let current_state = app_state.state.clone();
    app_state.state = status.clone();
    // release the lock asap
    drop(app_state);

    if current_state == status {
        return Ok(());
    }
    match status {
        ConnectionState::Connected => {
            info!("vpn status → [Connected]");
            let t = connection_time.unwrap_or_else(|| {
                info!("established connection time was not given, using current utc time");
                OffsetDateTime::now_utc()
            });
            let mut app_state = state.lock().await;
            app_state.state = status.clone();
            app_state.connection_start_time = Some(t);
            drop(app_state);
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEvent::update(
                    ConnectionState::Connected,
                    error,
                    Some(t.unix_timestamp()),
                ),
            )
            .ok();
        }
        ConnectionState::Disconnected => {
            info!("vpn status → [Disconnected]");
            let mut app_state = state.lock().await;
            app_state.state = status.clone();
            app_state.connection_start_time = None;
            drop(app_state);
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEvent::update(ConnectionState::Disconnected, error, None),
            )
            .ok();
        }
        ConnectionState::Connecting => {
            info!("vpn status → [Connecting]");
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEvent::update(ConnectionState::Connecting, error, None),
            )
            .ok();
        }
        ConnectionState::Disconnecting => {
            info!("vpn status → [Disconnecting]");
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEvent::update(ConnectionState::Disconnecting, error, None),
            )
            .ok();
        }
        ConnectionState::Unknown => {
            warn!("vpn status → [Unknown]");
            app.emit_all(
                EVENT_CONNECTION_STATE,
                ConnectionEvent::update(ConnectionState::Unknown, error, None),
            )
            .ok();
        }
    }
    Ok(())
}

#[instrument(skip_all)]
pub async fn connection_update(
    app: &tauri::AppHandle,
    update: ConnectionStatusUpdate,
) -> Result<()> {
    debug!("{:?}, {}", update.kind(), update.message);
    if !update.details.is_empty() {
        trace!("details: {:?}", update.details);
    }
    app.emit_all(EVENT_STATUS_UPDATE, StatusUpdatePayload::from(update))
        .ok();
    Ok(())
}
