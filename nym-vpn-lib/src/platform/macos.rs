// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::gateway_client::{EntryPoint, ExitPoint};
use crate::{NymVpn, UniffiCustomTypeConverter};
use log::warn;
use std::str::FromStr;
use url::Url;

impl UniffiCustomTypeConverter for Url {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Url::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

#[allow(non_snake_case)]
pub async fn initVPN(api_url: Url, entry_gateway: EntryPoint, exit_router: ExitPoint) {
    if get_vpn_state().await != ClientState::Uninitialised {
        warn!("VPN was already inited. Try starting it");
        return;
    }

    let mut vpn = NymVpn::new(entry_gateway, exit_router);
    vpn.gateway_config.api_url = api_url;

    set_inited_vpn(vpn).await
}

#[allow(non_snake_case)]
pub async fn runVPN() {
    let state = get_vpn_state().await;
    if state != ClientState::Disconnected {
        warn!("Invalid vpn state: {:?}", state);
        return;
    }

    let vpn = take_vpn().await.expect("VPN was not inited");

    _async_run_vpn(vpn)
        .await
        .map_err(|err| {
            warn!("failed to run vpn: {}", err);
        })
        .ok();
}

#[allow(non_snake_case)]
pub async fn stopVPN() {
    if get_vpn_state().await != ClientState::Connected {
        warn!("could not stop the vpn as it's not running");
        return;
    }
    stop_and_reset_shutdown_handle().await;
}
