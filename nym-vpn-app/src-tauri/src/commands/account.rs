use tauri::State;
use tracing::{error, info, instrument, warn};

use crate::state::SharedAppState;
use crate::vpnd::account::{AccountState, AccountSummary};
use crate::vpnd::account::{AutologinResponse, StoredAccountMode};
use crate::vpnd::account_links::AccountLinks;
use crate::vpnd::deeplink::DeeplinkKind;
use crate::vpnd::tunnel::TunnelState;
use crate::{error::BackendError, vpnd::client::VpndClient};

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_state(
    app: State<'_, SharedAppState>,
) -> Result<AccountState, BackendError> {
    let state = app.lock().await;
    Ok(state.account_state.clone())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_account(
    mnemonic: Option<String>,
    signature: Option<String>,
    vpnd: State<'_, VpndClient>,
    app_state: State<'_, SharedAppState>,
) -> Result<(), BackendError> {
    let state = app_state.lock().await;
    if !matches!(state.tunnel, TunnelState::Disconnected) {
        return Err(BackendError::internal(
            &format!("cannot add account from state {}", state.tunnel),
            None,
        ));
    };
    drop(state);

    vpnd.store_account(mnemonic, signature)
        .await
        .map_err(|e| {
            error!("failed to add account: {}", e);
            e.into()
        })
        .inspect(|_| {
            info!("account added successfully");
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn forget_account(
    vpnd: State<'_, VpndClient>,
    app_state: State<'_, SharedAppState>,
) -> Result<(), BackendError> {
    let state = app_state.lock().await;
    if !matches!(state.tunnel, TunnelState::Disconnected) {
        return Err(BackendError::internal(
            &format!("cannot forget account from state {}", state.tunnel),
            None,
        ));
    };
    drop(state);

    vpnd.forget_account()
        .await
        .map_err(|e| {
            error!("failed to forget account: {}", e);
            e.into()
        })
        .inspect(|_| {
            info!("account removed successfully");
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn is_account_stored(vpnd: State<'_, VpndClient>) -> Result<bool, BackendError> {
    vpnd.is_account_stored()
        .await
        .map_err(|e| {
            error!("failed to check stored account: {e}");
            e.into()
        })
        .inspect(|stored| {
            info!("account stored: {stored}");
        })
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn account_links(
    vpnd: State<'_, VpndClient>,
    locale: String,
) -> Result<AccountLinks, BackendError> {
    vpnd.account_links(&locale).await.map_err(|e| {
        error!("failed to get account link: {e}");
        e.into()
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_id(vpnd: State<'_, VpndClient>) -> Result<Option<String>, BackendError> {
    vpnd.account_id()
        .await
        .map_err(|e| {
            warn!("failed to get account id: {e}");
            e.into()
        })
        .inspect(|id| {
            info!("account id: {:?}", id);
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_canonical_account_id(
    vpnd: State<'_, VpndClient>,
) -> Result<Option<String>, BackendError> {
    vpnd.canonical_account_id()
        .await
        .map_err(|e| {
            warn!("failed to get canonical account id: {e}");
            e.into()
        })
        .inspect(|id| {
            info!("canonical account id: {:?}", id);
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_device_id(vpnd: State<'_, VpndClient>) -> Result<Option<String>, BackendError> {
    vpnd.device_id()
        .await
        .map_err(|e| {
            warn!("failed to get device id: {e}");
            e.into()
        })
        .inspect(|id| {
            if let Some(id) = id {
                info!("device id: {id}");
            } else {
                info!("no device id");
            }
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_deep_link(
    vpnd: State<'_, VpndClient>,
    locale: String,
    kind: nym_vpn_lib_types::DeeplinkKind,
) -> Result<Option<String>, BackendError> {
    vpnd.get_deep_link(locale, kind).await.map_err(|e| {
        error!("failed to get deep link: {e}");
        e.into()
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_autologin_deeplink(
    vpnd: State<'_, VpndClient>,
    locale: String,
    kind: DeeplinkKind,
) -> Result<Option<AutologinResponse>, BackendError> {
    vpnd.get_autologin_deeplink(locale, kind)
        .await
        .map_err(|e| {
            error!("failed to get autologin deeplink: {e}");
            e.into()
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn store_deeplink_account(
    vpnd: State<'_, VpndClient>,
    callback_url: String,
) -> Result<(), BackendError> {
    vpnd.store_deeplink_account(callback_url)
        .await
        .map_err(|e| {
            error!("failed to store deeplink account: {e}");
            e.into()
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_mode(
    vpnd: State<'_, VpndClient>,
) -> Result<Option<StoredAccountMode>, BackendError> {
    vpnd.account_mode().await.map_err(|e| {
        warn!("failed to get account mode: {e}");
        e.into()
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_summary(
    vpnd: State<'_, VpndClient>,
) -> Result<Option<AccountSummary>, BackendError> {
    vpnd.get_account_summary().await.map_err(|e| {
        error!("failed to get account summary: {e}");
        e.into()
    })
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn refresh_account_state(
    force: bool,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    vpnd.refresh_account_state(force).await.map_err(|e| {
        error!("failed to refresh account state: {e}");
        e.into()
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn handle_subscription_payment(vpnd: State<'_, VpndClient>) -> Result<(), BackendError> {
    vpnd.handle_subscription_payment().await.map_err(|e| {
        error!("failed to handle subscription payment: {e}");
        e.into()
    })
}
