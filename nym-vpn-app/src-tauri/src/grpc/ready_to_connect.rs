use nym_vpn_proto::is_ready_to_connect_response::IsReadyToConnectResponseType;
use serde::Serialize;
use strum::Display;
use ts_rs::TS;

#[derive(Clone, Serialize, Display, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ReadyToConnect {
    Ready,
    NotReady(String),
}

impl TryFrom<IsReadyToConnectResponseType> for ReadyToConnect {
    type Error = String;

    fn try_from(kind: IsReadyToConnectResponseType) -> Result<Self, Self::Error> {
        match kind {
            IsReadyToConnectResponseType::Unspecified => Err("grpc UNSPECIFIED".to_string()),
            IsReadyToConnectResponseType::Ready => Ok(ReadyToConnect::Ready),
            IsReadyToConnectResponseType::NoAccountStored => Ok(ReadyToConnect::NotReady(
                "no account recovery phrase stored".to_string(),
            )),
            IsReadyToConnectResponseType::AccountNotSynced => Ok(ReadyToConnect::NotReady(
                "the account is not synced".to_string(),
            )),
            IsReadyToConnectResponseType::AccountNotRegistered => Ok(ReadyToConnect::NotReady(
                "the account is not registered".to_string(),
            )),
            IsReadyToConnectResponseType::AccountNotActive => Ok(ReadyToConnect::NotReady(
                "the account is not active".to_string(),
            )),
            IsReadyToConnectResponseType::NoActiveSubscription => Ok(ReadyToConnect::NotReady(
                "the account does not have an active subscription".to_string(),
            )),
            IsReadyToConnectResponseType::DeviceNotRegistered => Ok(ReadyToConnect::NotReady(
                "the device is not registered".to_string(),
            )),
            IsReadyToConnectResponseType::DeviceNotActive => Ok(ReadyToConnect::NotReady(
                "the device is not active".to_string(),
            )),
            IsReadyToConnectResponseType::MaxDevicesReached => Ok(ReadyToConnect::NotReady(
                "maximum number of device reached".to_string(),
            )),
            IsReadyToConnectResponseType::DeviceRegistrationFailed => Ok(ReadyToConnect::NotReady(
                "device registration failed".to_string(),
            )),
        }
    }
}
