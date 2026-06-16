// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

// The account controller is responsible for
// 1. checking if the account exists
// 2. register the device
// 3. request ticketbooks and top up the local credential store

mod account_readiness;
mod command_sender;
mod commands;
mod config;
mod controller;
mod deeplink;
mod error;
mod event_sender;
mod nyxd_client;
mod prefetch;
mod shared_state;
mod state_machine;
mod state_receiver;
mod storage;
mod ticketbooks;

pub(crate) use shared_state::SharedAccountState;

pub use account_readiness::{
    DEVICE_TIME_DESYNCED, DeviceRegistrationReadiness, FAIR_USAGE_DEPLETED, LocalSyncCheck,
    MAX_DEVICES_REACHED, SUMMARY_STALE_AFTER, apply_post_login_device_registration,
    classify_local_sync, device_registration_readiness, is_connect_ready_after_summary_sync,
    post_login_setup_from_classified_sync, register_device_for_prefetch_if_needed,
    register_device_if_needed, validate_active_device_time_sync, verify_time_synced,
};
pub use command_sender::AccountCommandSender;
pub use config::AccountControllerConfig;
pub use controller::AccountController;
pub use deeplink::{CreateDeeplinkParams, Deeplink, DeeplinkError, DeeplinkMnemonic, Deeplinks};
pub use error::Error;
pub use event_sender::AccountControllerEventSender;
pub use nyxd_client::NyxdClient;
pub use prefetch::{
    DEVICE_NOT_AUTHENTICATED_CODE_ID, PrefetchExternalError, PrefetchZkNymOutcome,
    app_prefetch_zk_nyms_after_fresh_summary, map_prefetch_error_for_external,
    prefetch_api_failure_suggests_stale_device_registration,
    prefetch_error_suggests_stale_device_registration, prefetch_zk_nyms, prefetch_zk_nyms_unlocked,
};
pub use state_receiver::AccountStateReceiver;
pub use storage::{CredentialStoreAccessLock, remove_files_for_account};
pub use ticketbooks::AvailableTicketbooks;
