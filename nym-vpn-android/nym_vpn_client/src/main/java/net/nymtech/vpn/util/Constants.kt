package net.nymtech.vpn.util

import android.system.Os

object Constants {
	const val NYM_VPN_LIB = "nym_vpn_lib"

	const val NYM_VPN_LIB_TAG = "libnymvpn"

	// Add Rust environment vars for lib
	const val DEFAULT_COUNTRY_ISO = "DE"

	fun setupEnvironmentMainnet() {
		Os.setenv("CONFIGURED", "true", true)
		Os.setenv("RUST_LOG", "info", true)
		Os.setenv("RUST_BACKTRACE", "1", true)
		Os.setenv("NETWORK_NAME", "sandbox", true)
		Os.setenv("BECH32_PREFIX", "n", true)
		Os.setenv("MIX_DENOM", "unym", true)
		Os.setenv("MIX_DENOM_DISPLAY", "nym", true)
		Os.setenv("STAKE_DENOM", "unyx", true)
		Os.setenv("STAKE_DENOM_DISPLAY", "nyx", true)
		Os.setenv("DENOMS_EXPONENT", "6", true)

		Os.setenv("REWARDING_VALIDATOR_ADDRESS", "n10yyd98e2tuwu0f7ypz9dy3hhjw7v772q6287gy", true)
		Os.setenv(
			"MIXNET_CONTRACT_ADDRESS",
			"n17srjznxl9dvzdkpwpw24gg668wc73val88a6m5ajg6ankwvz9wtst0cznr",
			true,
		)
		Os.setenv(
			"VESTING_CONTRACT_ADDRESS",
			"n1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrq73f2nw",
			true,
		)

		Os.setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "https://mainnet-stats.nymte.ch:8090", true)
		Os.setenv("EXPLORER_API", "https://explorer.nymtech.net/api/", true)
		Os.setenv("NYXD", "https://rpc.nymtech.net", true)
		Os.setenv("NYXD_WS", "wss://rpc.nymtech.net/websocket", true)
		Os.setenv("NYM_API", "https://validator.nymtech.net/api/", true)
	}

	fun setupEnvironmentSandbox() {
		Os.setenv("CONFIGURED", "true", true)
		Os.setenv("RUST_LOG", "info", true)
		Os.setenv("RUST_BACKTRACE", "1", true)
		Os.setenv("NETWORK_NAME", "sandbox", true)
		Os.setenv("BECH32_PREFIX", "n", true)
		Os.setenv("MIX_DENOM", "unym", true)
		Os.setenv("MIX_DENOM_DISPLAY", "nym", true)
		Os.setenv("STAKE_DENOM", "unyx", true)
		Os.setenv("STAKE_DENOM_DISPLAY", "nyx", true)
		Os.setenv("DENOMS_EXPONENT", "6", true)

		Os.setenv("REWARDING_VALIDATOR_ADDRESS", "n1pefc2utwpy5w78p2kqdsfmpjxfwmn9d39k5mqa", true)
		Os.setenv(
			"MIXNET_CONTRACT_ADDRESS",
			"n1xr3rq8yvd7qplsw5yx90ftsr2zdhg4e9z60h5duusgxpv72hud3sjkxkav",
			true,
		)
		Os.setenv(
			"VESTING_CONTRACT_ADDRESS",
			"n1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qackslz",
			true,
		)
		Os.setenv(
			"COCONUT_BANDWIDTH_CONTRACT_ADDRESS",
			"n13902g92xfefeyzuyed49snlm5fxv5ms6mdq5kvrut27hasdw5a9q9vyw6c",
			true,
		)
		Os.setenv(
			"GROUP_CONTRACT_ADDRESS",
			"n18nczmqw6adwxg2wnlef3hf0etf8anccafp2pjpul5rrtmv96umyq5mv7t5",
			true,
		)
		Os.setenv(
			"MULTISIG_CONTRACT_ADDRESS",
			"n1q3zzxl78rlmxv3vn0uf4vkyz285lk8q2xzne299yt9x6mpfgk90qukuzmv",
			true,
		)
		Os.setenv(
			"COCONUT_DKG_CONTRACT_ADDRESS",
			"n1jsz20ggp5a6v76j060erkzvxmeus8htlpl77yxp878f0gf95cyaq6p2pee",
			true,
		)
		Os.setenv(
			"NAME_SERVICE_CONTRACT_ADDRESS",
			"n12ne7qtmdwd0j03t9t5es8md66wq4e5xg9neladrsag8fx3y89rcs36asfp",
			true,
		)
		Os.setenv(
			"SERVICE_PROVIDER_DIRECTORY_CONTRACT_ADDRESS",
			"n1ps5yutd7sufwg058qd7ac7ldnlazsvmhzqwucsfxmm445d70u8asqxpur4",
			true,
		)
		Os.setenv("EPHEMERA_CONTRACT_ADDRESS", "n19lc9u84cz0yz3fww5283nucc9yvr8gsjmgeul0", true)

		Os.setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "http://0.0.0.0", true)
		Os.setenv("EXPLORER_API", "https://sandbox-explorer.nymtech.net/api", true)
		Os.setenv("NYXD", "https://rpc.sandbox.nymtech.net", true)
		Os.setenv("NYXD_WS", "wss://rpc.sandbox.nymtech.net/websocket", true)
		Os.setenv("NYM_API", "https://sandbox-nym-api1.nymtech.net/api", true)
	}
}
