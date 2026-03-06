// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub const CONFIGURED: bool = true;

pub const NETWORK_NAME: &str = "mainnet";

pub const RUST_LOG: &str = "info";
pub const RUST_BACKTRACE: &str = "1";

pub const BECH32_PREFIX_VAR: &str = "BECH32_PREFIX";
pub const BECH32_PREFIX: &str = "n";
pub const MIX_DENOM_VAR: &str = "MIX_DENOM";
pub const MIX_DENOM: &str = "unym";
pub const MIX_DENOM_DISPLAY_VAR: &str = "MIX_DENOM_DISPLAY";
pub const MIX_DENOM_DISPLAY: &str = "nym";
pub const STAKE_DENOM_VAR: &str = "STAKE_DENOM";
pub const STAKE_DENOM: &str = "unyx";
pub const STAKE_DENOM_DISPLAY_VAR: &str = "STAKE_DENOM_DISPLAY";
pub const STAKE_DENOM_DISPLAY: &str = "nyx";
pub const DENOMS_EXPONENT_VAR: &str = "DENOMS_EXPONENT";
pub const DENOMS_EXPONENT: &str = "6";

pub const MIXNET_CONTRACT_ADDRESS_VAR: &str = "MIXNET_CONTRACT_ADDRESS";
pub const MIXNET_CONTRACT_ADDRESS: &str =
    "n17srjznxl9dvzdkpwpw24gg668wc73val88a6m5ajg6ankwvz9wtst0cznr";
pub const VESTING_CONTRACT_ADDRESS_VAR: &str = "VESTING_CONTRACT_ADDRESS";
pub const VESTING_CONTRACT_ADDRESS: &str =
    "n1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrq73f2nw";
pub const GROUP_CONTRACT_ADDRESS_VAR: &str = "GROUP_CONTRACT_ADDRESS";
pub const GROUP_CONTRACT_ADDRESS: &str =
    "n1e2zq4886zzewpvpucmlw8v9p7zv692f6yck4zjzxh699dkcmlrfqk2knsr";
pub const ECASH_CONTRACT_ADDRESS_VAR: &str = "ECASH_CONTRACT_ADDRESS";
pub const ECASH_CONTRACT_ADDRESS: &str =
    "n1r7s6aksyc6pqardx88k3rkgfagwvj4z4zum9mmz2sfk3zm2mha0sd4dnun";
pub const MULTISIG_CONTRACT_ADDRESS_VAR: &str = "MULTISIG_CONTRACT_ADDRESS";
pub const MULTISIG_CONTRACT_ADDRESS: &str =
    "n1txayqfz5g9qww3rlflpg025xd26m9payz96u54x4fe3s2ktz39xqk67gzx";
pub const COCONUT_DKG_CONTRACT_ADDRESS_VAR: &str = "COCONUT_DKG_CONTRACT_ADDRESS";
pub const COCONUT_DKG_CONTRACT_ADDRESS: &str =
    "n19604yflqggs9mk2z26mqygq43q2kr3n932egxx630svywd5mpxjsztfpvx";

pub const REWARDING_VALIDATOR_ADDRESS_VAR: &str = "REWARDING_VALIDATOR_ADDRESS";
pub const REWARDING_VALIDATOR_ADDRESS: &str = "n10yyd98e2tuwu0f7ypz9dy3hhjw7v772q6287gy";
pub const STATISTICS_SERVICE_DOMAIN_ADDRESS_VAR: &str = "STATISTICS_SERVICE_DOMAIN_ADDRESS";
pub const STATISTICS_SERVICE_DOMAIN_ADDRESS: &str = "https: //mainnet-stats.nymte.ch:8090";
pub const NYXD_VAR: &str = "NYXD";
pub const NYXD: &str = "https: //rpc.nymtech.net";
pub const NYM_API_VAR: &str = "NYM_API";
pub const NYM_API: &str = "https: //validator.nymtech.net/api/";
pub const NYXD_WS_VAR: &str = "NYXD_WS";
pub const NYXD_WS: &str = "wss: //rpc.nymtech.net/websocket";
pub const NYM_VPN_API_VAR: &str = "NYM_VPN_API";
pub const NYM_VPN_API: &str = "https: //nymvpn.com/api/";
