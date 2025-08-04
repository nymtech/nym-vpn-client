// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_api_client::{
    response::{
        NymVpnAccountResponse, NymVpnAccountStatusResponse, NymVpnAccountSummaryDevices,
        NymVpnAccountSummaryFairUsage, NymVpnAccountSummaryResponse,
        NymVpnAccountSummarySubscription, NymVpnDeviceStatus, NymVpnSubscriptionStatus,
    },
    types::{Device, VpnApiAccount},
};
use nym_vpn_lib_types::{RegisterDeviceError, RequestZkNymError, RequestZkNymSuccess};
use serde::Serialize;
use tokio::sync::mpsc;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    AccountControllerConfig,
    storage::{AccountStorageOp, VpnCredentialStorage},
    vpn_api_client::AccountControllerVpnApiClient,
};

#[derive(Zeroize, ZeroizeOnDrop)]
pub(crate) struct SharedAccountState {
    // SW add tunnel state? Yes, to remove some conditions on forget account and reset device id
    #[zeroize(skip)]
    pub connectivity_handle: ConnectivityHandle,

    #[zeroize(skip)]
    pub config: AccountControllerConfig,

    #[zeroize(skip)]
    pub(crate) credential_storage: VpnCredentialStorage,

    #[zeroize(skip)]
    pub(crate) vpn_api_client: AccountControllerVpnApiClient,

    pub(crate) vpn_api_account: Option<VpnApiAccount>,

    #[zeroize(skip)]
    pub(crate) device: Option<Device>,

    #[zeroize(skip)]
    pub(crate) storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToRegisterDevice {
    Ready,
    InProgress,
    NoMnemonicStored,
    AccountNotSynced,
    AccountNotRegistered,
    AccountNotActive,
    NoActiveSubscription,
    DeviceStateNotSynced,
    DeviceAlreadyRegistered,
    MaxDevicesReached(u64),
}

impl fmt::Display for ReadyToRegisterDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToRegisterDevice::Ready => write!(f, "ready to register device"),
            ReadyToRegisterDevice::InProgress => write!(f, "device registration in progress"),
            ReadyToRegisterDevice::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToRegisterDevice::AccountNotSynced => write!(f, "account not synced"),
            ReadyToRegisterDevice::AccountNotRegistered => write!(f, "account not registered"),
            ReadyToRegisterDevice::AccountNotActive => write!(f, "account not active"),
            ReadyToRegisterDevice::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToRegisterDevice::DeviceStateNotSynced => write!(f, "device state not synced"),
            ReadyToRegisterDevice::DeviceAlreadyRegistered => {
                write!(f, "device already registered")
            }
            ReadyToRegisterDevice::MaxDevicesReached(max) => {
                write!(f, "maximum number of devices reached: {max}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToRequestZkNym {
    Ready,
    InProgress,
    NoMnemonicStored,
    AccountNotSynced,
    AccountNotRegistered,
    AccountNotActive,
    DeviceNotSynced,
    NoActiveSubscription,
    DeviceNotRegistered,
    DeviceNotActive,
}

impl fmt::Display for ReadyToRequestZkNym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToRequestZkNym::Ready => write!(f, "ready to request zk-nym"),
            ReadyToRequestZkNym::InProgress => write!(f, "zk-nym request in progress"),
            ReadyToRequestZkNym::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToRequestZkNym::AccountNotSynced => write!(f, "account not synced"),
            ReadyToRequestZkNym::AccountNotRegistered => write!(f, "account not registered"),
            ReadyToRequestZkNym::AccountNotActive => write!(f, "account not active"),
            ReadyToRequestZkNym::DeviceNotSynced => write!(f, "device not synced"),
            ReadyToRequestZkNym::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToRequestZkNym::DeviceNotRegistered => write!(f, "device not registered"),
            ReadyToRequestZkNym::DeviceNotActive => write!(f, "device not active"),
        }
    }
}

impl SharedAccountState {
    pub(crate) async fn new(
        connectivity_handle: ConnectivityHandle,
        config: AccountControllerConfig,
        credential_storage: VpnCredentialStorage,
        vpn_api_client: AccountControllerVpnApiClient,
        vpn_api_account: Option<VpnApiAccount>,
        device: Option<Device>,
        storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
    ) -> Self {
        SharedAccountState {
            connectivity_handle,
            config,
            credential_storage,
            vpn_api_client,
            vpn_api_account,
            device,
            storage_op_sender,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AccountStateSummary {
    // The locally stored recovery phrase that is deeply tied to the account
    pub mnemonic: Option<MnemonicState>,

    // If the account is active on nym-vpn-api
    pub account_registered: Option<AccountRegistered>,

    // The summary of the account on nym-vpn-api
    pub account_summary: Option<AccountSummary>,

    // The state of the device as reported by nym-vpn-api
    pub device: Option<DeviceState>,

    // The result of the latest registration attempt, if any
    pub register_device_result: Option<RegisterDeviceResult>,

    // The result of the latest zk-nym request, if any
    pub request_zk_nym_result: Option<RequestZkNymResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AccountRegistered {
    NotRegistered,
    Registered,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountSummary {
    pub account: AccountState,
    pub subscription: SubscriptionState,
    pub device_summary: DeviceSummary,
    pub fair_usage: FairUsage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MnemonicState {
    // The recovery phrase is not stored locally, or at least not confirmed to be stored
    NotStored,

    // The recovery phrase is stored locally
    Stored { id: String },
}

impl MnemonicState {
    pub fn is_stored(&self) -> bool {
        matches!(self, MnemonicState::Stored { .. })
    }

    pub fn id(&self) -> Option<String> {
        match self {
            MnemonicState::Stored { id } => Some(id.clone()),
            MnemonicState::NotStored => None,
        }
    }
}

impl fmt::Display for MnemonicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MnemonicState::NotStored => write!(f, "not stored"),
            MnemonicState::Stored { id } => write!(f, "stored with id {id}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountState {
    // The account is registered but not active
    Inactive,

    // The account is registered and active
    Active,

    // The account is marked for deletion
    DeleteMe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceSummary {
    pub active: u64,
    pub max: u64,
    pub remaining: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FairUsage {
    pub used_gb: u64,
    pub limit_gb: u64,
    pub resets_on_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionState {
    // There is no active subscription
    NotActive,

    // The subscription is pending
    Pending,

    // The subscription is complete
    Complete,

    // The subscription is active
    Active,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    // The device is not registered on the remote server
    NotRegistered,

    // The device is registered but not active
    Inactive,

    // The device is registered and active
    Active,

    // The device is marked for deletion
    DeleteMe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterDeviceResult {
    // The device registration is in progress
    InProgress,

    // The device registration was successful
    Success,

    // The device registration failed
    Failed(RegisterDeviceError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestZkNymResult {
    // The zk-nym request is in progress
    InProgress,

    // The the last zk-nym request finished
    Done {
        successes: Vec<RequestZkNymSuccess>,
        failures: Vec<RequestZkNymError>,
    },

    // The last zk-nym request failed before any requests were made
    Error(RequestZkNymError),
}

impl From<Vec<Result<RequestZkNymSuccess, RequestZkNymError>>> for RequestZkNymResult {
    fn from(results: Vec<Result<RequestZkNymSuccess, RequestZkNymError>>) -> Self {
        let (successes, failures): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

        let successes = successes.into_iter().map(Result::unwrap).collect();
        let failures = failures.into_iter().map(Result::unwrap_err).collect();

        RequestZkNymResult::Done {
            successes,
            failures,
        }
    }
}

impl From<RequestZkNymError> for RequestZkNymResult {
    fn from(err: RequestZkNymError) -> Self {
        RequestZkNymResult::Error(err)
    }
}

impl fmt::Display for AccountStateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountState {{ mnemonic: {}, account_registered: {}, account_summary: {}, device: {} }}",
            debug_or_unknown(self.mnemonic.as_ref()),
            debug_or_unknown(self.account_registered.as_ref()),
            debug_or_unknown(self.account_summary.as_ref()),
            debug_or_unknown(self.device.as_ref()),
        )
    }
}

fn debug_or_unknown(state: Option<&impl fmt::Debug>) -> String {
    state
        .map(|s| format!("{s:?}"))
        .unwrap_or_else(|| "Unknown".to_string())
}

impl From<NymVpnAccountResponse> for AccountState {
    fn from(account: NymVpnAccountResponse) -> Self {
        match account.status {
            NymVpnAccountStatusResponse::Active => AccountState::Active,
            NymVpnAccountStatusResponse::Inactive => AccountState::Inactive,
            NymVpnAccountStatusResponse::DeleteMe => AccountState::DeleteMe,
        }
    }
}

impl From<NymVpnAccountSummarySubscription> for SubscriptionState {
    fn from(subscription: NymVpnAccountSummarySubscription) -> Self {
        if subscription.is_active {
            SubscriptionState::Active
        } else if let Some(subscription) = subscription.active {
            match subscription.status {
                NymVpnSubscriptionStatus::Pending => SubscriptionState::Pending,
                NymVpnSubscriptionStatus::Complete => SubscriptionState::Complete,
                NymVpnSubscriptionStatus::Active => SubscriptionState::Active,
            }
        } else {
            SubscriptionState::NotActive
        }
    }
}

impl From<NymVpnAccountSummaryResponse> for AccountSummary {
    fn from(summary: NymVpnAccountSummaryResponse) -> Self {
        Self {
            account: AccountState::from(summary.account),
            subscription: SubscriptionState::from(summary.subscription),
            device_summary: DeviceSummary::from(summary.devices),
            fair_usage: FairUsage::from(summary.fair_usage),
        }
    }
}

impl From<NymVpnAccountSummaryDevices> for DeviceSummary {
    fn from(devices: NymVpnAccountSummaryDevices) -> Self {
        DeviceSummary {
            active: devices.active,
            max: devices.max,
            remaining: devices.remaining,
        }
    }
}

impl From<NymVpnAccountSummaryFairUsage> for FairUsage {
    fn from(fair_usage: NymVpnAccountSummaryFairUsage) -> Self {
        FairUsage {
            used_gb: fair_usage.usedGB,
            limit_gb: fair_usage.limitGB,
            resets_on_utc: fair_usage.resetsOnUtc,
        }
    }
}

impl From<NymVpnDeviceStatus> for DeviceState {
    fn from(status: NymVpnDeviceStatus) -> Self {
        match status {
            NymVpnDeviceStatus::Active => DeviceState::Active,
            NymVpnDeviceStatus::Inactive => DeviceState::Inactive,
            NymVpnDeviceStatus::DeleteMe => DeviceState::DeleteMe,
        }
    }
}
