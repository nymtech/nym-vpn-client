use anyhow::Result;
use nym_vpn_proto::ConnectionStatusUpdate;
use tauri::Emitter;
use tracing::{debug, instrument, trace};

use crate::events::{StatusUpdatePayload, EVENT_STATUS_UPDATE};

#[instrument(skip_all)]
pub async fn update(app: &tauri::AppHandle, update: ConnectionStatusUpdate) -> Result<()> {
    debug!("{:?}, {}", update.kind(), update.message);
    if !update.details.is_empty() {
        trace!("details: {:?}", update.details);
    }
    app.emit(EVENT_STATUS_UPDATE, StatusUpdatePayload::from(update))
        .ok();
    Ok(())
}
