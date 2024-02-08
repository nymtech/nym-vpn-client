use tauri::State;
use tracing::{debug, instrument};

use crate::{
    error::CmdError,
    features::{FeatureFlag, FEATURE_FLAGS},
    states::SharedAppState,
};

#[instrument]
#[tauri::command]
pub async fn feature_flags(
    app_state: State<'_, SharedAppState>,
) -> Result<Vec<FeatureFlag>, CmdError> {
    debug!("feature_flags");
    Ok(Vec::from(FEATURE_FLAGS))
}
